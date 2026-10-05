//! `Source/dvlnet/packet.cpp`, `packet.h` and `frame_queue.cpp`: the wire format shared by the
//! TCP client and server. Fields are little-endian, in the order `packet_proc::process_data`
//! visits them; byte buffers take the rest of the packet. With a game password the whole packet
//! is a `crypto_secretbox_easy` box behind a random 24-byte nonce.

use std::collections::VecDeque;

use super::crypto;

pub type PlrT = u8;
pub type SeqT = u8;
pub type CookieT = u32;
pub type TimestampT = u32;
pub type LeaveinfoT = i32;

/// `packet_type`
pub const PT_MESSAGE: u8 = 0x01;
pub const PT_TURN: u8 = 0x02;
pub const PT_JOIN_REQUEST: u8 = 0x11;
pub const PT_JOIN_ACCEPT: u8 = 0x12;
pub const PT_CONNECT: u8 = 0x13;
pub const PT_DISCONNECT: u8 = 0x14;
pub const PT_INFO_REQUEST: u8 = 0x21;
pub const PT_INFO_REPLY: u8 = 0x22;
pub const PT_ECHO_REQUEST: u8 = 0x31;
pub const PT_ECHO_REPLY: u8 = 0x32;

pub const PLR_MASTER: PlrT = 0xFE;
pub const PLR_BROADCAST: PlrT = 0xFF;

/// `turn_t`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Turn {
    pub sequence_number: SeqT,
    pub value: i32,
}

/// `dvlnet_exception` and its subclasses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetError {
    /// `packet_exception`: "Incorrect package size" (also a failed decryption)
    Packet,
    /// `wrong_packet_type_exception`
    WrongPacketType(String),
    /// `frame_queue_exception`: "Incorrect frame size"
    FrameQueue,
    /// `server_exception`: "Invalid player ID"
    Server,
}

impl std::fmt::Display for NetError {
    /// The `what()` of the original's exception classes.
    // @port dvlnet/abstract_net.h|devilution::net::dvlnet_exception::what()
    // @port dvlnet/packet.h|devilution::net::packet_exception::what()
    // @port dvlnet/packet.h|devilution::net::wrong_packet_type_exception::what()
    // @port dvlnet/frame_queue.h|devilution::net::frame_queue_exception::what()
    // @port dvlnet/tcp_server.h|devilution::net::server_exception::what()
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetError::Packet => write!(f, "Incorrect package size"),
            NetError::WrongPacketType(m) => write!(f, "{m}"),
            NetError::FrameQueue => write!(f, "Incorrect frame size"),
            NetError::Server => write!(f, "Invalid player ID"),
        }
    }
}

/// Original: `devilution::net::packet_type_to_string` (dvlnet/packet.cpp).
// @port dvlnet/packet.cpp|devilution::net::packet_type_to_string(uint8_t packetType)
pub fn packet_type_to_string(packet_type: u8) -> Option<&'static str> {
    Some(match packet_type {
        PT_MESSAGE => "PT_MESSAGE",
        PT_TURN => "PT_TURN",
        PT_JOIN_REQUEST => "PT_JOIN_REQUEST",
        PT_JOIN_ACCEPT => "PT_JOIN_ACCEPT",
        PT_CONNECT => "PT_CONNECT",
        PT_DISCONNECT => "PT_DISCONNECT",
        PT_INFO_REQUEST => "PT_INFO_REQUEST",
        PT_INFO_REPLY => "PT_INFO_REPLY",
        PT_ECHO_REQUEST => "PT_ECHO_REQUEST",
        PT_ECHO_REPLY => "PT_ECHO_REPLY",
        _ => return None,
    })
}

/// Original: `wrong_packet_type_exception::wrong_packet_type_exception` (dvlnet/packet.cpp).
// @port dvlnet/packet.cpp|devilution::net::wrong_packet_type_exception::wrong_packet_type_exception(std::initializer_list<packet_type> expectedTypes, std::uint8_t actual)
fn wrong_packet_type_message(expected_types: &[u8], actual: u8) -> String {
    let name = |t: u8| packet_type_to_string(t).map(str::to_string).unwrap_or_else(|| t.to_string());
    let expected: Vec<String> = expected_types.iter().map(|&t| name(t)).collect();
    format!("Expected packet of type {}, got{}", expected.join(" or "), name(actual))
}

