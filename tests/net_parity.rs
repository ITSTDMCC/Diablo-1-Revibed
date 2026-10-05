//! The network layer: the libsodium primitives DevilutionX's packet encryption uses, checked
//! against published test vectors (RFC 7693, RFC 8439, RFC 9106 and NaCl's own tests), and the
//! dvlnet packet/frame formats.

use diablo1_rs::dvlnet::crypto::*;

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

#[test]
fn blake2b_rfc7693() {
    // RFC 7693 appendix A: BLAKE2b-512("abc")
    assert_eq!(
        blake2b(64, b"abc"),
        hex("ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923")
    );
    assert_eq!(
        blake2b(64, b""),
        hex("786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce")
    );
}

#[test]
fn poly1305_rfc8439() {
    // RFC 8439 section 2.5.2
    let key: [u8; 32] = hex("85d6be7857556d337f4452fe42d506a80103808afb0db2fd4abff6af4149f51b").try_into().unwrap();
    assert_eq!(poly1305(b"Cryptographic Forum Research Group", &key).to_vec(), hex("a8061dc1305136c6c22b8baf0c0127a9"));
}

#[test]
fn argon2_rfc9106() {
    // RFC 9106 section 5: t=3, m=32 KiB, p=4, 32-byte tag
    let (pw, salt, secret, ad) = ([1u8; 32], [2u8; 16], [3u8; 8], [4u8; 12]);
    let run = |ty| argon2(ty, &pw, &salt, &secret, &ad, 3, 32, 4, 32);
    assert_eq!(run(Argon2Type::D), hex("512b391b6f1162975371d30919734294f868e3be3984f3c1a13a4db9fabe4acb"), "Argon2d");
    assert_eq!(run(Argon2Type::I), hex("c814d9d1dc7f37aa13f0d77f2494bda1c8de6b016dd388d29952a4c4672b6ce8"), "Argon2i");
    assert_eq!(run(Argon2Type::Id), hex("0d640df58d78766c08c037a34a8b53c9d01ef0452d75b65eb52520e96b01e659"), "Argon2id");
}

#[test]
fn hsalsa20_nacl_core1() {
    // NaCl tests/core1.c: HSalsa20 of the shared secret with a zero input gives "firstkey"
    let shared: [u8; 32] = hex("4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742").try_into().unwrap();
    assert_eq!(hsalsa20(&shared, &[0; 16]).to_vec(), hex("1b27556473e985d462cd51197a9a46c76009549eac6474f206c4ee0844f68389"));
}

#[test]
fn secretbox_nacl() {
    // NaCl tests/secretbox.c
    let key: [u8; 32] = hex("1b27556473e985d462cd51197a9a46c76009549eac6474f206c4ee0844f68389").try_into().unwrap();
    let nonce: [u8; 24] = hex("69696ee955b62b73cd62bda875fc73d68219e0036b7a0b37").try_into().unwrap();
    let m = hex(
        "be075fc53c81f2d5cf141316ebeb0c7b5228c52a4c62cbd44b66849b64244ffce5ecbaaf33bd751a1ac728d45e6c61296cdc3c01233561f41db66cce314adb310e3be8250c46f06dceea3a7fa1348057e2f6556ad6b1318a024a838f21af1fde048977eb48f59ffd4924ca1c60902e52f0a089bc76897040e082f937763848645e0705",
    );
    let c = hex(
        "f3ffc7703f9400e52a7dfb4b3d3305d98e993b9f48681273c29650ba32fc76ce48332ea7164d96a4476fb8c531a1186ac0dfc17c98dce87b4da7f011ec48c97271d2c20f9b928fe2270d6fb863d51738b48eeee314a7cc8ab932164548e526ae90224368517acfeabd6bb3732bc0e9da99832b61ca01b6de56244a9e88d5f9b37973f622a43d14a6599b1f654cb45a74e355a5",
    );
    assert_eq!(secretbox_easy(&m, &nonce, &key), c);
    assert_eq!(secretbox_open_easy(&c, &nonce, &key), Some(m.clone()));
    let mut forged = c.clone();
    forged[20] ^= 1;
    assert_eq!(secretbox_open_easy(&forged, &nonce, &key), None);
}

