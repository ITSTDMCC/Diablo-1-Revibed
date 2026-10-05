//! `Source/dvlnet/tcp_server.cpp` and `tcp_client.cpp`: the "Client-Server (TCP)" provider.
//! The host runs a server that relays packets between up to four clients, its own game being
//! one of them (connected over loopback).
//!
//! The original drives asio handlers from `ioc.poll()`; the port polls non-blocking sockets in
//! the same places (`tcp_client::poll`) and runs the same handler bodies. Writes are attempted
//! at once and the rest is flushed on later polls, as asio's speculative `async_write` does.

use std::collections::{HashMap, VecDeque};
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use super::base::{Base, BaseState, NetEnv};
use super::packet::*;
use crate::multi::{GameInfo, MAX_PLRS};
use crate::storm::storm_net::{AbstractNet, EventType, SevtHandler, SnetCaps, SnetEvent, LEAVE_DROP};

/// Writes what the socket takes now; the rest stays queued. Errors are ignored, as the
/// original's empty `HandleSend`; the read side notices a dead connection.
fn flush(socket: &mut TcpStream, send_buf: &mut VecDeque<u8>) -> bool {
    let mut progress = false;
    while !send_buf.is_empty() {
        let (a, _) = send_buf.as_slices();
        match socket.write(a) {
            Ok(0) => break,
            Ok(n) => {
                send_buf.drain(..n);
                progress = true;
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    progress
}

enum ReadResult {
    Data(Vec<u8>),
    WouldBlock,
    /// End of stream or an error: the asio read handler gets an error code.
    Closed,
}

fn read_some(socket: &mut TcpStream) -> ReadResult {
    let mut buf = vec![0u8; FrameQueue::MAX_FRAME_SIZE];
    loop {
        return match socket.read(&mut buf) {
            Ok(0) => ReadResult::Closed,
            Ok(n) => {
                buf.truncate(n);
                ReadResult::Data(buf)
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => ReadResult::WouldBlock,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(_) => ReadResult::Closed,
        };
    }
}

/// `tcp_server::client_connection`
struct ClientConnection {
    recv_queue: FrameQueue,
    plr: PlrT,
    socket: TcpStream,
    send_buf: VecDeque<u8>,
    /// the 1-second `timer`
    next_tick: Instant,
    timeout: i32,
    closed: bool,
}

impl ClientConnection {
    /// Original: `tcp_server::client_connection::client_connection` (dvlnet/tcp_server.h).
    // @port dvlnet/tcp_server.h|devilution::net::tcp_server::client_connection::client_connection(asio::io_context &ioc) sha=0dff378cd9c3
    fn new(socket: TcpStream) -> ClientConnection {
        ClientConnection {
            recv_queue: FrameQueue::default(),
            plr: PLR_BROADCAST,
            socket,
            send_buf: VecDeque::new(),
            next_tick: Instant::now(),
            timeout: 0,
            closed: false,
        }
    }
}

type ConId = u64;

/// `tcp_server`
pub struct TcpServer {
    acceptor: Option<TcpListener>,
    cons: HashMap<ConId, ClientConnection>,
    next_id: ConId,
    connections: [Option<ConId>; MAX_PLRS],
    game_init_info: Vec<u8>,
}

impl TcpServer {
    const TIMEOUT_CONNECT: i32 = 30;
    const TIMEOUT_ACTIVE: i32 = 60;

    /// Original: `tcp_server::tcp_server` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::tcp_server(asio::io_context &ioc, const std::string &bindaddr, unsigned short port, packet_factory &pktfty) sha=9ace6b7c7925
    pub fn new(bindaddr: &str, port: u16) -> std::io::Result<TcpServer> {
        let addr: IpAddr = bindaddr.parse().map_err(|e| std::io::Error::new(ErrorKind::InvalidInput, format!("{e}")))?;
        let acceptor = TcpListener::bind(SocketAddr::new(addr, port))?;
        acceptor.set_nonblocking(true)?;
        // StartAccept: accepting is polled in `poll`
        Ok(TcpServer { acceptor: Some(acceptor), cons: HashMap::new(), next_id: 0, connections: [None; MAX_PLRS], game_init_info: Vec::new() })
    }

    /// Original: `tcp_server::LocalhostSelf` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::LocalhostSelf() sha=8783ca1f5b91
    pub fn localhost_self(&self) -> String {
        let addr = self.acceptor.as_ref().and_then(|a| a.local_addr().ok()).map(|a| a.ip()).unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        if addr.is_unspecified() {
            return match addr {
                IpAddr::V4(_) => Ipv4Addr::LOCALHOST.to_string(),
                IpAddr::V6(_) => Ipv6Addr::LOCALHOST.to_string(),
            };
        }
        addr.to_string()
    }

    /// Original: `tcp_server::MakeConnection` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::MakeConnection() sha=0c2f4a768aa8
    fn make_connection(&mut self, socket: TcpStream) -> ConId {
        let id = self.next_id;
        self.next_id += 1;
        self.cons.insert(id, ClientConnection::new(socket));
        id
    }

    /// Original: `tcp_server::NextFree` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::NextFree() sha=42a09fa724e5
    fn next_free(&self, players: usize) -> PlrT {
        (0..players).find(|&i| self.connections[i].is_none()).map_or(PLR_BROADCAST, |i| i as PlrT)
    }

    /// Original: `tcp_server::Empty` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::Empty() sha=8c9d85ae32bd
    fn empty(&self, players: usize) -> bool {
        (0..players).all(|i| self.connections[i].is_none())
    }

    /// Runs whatever handlers are ready (`ioc.poll()` for the server's sockets and timers).
    /// Returns whether anything happened.
    pub fn poll(&mut self, pktfty: &PacketFactory, players: usize) -> bool {
        let mut progress = false;
        // StartAccept / HandleAccept
        loop {
            let Some(acceptor) = &self.acceptor else { break };
            match acceptor.accept() {
                Ok((socket, _)) => {
                    progress = true;
                    let id = self.make_connection(socket);
                    self.handle_accept(id, players);
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
        let ids: Vec<ConId> = self.cons.keys().copied().collect();
        for id in &ids {
            // StartReceive / HandleReceive
            loop {
                let Some(con) = self.cons.get_mut(id) else { break };
                if con.closed {
                    break;
                }
                match read_some(&mut con.socket) {
                    ReadResult::Data(buf) => {
                        progress = true;
                        if !self.handle_receive(*id, buf, pktfty, players) {
                            break;
                        }
                    }
                    ReadResult::WouldBlock => break,
                    ReadResult::Closed => {
                        progress = true;
                        self.drop_connection(*id, pktfty, players);
                        break;
                    }
                }
            }
        }
        // StartTimeout / HandleTimeout
        let now = Instant::now();
        for id in &ids {
            while let Some(con) = self.cons.get_mut(id) {
                if con.closed || now < con.next_tick {
                    break;
                }
                con.next_tick += Duration::from_secs(1);
                self.handle_timeout(*id, pktfty, players);
            }
        }
        for con in self.cons.values_mut() {
            progress |= flush(&mut con.socket, &mut con.send_buf);
        }
        // a closed connection's socket closes once what was queued for it is gone
        self.cons.retain(|_, c| !(c.closed && c.send_buf.is_empty()));
        progress
    }

    /// Original: `tcp_server::StartReceive` (dvlnet/tcp_server.cpp): reading is polled.
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::StartReceive(const scc &con) sha=5b035f9e9cce
    fn start_receive(&mut self, _id: ConId) {}

    /// Original: `tcp_server::HandleReceive` (dvlnet/tcp_server.cpp). Returns false when the
    /// connection was dropped.
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::HandleReceive(const scc &con, const asio::error_code &ec, size_t bytesRead) sha=da7cbed5d7d2
    fn handle_receive(&mut self, id: ConId, buf: Vec<u8>, pktfty: &PacketFactory, players: usize) -> bool {
        let con = self.cons.get_mut(&id).unwrap();
        con.recv_queue.write(buf);
        loop {
            let con = self.cons.get_mut(&id).unwrap();
            let ready = match con.recv_queue.packet_ready() {
                Ok(r) => r,
                Err(e) => {
                    crate::platform::log::info!("Invalid packet: {}", e);
                    self.drop_connection(id, pktfty, players);
                    return false;
                }
            };
            if !ready {
                break;
            }
            let plr = con.plr;
            let result = con.recv_queue.read_packet().and_then(|b| pktfty.make_packet_in(b)).and_then(|pkt| {
                if plr == PLR_BROADCAST {
                    self.handle_receive_new_player(id, &pkt, pktfty, players)
                } else {
                    self.cons.get_mut(&id).unwrap().timeout = Self::TIMEOUT_ACTIVE;
                    self.handle_receive_packet(&pkt)
                }
            });
            if let Err(e) = result {
                match e {
                    NetError::FrameQueue => crate::platform::log::info!("Invalid packet: {}", e),
                    _ => crate::platform::log::info!("Network error: {}", e),
                }
                self.drop_connection(id, pktfty, players);
                return false;
            }
        }
        self.start_receive(id);
        true
    }

    /// Original: `tcp_server::HandleReceiveNewPlayer` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::HandleReceiveNewPlayer(const scc &con, packet &pkt) sha=89d56e1cae41
    fn handle_receive_new_player(&mut self, id: ConId, pkt: &Packet, pktfty: &PacketFactory, players: usize) -> Result<(), NetError> {
        let newplr = self.next_free(players);
        if newplr == PLR_BROADCAST {
            return Err(NetError::Server);
        }
        if self.empty(players) {
            self.game_init_info = pkt.info().to_vec();
        }
        for player in 0..players {
            if let Some(other) = self.connections[player] {
                let player_packet = pktfty.connect(PLR_MASTER, PLR_BROADCAST, newplr, Vec::new());
                self.start_send(other, &player_packet);
                let newplr_packet = pktfty.connect(PLR_MASTER, PLR_BROADCAST, player as PlrT, Vec::new());
                self.start_send(id, &newplr_packet);
            }
        }
        let reply = pktfty.join_accept(PLR_MASTER, PLR_BROADCAST, pkt.cookie(), newplr, self.game_init_info.clone());
        self.start_send(id, &reply);
        let con = self.cons.get_mut(&id).unwrap();
        con.plr = newplr;
        con.timeout = Self::TIMEOUT_ACTIVE;
        self.connections[newplr as usize] = Some(id);
        Ok(())
    }

    /// Original: `tcp_server::HandleReceivePacket` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::HandleReceivePacket(packet &pkt) sha=62ea56607ec8
    fn handle_receive_packet(&mut self, pkt: &Packet) -> Result<(), NetError> {
        self.send_packet(pkt)
    }

    /// Original: `tcp_server::SendPacket` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::SendPacket(packet &pkt) sha=961af3fb3e8e
    fn send_packet(&mut self, pkt: &Packet) -> Result<(), NetError> {
        if pkt.destination() == PLR_BROADCAST {
            for i in 0..MAX_PLRS {
                if i != pkt.source() as usize {
                    if let Some(id) = self.connections[i] {
                        self.start_send(id, pkt);
                    }
                }
            }
        } else {
            if pkt.destination() as usize >= MAX_PLRS {
                return Err(NetError::Server);
            }
            if pkt.destination() != pkt.source() {
                if let Some(id) = self.connections[pkt.destination() as usize] {
                    self.start_send(id, pkt);
                }
            }
        }
        Ok(())
    }

    /// Original: `tcp_server::StartSend` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::StartSend(const scc &con, packet &pkt) sha=263e836a7ede
    fn start_send(&mut self, id: ConId, pkt: &Packet) {
        let Some(con) = self.cons.get_mut(&id) else { return };
        con.send_buf.extend(FrameQueue::make_frame(pkt.data()));
        flush(&mut con.socket, &mut con.send_buf);
        Self::handle_send();
    }

    /// Original: `tcp_server::HandleSend` (dvlnet/tcp_server.cpp): empty for now.
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::HandleSend(const scc &con, const asio::error_code &ec, size_t bytesSent) sha=58dbfb33bf70
    fn handle_send() {}

    /// Original: `tcp_server::StartAccept` (dvlnet/tcp_server.cpp): accepting is polled.
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::StartAccept() sha=075d94184200
    fn start_accept(&mut self) {}

    /// Original: `tcp_server::HandleAccept` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::HandleAccept(const scc &con, const asio::error_code &ec) sha=478641e8539c
    fn handle_accept(&mut self, id: ConId, players: usize) {
        if self.next_free(players) == PLR_BROADCAST {
            let con = self.cons.get_mut(&id).unwrap();
            con.closed = true; // DropConnection of a connection without a player
        } else {
            let con = self.cons.get_mut(&id).unwrap();
            let _ = con.socket.set_nonblocking(true);
            let _ = con.socket.set_nodelay(true);
            con.timeout = Self::TIMEOUT_CONNECT;
            self.start_receive(id);
            self.start_timeout(id);
        }
        self.start_accept();
    }

    /// Original: `tcp_server::StartTimeout` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::StartTimeout(const scc &con) sha=0308b9672b0b
    fn start_timeout(&mut self, id: ConId) {
        if let Some(con) = self.cons.get_mut(&id) {
            con.next_tick = Instant::now() + Duration::from_secs(1);
        }
    }

    /// Original: `tcp_server::HandleTimeout` (dvlnet/tcp_server.cpp). The next tick is set by
    /// `poll`.
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::HandleTimeout(const scc &con, const asio::error_code &ec) sha=bf9b49e47a06
    fn handle_timeout(&mut self, id: ConId, pktfty: &PacketFactory, players: usize) {
        let con = self.cons.get_mut(&id).unwrap();
        if con.timeout > 0 {
            con.timeout -= 1;
        }
        if con.timeout <= 0 {
            con.timeout = 0;
            self.drop_connection(id, pktfty, players);
        }
    }

    /// Original: `tcp_server::DropConnection` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::DropConnection(const scc &con) sha=53a5150d067e
    fn drop_connection(&mut self, id: ConId, pktfty: &PacketFactory, _players: usize) {
        let Some(con) = self.cons.get_mut(&id) else { return };
        if con.closed {
            return;
        }
        con.closed = true;
        con.send_buf.clear();
        let _ = con.socket.shutdown(std::net::Shutdown::Both);
        let plr = con.plr;
        if plr != PLR_BROADCAST {
            let pkt = pktfty.disconnect(PLR_MASTER, PLR_BROADCAST, plr, LEAVE_DROP);
            self.connections[plr as usize] = None;
            let _ = self.send_packet(&pkt);
            // TODO: investigate if it is really ok for the server to
            //       drop a client directly.
        }
    }

    /// Original: `tcp_server::Close` (dvlnet/tcp_server.cpp).
    // @port dvlnet/tcp_server.cpp|devilution::net::tcp_server::Close() sha=4f91c355bb8b
    pub fn close(&mut self) {
        self.acceptor = None;
    }
}

/// `tcp_client`
#[derive(Default)]
pub struct TcpClient {
    base: BaseState,
    recv_queue: FrameQueue,
    sock: Option<TcpStream>,
    send_buf: VecDeque<u8>,
    receiving: bool,
    local_server: Option<TcpServer>,
}

impl TcpClient {
    /// Original: `tcp_client::create` (dvlnet/tcp_client.cpp).
    // @port dvlnet/tcp_client.cpp|devilution::net::tcp_client::create(std::string addrstr) sha=0b6645154524
    fn tcp_create(&mut self, addrstr: &str) -> i32 {
        let port = self.base.env.port;
        match TcpServer::new(addrstr, port) {
            Ok(server) => {
                let self_addr = server.localhost_self();
                self.local_server = Some(server);
                self.tcp_join(&self_addr)
            }
            Err(e) => {
                self.base.error = Some(e.to_string());
                -1
            }
        }
    }

    /// Original: `tcp_client::join` (dvlnet/tcp_client.cpp).
    // @port dvlnet/tcp_client.cpp|devilution::net::tcp_client::join(std::string addrstr) sha=b5c70dac2515
    fn tcp_join(&mut self, addrstr: &str) -> i32 {
        const MS_SLEEP: u64 = 10;
        const NO_SLEEP: i32 = 250;
        let port = self.base.env.port;
        let connected = (addrstr, port).to_socket_addrs().and_then(|addrs| {
            let addrs: Vec<SocketAddr> = addrs.collect();
            // the host's own server is polled by this thread, so connect while polling it
            let sock = TcpStream::connect(&addrs[..])?;
            sock.set_nodelay(true)?;
            sock.set_nonblocking(true)?;
            Ok(sock)
        });
        match connected {
            Ok(sock) => self.sock = Some(sock),
            Err(e) => {
                self.base.error = Some(e.to_string());
                return -1;
            }
        }
        self.start_receive();
        {
            self.base.cookie_self = generate_cookie();
            let pkt = self.base.pktfty.join_request(PLR_BROADCAST, PLR_MASTER, self.base.cookie_self, self.base.game_init_info.clone());
            self.send(&pkt);
            for _ in 0..NO_SLEEP {
                if let Err(e) = self.poll_inner() {
                    self.base.error = Some(format!("Network error: {e}"));
                    return -1;
                }
                if self.base.plr_self != PLR_BROADCAST {
                    break; // join successful
                }
                std::thread::sleep(Duration::from_millis(MS_SLEEP));
            }
        }
        if self.base.plr_self == PLR_BROADCAST {
            self.base.error = Some(crate::utils::language::tr("Unable to connect"));
            return -1;
        }
        self.base.plr_self as i32
    }

    /// `ioc.poll()`: the local server's handlers, then this client's, until nothing is ready.
    fn poll_inner(&mut self) -> Result<(), NetError> {
        for _ in 0..16 {
            let mut progress = false;
            if let Some(server) = &mut self.local_server {
                progress |= server.poll(&self.base.pktfty, self.base.env.players);
            }
            if let Some(sock) = &mut self.sock {
                progress |= flush(sock, &mut self.send_buf);
            }
            while self.receiving {
                let Some(sock) = &mut self.sock else { break };
                match read_some(sock) {
                    ReadResult::Data(buf) => {
                        progress = true;
                        self.handle_receive(buf)?;
                    }
                    ReadResult::WouldBlock => break,
                    // error in recv from server
                    // returning and doing nothing should be the same
                    // as if all connections to other clients were lost
                    ReadResult::Closed => self.receiving = false,
                }
            }
            if !progress {
                break;
            }
        }
        Ok(())
    }

    /// Original: `tcp_client::HandleReceive` (dvlnet/tcp_client.cpp).
    // @port dvlnet/tcp_client.cpp|devilution::net::tcp_client::HandleReceive(const asio::error_code &error, size_t bytesRead) sha=09614cce0c6a
    fn handle_receive(&mut self, buf: Vec<u8>) -> Result<(), NetError> {
        self.recv_queue.write(buf);
        while self.recv_queue.packet_ready()? {
            let pkt = self.base.pktfty.make_packet_in(self.recv_queue.read_packet()?)?;
            self.recv_local(&pkt);
        }
        self.start_receive();
        Ok(())
    }

    /// Original: `tcp_client::StartReceive` (dvlnet/tcp_client.cpp): reading is polled.
    // @port dvlnet/tcp_client.cpp|devilution::net::tcp_client::StartReceive() sha=908b8d76688e
    fn start_receive(&mut self) {
        self.receiving = true;
    }

    /// Original: `tcp_client::HandleSend` (dvlnet/tcp_client.cpp): empty for now.
    // @port dvlnet/tcp_client.cpp|devilution::net::tcp_client::HandleSend(const asio::error_code &error, size_t bytesSent) sha=72bdda5dcdc8
    fn handle_send(&mut self) {}

    /// Original: `tcp_client::SNetLeaveGame` (dvlnet/tcp_client.cpp).
    // @port dvlnet/tcp_client.cpp|devilution::net::tcp_client::SNetLeaveGame(int type) sha=80d50c1c533a
    fn tcp_snet_leave_game(&mut self, type_: i32) -> bool {
        let ret = self.base_snet_leave_game(type_);
        self.poll();
        if let Some(server) = &mut self.local_server {
            server.close();
        }
        if let Some(sock) = self.sock.take() {
            let _ = sock.shutdown(std::net::Shutdown::Both);
        }
        ret
    }
}

impl Base for TcpClient {
    fn base(&mut self) -> &mut BaseState {
        &mut self.base
    }

    fn base_ref(&self) -> &BaseState {
        &self.base
    }

    /// Original: `tcp_client::poll` (dvlnet/tcp_client.cpp). A malformed packet from the
    /// server ends the game, as the original's uncaught exception does.
    // @port dvlnet/tcp_client.cpp|devilution::net::tcp_client::poll() sha=1ab415e7b542
    fn poll(&mut self) {
        if let Err(e) = self.poll_inner() {
            panic!("Network error: {e}");
        }
    }

    /// Original: `tcp_client::send` (dvlnet/tcp_client.cpp).
    // @port dvlnet/tcp_client.cpp|devilution::net::tcp_client::send(packet &pkt) sha=835c14ee4e4f
    fn send(&mut self, pkt: &Packet) {
        self.send_buf.extend(FrameQueue::make_frame(pkt.data()));
        if let Some(sock) = &mut self.sock {
            flush(sock, &mut self.send_buf);
        }
        self.handle_send();
    }

    /// Original: `tcp_client::IsGameHost` (dvlnet/tcp_client.cpp).
    // @port dvlnet/tcp_client.cpp|devilution::net::tcp_client::IsGameHost() sha=001383dc8aef
    fn is_game_host(&self) -> bool {
        self.local_server.is_some()
    }
}

impl AbstractNet for TcpClient {
    fn create(&mut self, addrstr: &str, _is_loopback: &mut bool) -> i32 {
        self.tcp_create(addrstr)
    }

    fn join(&mut self, addrstr: &str) -> i32 {
        self.tcp_join(addrstr)
    }

    fn snet_receive_message(&mut self) -> Option<(u8, Vec<u8>)> {
        self.base_snet_receive_message()
    }

    fn snet_send_message(&mut self, dest: i32, data: &[u8]) -> bool {
        self.base_snet_send_message(dest, data)
    }

    fn snet_receive_turns(&mut self, _players: usize, data: &mut [Option<Vec<u8>>; MAX_PLRS], size: &mut [usize; MAX_PLRS], status: &mut [u32; MAX_PLRS]) -> bool {
        self.base_snet_receive_turns(data, size, status)
    }

    fn snet_send_turn(&mut self, data: &[u8]) -> bool {
        self.base_snet_send_turn(data)
    }

    fn snet_get_provider_caps(&self, caps: &mut SnetCaps) {
        self.base_snet_get_provider_caps(caps)
    }

    fn snet_register_event_handler(&mut self, evtype: EventType, func: SevtHandler) -> bool {
        self.base_snet_register_event_handler(evtype, func)
    }

    fn snet_unregister_event_handler(&mut self, evtype: EventType) -> bool {
        self.base_snet_unregister_event_handler(evtype)
    }

    fn snet_leave_game(&mut self, type_: i32, _is_loopback: &mut bool) -> bool {
        self.tcp_snet_leave_game(type_)
    }

    fn snet_drop_player(&mut self, playerid: i32, flags: u32) -> bool {
        self.base_snet_drop_player(playerid, flags)
    }

    fn snet_get_owner_turns_waiting(&mut self, turns: &mut u32) -> bool {
        self.base_snet_get_owner_turns_waiting(turns)
    }

    fn snet_get_turns_in_transit(&mut self, turns: &mut u32) -> bool {
        self.base_snet_get_turns_in_transit(turns)
    }

    fn setup_gameinfo(&mut self, info: Vec<u8>) {
        self.base_setup_gameinfo(info)
    }

    /// Original: `tcp_client::make_default_gamename` (dvlnet/tcp_client.cpp).
    // @port dvlnet/tcp_client.cpp|devilution::net::tcp_client::make_default_gamename() sha=a282a3db2d09
    fn make_default_gamename(&self) -> String {
        self.base.env.bind_address.clone()
    }

    fn setup_password(&mut self, passwd: String) {
        self.base_setup_password(&passwd)
    }

    fn clear_password(&mut self) {
        self.base_clear_password()
    }

    fn get_gamelist(&mut self) -> Vec<GameInfo> {
        Vec::new()
    }

    fn set_env(&mut self, env: NetEnv) {
        self.base.env = env;
    }

    fn take_events(&mut self) -> Vec<(SevtHandler, SnetEvent)> {
        std::mem::take(&mut self.base.events)
    }

    fn take_error(&mut self) -> Option<String> {
        self.base.error.take()
    }
}