/// Original: `CheckPacketTypeOneOf` (dvlnet/packet.cpp). The original throws; reading a field
/// of the wrong packet type is a bug in the caller, so the port stops there.
// @port dvlnet/packet.cpp|devilution::net::CheckPacketTypeOneOf(std::initializer_list<packet_type> expectedTypes, std::uint8_t actualType)
fn check_packet_type_one_of(expected_types: &[u8], actual_type: u8) {
    if !expected_types.contains(&actual_type) {
        panic!("{}", wrong_packet_type_message(expected_types, actual_type));
    }
}

/// `packet` (with `packet_in` / `packet_out`)
#[derive(Clone, Debug, Default)]
pub struct Packet {
    m_type: u8,
    m_src: PlrT,
    m_dest: PlrT,
    m_message: Vec<u8>,
    m_turn: Turn,
    m_cookie: CookieT,
    m_newplr: PlrT,
    m_time: TimestampT,
    m_info: Vec<u8>,
    m_leaveinfo: LeaveinfoT,
    have_encrypted: bool,
    have_decrypted: bool,
    encrypted_buffer: Vec<u8>,
    decrypted_buffer: Vec<u8>,
}

impl Packet {
    /// Original: `packet::Data` (dvlnet/packet.cpp): the bytes that go on the wire.
    // @port dvlnet/packet.cpp|devilution::net::packet::Data()
    pub fn data(&self) -> &[u8] {
        assert!(self.have_encrypted || self.have_decrypted);
        if self.have_encrypted { &self.encrypted_buffer } else { &self.decrypted_buffer }
    }

    /// Original: `packet::Type` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet::Type()
    pub fn type_(&self) -> u8 {
        self.m_type
    }

    /// Original: `packet::Source` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet::Source()
    pub fn source(&self) -> PlrT {
        self.m_src
    }

    /// Original: `packet::Destination` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet::Destination()
    pub fn destination(&self) -> PlrT {
        self.m_dest
    }

    /// Original: `packet::Message` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet::Message()
    pub fn message(&self) -> &[u8] {
        check_packet_type_one_of(&[PT_MESSAGE], self.m_type);
        &self.m_message
    }

    /// Original: `packet::Turn` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet::Turn()
    pub fn turn(&self) -> Turn {
        check_packet_type_one_of(&[PT_TURN], self.m_type);
        self.m_turn
    }

    /// Original: `packet::Cookie` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet::Cookie()
    pub fn cookie(&self) -> CookieT {
        check_packet_type_one_of(&[PT_JOIN_REQUEST, PT_JOIN_ACCEPT], self.m_type);
        self.m_cookie
    }

    /// Original: `packet::NewPlayer` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet::NewPlayer()
    pub fn new_player(&self) -> PlrT {
        check_packet_type_one_of(&[PT_JOIN_ACCEPT, PT_CONNECT, PT_DISCONNECT], self.m_type);
        self.m_newplr
    }

    /// Original: `packet::Time` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet::Time()
    pub fn time(&self) -> TimestampT {
        check_packet_type_one_of(&[PT_ECHO_REQUEST, PT_ECHO_REPLY], self.m_type);
        self.m_time
    }

    /// Original: `packet::Info` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet::Info()
    pub fn info(&self) -> &[u8] {
        check_packet_type_one_of(&[PT_JOIN_REQUEST, PT_JOIN_ACCEPT, PT_CONNECT, PT_INFO_REPLY], self.m_type);
        &self.m_info
    }

    /// Original: `packet::LeaveInfo` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet::LeaveInfo()
    pub fn leave_info(&self) -> LeaveinfoT {
        check_packet_type_one_of(&[PT_DISCONNECT], self.m_type);
        self.m_leaveinfo
    }