// ---------------------------------------------------------------------------------------------
// dvlnet: packets, frames and the TCP provider

use diablo1_rs::dvlnet::base::NetEnv;
use diablo1_rs::dvlnet::packet::*;
use diablo1_rs::dvlnet::tcp::TcpClient;
use diablo1_rs::multi::MAX_PLRS;
use diablo1_rs::storm::storm_net::{AbstractNet, SnetEvent, EVENT_TYPE_PLAYER_CREATE_GAME, EVENT_TYPE_PLAYER_LEAVE_GAME, PS_CONNECTED, PS_TURN_ARRIVED};

#[test]
fn packet_wire_format() {
    let f = PacketFactory::new();
    // fields in process_data order, little-endian, buffers last
    assert_eq!(f.turn(1, PLR_BROADCAST, Turn { sequence_number: 7, value: 0x01020304 }).data(), &[PT_TURN, 1, 0xFF, 7, 4, 3, 2, 1]);
    assert_eq!(f.join_accept(PLR_MASTER, PLR_BROADCAST, 0xAABBCCDD, 2, vec![9, 9]).data(), &[PT_JOIN_ACCEPT, 0xFE, 0xFF, 0xDD, 0xCC, 0xBB, 0xAA, 2, 9, 9]);
    assert_eq!(f.disconnect(0, PLR_BROADCAST, 3, 0x40000006).data(), &[PT_DISCONNECT, 0, 0xFF, 3, 6, 0, 0, 0x40]);
    let back = f.make_packet_in(f.message(2, 0, b"hi".to_vec()).data().to_vec()).unwrap();
    assert_eq!((back.type_(), back.source(), back.destination(), back.message()), (PT_MESSAGE, 2, 0, &b"hi"[..]));
    assert_eq!(f.make_packet_in(vec![PT_TURN, 0]).unwrap_err(), NetError::Packet);
    assert_eq!(f.make_packet_in(vec![PT_TURN, 0, 0, 1, 2]).unwrap_err(), NetError::Packet);

    // with a password: nonce || MAC || ciphertext, and only the same password opens it
    let secure = PacketFactory::with_password("secret");
    let pkt = secure.echo_request(1, 2, 1234);
    assert_eq!(pkt.data().len(), 24 + 16 + 3 + 4);
    let back = secure.make_packet_in(pkt.data().to_vec()).unwrap();
    assert_eq!((back.type_(), back.time()), (PT_ECHO_REQUEST, 1234));
    assert!(PacketFactory::with_password("other").make_packet_in(pkt.data().to_vec()).is_err());
}

#[test]
fn frame_queue_reassembles() {
    let mut q = FrameQueue::default();
    let mut stream = FrameQueue::make_frame(b"abc");
    stream.extend(FrameQueue::make_frame(b"defgh"));
    assert_eq!(&stream[..4], &[3, 0, 0, 0]);
    // arrives in odd pieces
    for chunk in stream.chunks(3) {
        q.write(chunk.to_vec());
    }
    assert!(q.packet_ready().unwrap());
    assert_eq!(q.read_packet().unwrap(), b"abc");
    assert!(q.packet_ready().unwrap());
    assert_eq!(q.read_packet().unwrap(), b"defgh");
    assert!(!q.packet_ready().unwrap());
    q.write(vec![0, 0, 0, 0]);
    assert_eq!(q.packet_ready(), Err(NetError::FrameQueue));
}

fn env(port: u16) -> NetEnv {
    NetEnv { players: MAX_PLRS, ticks: 0, port, bind_address: "127.0.0.1".into() }
}

fn noop_handler(_: &mut diablo1_rs::ctx::Ctx, _: &SnetEvent) {}

