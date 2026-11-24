//! The WebSocket client (RFC 6455): handshake, framing, reassembly, over an in-memory stream.

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};
use ux_browser::ws::{self, Assembler, Frame, Message, Stream, WebSocket};

const RFC_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
const RFC_ACCEPT: &str = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";
const MASK: [u8; 4] = [0x37, 0xfa, 0x21, 0x3d];

#[test]
fn accept_key_is_the_rfc_example() {
    assert_eq!(ws::accept_key(RFC_KEY), RFC_ACCEPT);
}

#[test]
fn handshake_request_names_the_upgrade_and_the_key() {
    let request = ws::handshake_request("127.0.0.1:9222", "/devtools/browser/abc", RFC_KEY);
    assert!(request.starts_with("GET /devtools/browser/abc HTTP/1.1\r\n"));
    for line in [
        "Host: 127.0.0.1:9222\r\n",
        "Upgrade: websocket\r\n",
        "Connection: Upgrade\r\n",
        "Sec-WebSocket-Version: 13\r\n",
    ] {
        assert!(request.contains(line), "{line:?} in {request:?}");
    }
    assert!(request.contains(&format!("Sec-WebSocket-Key: {RFC_KEY}\r\n")));
    assert!(request.ends_with("\r\n\r\n"));
}

#[test]
fn handshake_response_must_switch_and_carry_the_accept_key() {
    let good = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nsec-websocket-accept: {RFC_ACCEPT}\r\n\r\n"
    );
    assert!(ws::check_response(&good, RFC_KEY).is_ok());
    let wrong_status = good.replace("101 Switching Protocols", "400 Bad Request");
    assert!(ws::check_response(&wrong_status, RFC_KEY).is_err());
    let wrong_key = good.replace(RFC_ACCEPT, "AAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    assert!(ws::check_response(&wrong_key, RFC_KEY).is_err());
}

#[test]
fn encodes_the_rfc_masked_hello() {
    let frame = ws::encode_frame(true, ws::OP_TEXT, b"Hello", Some(MASK));
    assert_eq!(
        frame,
        [
            0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58
        ]
    );
}

#[test]
fn decodes_the_rfc_examples() {
    let unmasked = [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f];
    let (frame, used) = ws::decode_frame(&unmasked).unwrap().unwrap();
    assert_eq!(used, 7);
    assert!(frame.fin);
    assert_eq!(frame.opcode, ws::OP_TEXT);
    assert_eq!(frame.payload, b"Hello");

    let masked = [
        0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58,
    ];
    let (frame, used) = ws::decode_frame(&masked).unwrap().unwrap();
    assert_eq!(used, 11);
    assert_eq!(frame.payload, b"Hello");
}

#[test]
fn decodes_16_and_64_bit_lengths() {
    let payload = vec![7u8; 256];
    let mut bytes = vec![0x82, 0x7e, 0x01, 0x00];
    bytes.extend(&payload);
    let (frame, used) = ws::decode_frame(&bytes).unwrap().unwrap();
    assert_eq!(
        (frame.opcode, used, frame.payload.len()),
        (ws::OP_BINARY, 260, 256)
    );

    let payload = vec![9u8; 65536];
    let mut bytes = vec![0x82, 0x7f, 0, 0, 0, 0, 0, 1, 0, 0];
    bytes.extend(&payload);
    let (frame, used) = ws::decode_frame(&bytes).unwrap().unwrap();
    assert_eq!((used, frame.payload.len()), (65546, 65536));

    for len in [0usize, 125, 126, 65535, 65536] {
        let data = vec![1u8; len];
        let encoded = ws::encode_frame(true, ws::OP_BINARY, &data, Some(MASK));
        let (frame, used) = ws::decode_frame(&encoded).unwrap().unwrap();
        assert_eq!((used, frame.payload), (encoded.len(), data), "length {len}");
    }
}

#[test]
fn an_incomplete_frame_asks_for_more_bytes() {
    let bytes = [0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f];
    assert!(ws::decode_frame(&bytes).unwrap().is_none());
    assert!(ws::decode_frame(&[0x81]).unwrap().is_none());
    assert!(ws::decode_frame(&[0x82, 0x7e, 0x01]).unwrap().is_none());
}