    /// `packet_proc<packet_in>::process_data`: reads the fields from `decrypted_buffer`.
    fn process_data_in(&mut self) -> Result<(), NetError> {
        let mut buf: &[u8] = &self.decrypted_buffer;
        fn take<const N: usize>(buf: &mut &[u8]) -> Result<[u8; N], NetError> {
            if buf.len() < N {
                return Err(NetError::Packet);
            }
            let (head, rest) = buf.split_at(N);
            *buf = rest;
            Ok(head.try_into().unwrap())
        }
        fn rest(buf: &mut &[u8]) -> Vec<u8> {
            let v = buf.to_vec();
            *buf = &[];
            v
        }
        self.m_type = take::<1>(&mut buf)?[0];
        self.m_src = take::<1>(&mut buf)?[0];
        self.m_dest = take::<1>(&mut buf)?[0];
        match self.m_type {
            PT_MESSAGE => self.m_message = rest(&mut buf),
            PT_TURN => {
                self.m_turn.sequence_number = take::<1>(&mut buf)?[0];
                self.m_turn.value = i32::from_le_bytes(take::<4>(&mut buf)?);
            }
            PT_JOIN_REQUEST => {
                self.m_cookie = u32::from_le_bytes(take::<4>(&mut buf)?);
                self.m_info = rest(&mut buf);
            }
            PT_JOIN_ACCEPT => {
                self.m_cookie = u32::from_le_bytes(take::<4>(&mut buf)?);
                self.m_newplr = take::<1>(&mut buf)?[0];
                self.m_info = rest(&mut buf);
            }
            PT_CONNECT => {
                self.m_newplr = take::<1>(&mut buf)?[0];
                self.m_info = rest(&mut buf);
            }
            PT_DISCONNECT => {
                self.m_newplr = take::<1>(&mut buf)?[0];
                self.m_leaveinfo = i32::from_le_bytes(take::<4>(&mut buf)?);
            }
            PT_INFO_REPLY => self.m_info = rest(&mut buf),
            PT_ECHO_REQUEST | PT_ECHO_REPLY => self.m_time = u32::from_le_bytes(take::<4>(&mut buf)?),
            _ => {}
        }
        // packet_in consumes the decrypted buffer as it reads
        let consumed = self.decrypted_buffer.len() - buf.len();
        self.decrypted_buffer.drain(..consumed);
        Ok(())
    }

    /// `packet_proc<packet_out>::process_data`: writes the fields to `decrypted_buffer`.
    fn process_data_out(&mut self) {
        let mut b = Vec::new();
        b.push(self.m_type);
        b.push(self.m_src);
        b.push(self.m_dest);
        match self.m_type {
            PT_MESSAGE => b.extend_from_slice(&self.m_message),
            PT_TURN => {
                b.push(self.m_turn.sequence_number);
                b.extend_from_slice(&self.m_turn.value.to_le_bytes());
            }
            PT_JOIN_REQUEST => {
                b.extend_from_slice(&self.m_cookie.to_le_bytes());
                b.extend_from_slice(&self.m_info);
            }
            PT_JOIN_ACCEPT => {
                b.extend_from_slice(&self.m_cookie.to_le_bytes());
                b.push(self.m_newplr);
                b.extend_from_slice(&self.m_info);
            }
            PT_CONNECT => {
                b.push(self.m_newplr);
                b.extend_from_slice(&self.m_info);
            }
            PT_DISCONNECT => {
                b.push(self.m_newplr);
                b.extend_from_slice(&self.m_leaveinfo.to_le_bytes());
            }
            PT_INFO_REPLY => b.extend_from_slice(&self.m_info),
            PT_ECHO_REQUEST | PT_ECHO_REPLY => b.extend_from_slice(&self.m_time.to_le_bytes()),
            _ => {}
        }
        self.decrypted_buffer = b;
    }

    /// Original: `packet_in::Create` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet_in::Create(buffer_t buf)
    fn create_in(&mut self, buf: Vec<u8>) -> Result<(), NetError> {
        if buf.len() < 3 {
            return Err(NetError::Packet);
        }
        self.decrypted_buffer = buf;
        self.have_decrypted = true;
        // TCP server implementation forwards the original data to clients
        // so although we are not decrypting anything,
        // we save a copy in encrypted_buffer anyway
        self.encrypted_buffer = self.decrypted_buffer.clone();
        self.have_encrypted = true;
        Ok(())
    }