fn tcp_game(port: u16, password: Option<&str>) {
    let game_info = (0u8..24).collect::<Vec<u8>>();
    let mut host = TcpClient::default();
    let mut guest = TcpClient::default();
    for (c, info) in [(&mut host, game_info.clone()), (&mut guest, Vec::new())] {
        c.set_env(env(port));
        c.setup_gameinfo(info);
        match password {
            Some(p) => c.setup_password(p.into()),
            None => c.clear_password(),
        }
        c.snet_register_event_handler(EVENT_TYPE_PLAYER_CREATE_GAME as u8, noop_handler);
        c.snet_register_event_handler(EVENT_TYPE_PLAYER_LEAVE_GAME as u8, noop_handler);
    }
    let mut lb = false;
    assert_eq!(host.create("127.0.0.1", &mut lb), 0, "host is player 0");
    // the guest's join waits for the server, which the host's thread polls
    let guest_thread = std::thread::spawn(move || {
        let id = guest.join("127.0.0.1");
        (guest, id)
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !guest_thread.is_finished() && std::time::Instant::now() < deadline {
        let mut turns = 0;
        host.snet_get_owner_turns_waiting(&mut turns);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let (mut guest, id) = guest_thread.join().unwrap();
    assert_eq!(id, 1, "guest is player 1: {:?}", guest.take_error());
    // the guest learned the game from the join accept
    let events = guest.take_events();
    assert!(events.iter().any(|(_, e)| e.eventid == EVENT_TYPE_PLAYER_CREATE_GAME as u32 && e.data.as_deref() == Some(&game_info[..])));

    // messages
    assert!(host.snet_send_message(1, b"to guest"));
    assert!(guest.snet_send_message(-2, b"to others"));
    let mut got_guest = None;
    let mut got_host = None;
    for _ in 0..200 {
        got_guest = got_guest.or_else(|| guest.snet_receive_message());
        got_host = got_host.or_else(|| host.snet_receive_message());
        if got_guest.is_some() && got_host.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(got_guest, Some((0, b"to guest".to_vec())));
    assert_eq!(got_host, Some((1, b"to others".to_vec())));

    // turns: both send two, both receive the same sequence
    for c in [&mut host, &mut guest] {
        assert!(c.snet_send_turn(&100i32.to_le_bytes()));
        assert!(c.snet_send_turn(&200i32.to_le_bytes()));
    }
    // both poll until each has both turns (the guest waits for the host's sequence numbers)
    let mut received = [Vec::new(), Vec::new()];
    for _ in 0..400 {
        for (k, c) in [&mut host, &mut guest].into_iter().enumerate() {
            let mut data: [Option<Vec<u8>>; MAX_PLRS] = Default::default();
            let mut size = [0usize; MAX_PLRS];
            let mut status = [0u32; MAX_PLRS];
            if c.snet_receive_turns(MAX_PLRS, &mut data, &mut size, &mut status) {
                for p in 0..2 {
                    assert_ne!(status[p] & PS_CONNECTED, 0);
                    assert_ne!(status[p] & PS_TURN_ARRIVED, 0);
                }
                received[k].push((data[0].clone().unwrap(), data[1].clone().unwrap()));
            }
        }
        if received.iter().all(|r| r.len() == 2) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    for r in &received {
        let values: Vec<(i32, i32)> = r.iter().map(|(a, b)| (i32::from_le_bytes(a[..].try_into().unwrap()), i32::from_le_bytes(b[..].try_into().unwrap()))).collect();
        assert_eq!(values, vec![(100, 100), (200, 200)]);
    }

    // the guest leaves: the host is told
    assert!(guest.snet_leave_game(0x40000004, &mut lb));
    let mut left = false;
    for _ in 0..400 {
        let _ = host.snet_receive_message();
        if host.take_events().iter().any(|(_, e)| e.eventid == EVENT_TYPE_PLAYER_LEAVE_GAME as u32 && e.playerid == 1) {
            left = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(left, "host sees the guest leave");
    host.snet_leave_game(0x40000004, &mut lb);
}

#[test]
fn tcp_game_public() {
    tcp_game(26112, None);
}

#[test]
fn tcp_game_with_password() {
    tcp_game(26113, Some("hunter2"));
}