#[test]
fn reassembles_fragments_and_passes_control_frames_between_them() {
    let mut asm = Assembler::default();
    let first = Frame {
        fin: false,
        opcode: ws::OP_TEXT,
        payload: b"Hel".to_vec(),
    };
    assert_eq!(asm.push(first).unwrap(), None);
    let ping = Frame {
        fin: true,
        opcode: ws::OP_PING,
        payload: b"p".to_vec(),
    };
    assert_eq!(asm.push(ping).unwrap(), Some(Message::Ping(b"p".to_vec())));
    let last = Frame {
        fin: true,
        opcode: ws::OP_CONTINUATION,
        payload: b"lo".to_vec(),
    };
    assert_eq!(
        asm.push(last).unwrap(),
        Some(Message::Text("Hello".to_string()))
    );
}

#[test]
fn a_continuation_without_a_start_is_refused() {
    let mut asm = Assembler::default();
    let stray = Frame {
        fin: true,
        opcode: ws::OP_CONTINUATION,
        payload: vec![],
    };
    assert!(asm.push(stray).is_err());
}

#[test]
fn a_close_frame_carries_its_status() {
    let mut asm = Assembler::default();
    let close = Frame {
        fin: true,
        opcode: ws::OP_CLOSE,
        payload: vec![0x03, 0xe8],
    };
    assert_eq!(asm.push(close).unwrap(), Some(Message::Close(Some(1000))));
}

/// A stream that serves scripted bytes in small chunks and keeps what is written.
#[derive(Default)]
struct Memory {
    incoming: VecDeque<Vec<u8>>,
    written: Vec<u8>,
}

impl Read for Memory {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let Some(mut chunk) = self.incoming.pop_front() else {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "no more bytes"));
        };
        let n = chunk.len().min(buf.len());
        buf[..n].copy_from_slice(&chunk[..n]);
        if n < chunk.len() {
            self.incoming.push_front(chunk.split_off(n));
        }
        Ok(n)
    }
}

impl Write for Memory {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stream for Memory {
    fn set_timeout(&mut self, _: Option<Duration>) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_socket_answers_a_ping_and_reads_a_split_text_message() {
    let mut memory = Memory::default();
    // A ping, then "Hello" in two fragments, delivered three bytes at a time.
    let mut bytes = vec![0x89, 0x01, b'p'];
    bytes.extend([0x01, 0x03, b'H', b'e', b'l', 0x80, 0x02, b'l', b'o']);
    for chunk in bytes.chunks(3) {
        memory.incoming.push_back(chunk.to_vec());
    }
    let mut socket = WebSocket::over(memory);
    let deadline = Instant::now() + Duration::from_secs(1);
    assert_eq!(socket.recv(deadline).unwrap(), Some("Hello".to_string()));
    socket.send_text("{}").unwrap();
    let written = socket.into_inner().written;
    // The pong answers the ping with its payload, masked as every client frame is.
    let (pong, used) = ws::decode_frame(&written).unwrap().unwrap();
    assert_eq!((pong.opcode, pong.payload), (ws::OP_PONG, b"p".to_vec()));
    assert_eq!(written[1] & 0x80, 0x80, "client frames are masked");
    let (text, _) = ws::decode_frame(&written[used..]).unwrap().unwrap();
    assert_eq!((text.opcode, text.payload), (ws::OP_TEXT, b"{}".to_vec()));
}

#[test]
fn a_socket_with_nothing_to_read_returns_none_at_the_deadline() {
    let mut socket = WebSocket::over(Memory::default());
    assert_eq!(socket.recv(Instant::now()).unwrap(), None);
}

#[test]
fn splits_a_devtools_url() {
    assert_eq!(
        ws::parse_url("ws://127.0.0.1:41234/devtools/browser/0f-1").unwrap(),
        (
            "127.0.0.1:41234".to_string(),
            "/devtools/browser/0f-1".to_string()
        )
    );
    assert!(ws::parse_url("wss://example.org/x").is_err());
}