    /// Original: `packet_in::Decrypt` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet_in::Decrypt(buffer_t buf)
    fn decrypt(&mut self, buf: Vec<u8>, key: &[u8; 32]) -> Result<(), NetError> {
        self.encrypted_buffer = buf;
        self.have_encrypted = true;
        if self.encrypted_buffer.len() < crypto::SECRETBOX_NONCEBYTES + crypto::SECRETBOX_MACBYTES + 3 {
            return Err(NetError::Packet);
        }
        let nonce: [u8; 24] = self.encrypted_buffer[..crypto::SECRETBOX_NONCEBYTES].try_into().unwrap();
        match crypto::secretbox_open_easy(&self.encrypted_buffer[crypto::SECRETBOX_NONCEBYTES..], &nonce, key) {
            Some(m) => self.decrypted_buffer = m,
            None => return Err(NetError::Packet),
        }
        self.have_decrypted = true;
        Ok(())
    }

    /// Original: `packet_out::Encrypt` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet_out::Encrypt()
    fn encrypt(&mut self, key: &[u8; 32]) {
        if self.have_encrypted {
            return;
        }
        let mut nonce = [0u8; 24];
        crypto::randombytes_buf(&mut nonce);
        let mut out = nonce.to_vec();
        out.extend_from_slice(&crypto::secretbox_easy(&self.decrypted_buffer, &nonce, key));
        self.encrypted_buffer = out;
        self.have_encrypted = true;
    }
}

/// Original: `packet_out::GenerateCookie` (dvlnet/packet.cpp), the `PACKET_ENCRYPTION` version.
// @port dvlnet/packet.cpp|devilution::net::packet_out::GenerateCookie()
pub fn generate_cookie() -> CookieT {
    let mut b = [0u8; 4];
    crypto::randombytes_buf(&mut b);
    u32::from_le_bytes(b)
}

/// `packet_factory`
#[derive(Clone)]
pub struct PacketFactory {
    key: [u8; 32],
    secure: bool,
}

impl Default for PacketFactory {
    fn default() -> Self {
        PacketFactory::new()
    }
}

impl PacketFactory {
    pub const MAX_PACKET_SIZE: usize = 0xFFFF;

    /// Original: `packet_factory::packet_factory()` (dvlnet/packet.cpp).
    // @port dvlnet/packet.cpp|devilution::net::packet_factory::packet_factory()
    pub fn new() -> PacketFactory {
        PacketFactory { key: [0; 32], secure: false }
    }

    /// Original: `packet_factory::packet_factory(std::string pw)` (dvlnet/packet.cpp): the
    /// key is Argon2id of the password with a fixed salt.
    // @port dvlnet/packet.cpp|devilution::net::packet_factory::packet_factory(std::string pw)
    pub fn with_password(pw: &str) -> PacketFactory {
        let mut pw = pw.as_bytes().to_vec();
        pw.truncate(crypto::PWHASH_ARGON2ID_PASSWD_MAX);
        let mut salt = [0u8; 16];
        salt.copy_from_slice(b"W9bE9dQgVaeybwr2");
        let key = crypto::crypto_pwhash_argon2id(
            crypto::SECRETBOX_KEYBYTES,
            &pw,
            &salt,
            3 * crypto::PWHASH_ARGON2ID_OPSLIMIT_MIN,
            2 * crypto::PWHASH_ARGON2ID_MEMLIMIT_MIN,
        );
        PacketFactory { key: key.try_into().unwrap(), secure: true }
    }

    /// Original: `packet_factory::make_packet(buffer_t buf)` (dvlnet/packet.h): a received packet.
    // @port dvlnet/packet.h|devilution::net::packet_factory::make_packet(buffer_t buf)
    pub fn make_packet_in(&self, buf: Vec<u8>) -> Result<Packet, NetError> {
        let mut ret = Packet::default();
        if !self.secure {
            ret.create_in(buf)?;
        } else {
            ret.decrypt(buf, &self.key)?;
        }
        ret.process_data_in()?;
        Ok(ret)
    }

    /// `packet_factory::make_packet<t>(args...)`: a packet to send.
    fn make_out(&self, mut p: Packet) -> Packet {
        p.have_decrypted = true;
        p.process_data_out();
        if self.secure {
            p.encrypt(&self.key);
        }
        p
    }

    fn base(t: u8, s: PlrT, d: PlrT) -> Packet {
        Packet { m_type: t, m_src: s, m_dest: d, ..Default::default() }
    }

    /// `make_packet<PT_INFO_REQUEST>`
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_INFO_REQUEST>(plr_t s, plr_t d)
    pub fn info_request(&self, s: PlrT, d: PlrT) -> Packet {
        self.make_out(Self::base(PT_INFO_REQUEST, s, d))
    }

    /// `make_packet<PT_INFO_REPLY>`
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_INFO_REPLY>(plr_t s, plr_t d, buffer_t i)
    pub fn info_reply(&self, s: PlrT, d: PlrT, i: Vec<u8>) -> Packet {
        self.make_out(Packet { m_info: i, ..Self::base(PT_INFO_REPLY, s, d) })
    }

    /// `make_packet<PT_MESSAGE>`
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_MESSAGE>(plr_t s, plr_t d, buffer_t m)
    pub fn message(&self, s: PlrT, d: PlrT, m: Vec<u8>) -> Packet {
        self.make_out(Packet { m_message: m, ..Self::base(PT_MESSAGE, s, d) })
    }

    /// `make_packet<PT_TURN>`
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_TURN>(plr_t s, plr_t d, turn_t u)
    pub fn turn(&self, s: PlrT, d: PlrT, u: Turn) -> Packet {
        self.make_out(Packet { m_turn: u, ..Self::base(PT_TURN, s, d) })
    }

    /// `make_packet<PT_JOIN_REQUEST>`
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_JOIN_REQUEST>(plr_t s, plr_t d, cookie_t c, buffer_t i)
    pub fn join_request(&self, s: PlrT, d: PlrT, c: CookieT, i: Vec<u8>) -> Packet {
        self.make_out(Packet { m_cookie: c, m_info: i, ..Self::base(PT_JOIN_REQUEST, s, d) })
    }

    /// `make_packet<PT_JOIN_ACCEPT>`
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_JOIN_ACCEPT>(plr_t s, plr_t d, cookie_t c, plr_t n, buffer_t i)
    pub fn join_accept(&self, s: PlrT, d: PlrT, c: CookieT, n: PlrT, i: Vec<u8>) -> Packet {
        self.make_out(Packet { m_cookie: c, m_newplr: n, m_info: i, ..Self::base(PT_JOIN_ACCEPT, s, d) })
    }

    /// `make_packet<PT_CONNECT>` (both overloads; the info is empty for the 3-argument one)
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_CONNECT>(plr_t s, plr_t d, plr_t n, buffer_t i)
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_CONNECT>(plr_t s, plr_t d, plr_t n)
    pub fn connect(&self, s: PlrT, d: PlrT, n: PlrT, i: Vec<u8>) -> Packet {
        self.make_out(Packet { m_newplr: n, m_info: i, ..Self::base(PT_CONNECT, s, d) })
    }

    /// `make_packet<PT_DISCONNECT>`
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_DISCONNECT>(plr_t s, plr_t d, plr_t n, leaveinfo_t l)
    pub fn disconnect(&self, s: PlrT, d: PlrT, n: PlrT, l: LeaveinfoT) -> Packet {
        self.make_out(Packet { m_newplr: n, m_leaveinfo: l, ..Self::base(PT_DISCONNECT, s, d) })
    }

    /// `make_packet<PT_ECHO_REQUEST>`
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_ECHO_REQUEST>(plr_t s, plr_t d, timestamp_t t)
    pub fn echo_request(&self, s: PlrT, d: PlrT, t: TimestampT) -> Packet {
        self.make_out(Packet { m_time: t, ..Self::base(PT_ECHO_REQUEST, s, d) })
    }

    /// `make_packet<PT_ECHO_REPLY>`
    // @port dvlnet/packet.h|devilution::net::packet_out::create<PT_ECHO_REPLY>(plr_t s, plr_t d, timestamp_t t)
    pub fn echo_reply(&self, s: PlrT, d: PlrT, t: TimestampT) -> Packet {
        self.make_out(Packet { m_time: t, ..Self::base(PT_ECHO_REPLY, s, d) })
    }
}

/// `framesize_t`
pub type FramesizeT = u32;

/// `frame_queue`: reassembles length-prefixed frames from a byte stream.
#[derive(Default)]
pub struct FrameQueue {
    current_size: usize,
    buffer_deque: VecDeque<Vec<u8>>,
    nextsize: FramesizeT,
}

impl FrameQueue {
    pub const MAX_FRAME_SIZE: usize = 0xFFFF;

    /// Original: `frame_queue::Size` (dvlnet/frame_queue.cpp).
    // @port dvlnet/frame_queue.cpp|devilution::net::frame_queue::Size()
    fn size(&self) -> usize {
        self.current_size
    }

    /// Original: `frame_queue::Read` (dvlnet/frame_queue.cpp).
    // @port dvlnet/frame_queue.cpp|devilution::net::frame_queue::Read(framesize_t s)
    fn read(&mut self, mut s: usize) -> Result<Vec<u8>, NetError> {
        if self.current_size < s {
            return Err(NetError::FrameQueue);
        }
        let mut ret = Vec::with_capacity(s);
        while s > 0 && s >= self.buffer_deque.front().unwrap().len() {
            let front = self.buffer_deque.pop_front().unwrap();
            s -= front.len();
            self.current_size -= front.len();
            ret.extend_from_slice(&front);
        }
        if s > 0 {
            let front = self.buffer_deque.front_mut().unwrap();
            ret.extend_from_slice(&front[..s]);
            front.drain(..s);
            self.current_size -= s;
        }
        Ok(ret)
    }

    /// Original: `frame_queue::Write` (dvlnet/frame_queue.cpp).
    // @port dvlnet/frame_queue.cpp|devilution::net::frame_queue::Write(buffer_t buf)
    pub fn write(&mut self, buf: Vec<u8>) {
        self.current_size += buf.len();
        self.buffer_deque.push_back(buf);
    }

    /// Original: `frame_queue::PacketReady` (dvlnet/frame_queue.cpp).
    // @port dvlnet/frame_queue.cpp|devilution::net::frame_queue::PacketReady()
    pub fn packet_ready(&mut self) -> Result<bool, NetError> {
        if self.nextsize == 0 {
            if self.size() < 4 {
                return Ok(false);
            }
            let szbuf = self.read(4)?;
            self.nextsize = u32::from_le_bytes(szbuf.try_into().unwrap());
            if self.nextsize == 0 {
                return Err(NetError::FrameQueue);
            }
        }
        Ok(self.size() >= self.nextsize as usize)
    }

    /// Original: `frame_queue::ReadPacket` (dvlnet/frame_queue.cpp).
    // @port dvlnet/frame_queue.cpp|devilution::net::frame_queue::ReadPacket()
    pub fn read_packet(&mut self) -> Result<Vec<u8>, NetError> {
        if self.nextsize == 0 || self.size() < self.nextsize as usize {
            return Err(NetError::FrameQueue);
        }
        let ret = self.read(self.nextsize as usize)?;
        self.nextsize = 0;
        Ok(ret)
    }

    /// Original: `frame_queue::MakeFrame` (dvlnet/frame_queue.cpp).
    // @port dvlnet/frame_queue.cpp|devilution::net::frame_queue::MakeFrame(buffer_t packetbuf)
    pub fn make_frame(packetbuf: &[u8]) -> Vec<u8> {
        if packetbuf.len() > Self::MAX_FRAME_SIZE {
            panic!("ABORT: frame too large");
        }
        let mut ret = (packetbuf.len() as FramesizeT).to_le_bytes().to_vec();
        ret.extend_from_slice(packetbuf);
        ret
    }
}
