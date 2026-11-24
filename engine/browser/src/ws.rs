//! A WebSocket client (RFC 6455), as much as the DevTools protocol needs: the opening handshake,
//! masked client frames, unmasked server frames of any length, fragmented messages, ping and
//! close. Plain `ws://` only: Chromium's DevTools endpoint listens on the loopback interface.

use std::hash::{BuildHasher, RandomState};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

pub const OP_CONTINUATION: u8 = 0x0;
pub const OP_TEXT: u8 = 0x1;
pub const OP_BINARY: u8 = 0x2;
pub const OP_CLOSE: u8 = 0x8;
pub const OP_PING: u8 = 0x9;
pub const OP_PONG: u8 = 0xA;

/// RFC 6455 § 1.3: appended to the client's key before hashing.
const ACCEPT_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// A byte stream with a read timeout: a TCP socket, or an in-memory stream in tests.
pub trait Stream: Read + Write {
    fn set_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()>;
}

impl Stream for TcpStream {
    fn set_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        self.set_read_timeout(timeout)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    pub fin: bool,
    pub opcode: u8,
    pub payload: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<u16>),
}

/// The value the server must answer a key with (`Sec-WebSocket-Accept`).
pub fn accept_key(key: &str) -> String {
    crate::base64::encode(&crate::sha1::digest(
        format!("{key}{ACCEPT_GUID}").as_bytes(),
    ))
}

pub fn handshake_request(host: &str, path: &str, key: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
    )
}

/// Checks the head of the server's answer: status 101 and the accept value of `key`.
pub fn check_response(head: &str, key: &str) -> Result<(), String> {
    let mut lines = head.split("\r\n");
    let status = lines.next().unwrap_or_default();
    if status.split_whitespace().nth(1) != Some("101") {
        return Err(format!("WebSocket handshake refused: `{status}`"));
    }
    let want = accept_key(key);
    let accept = lines.find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.trim()
            .eq_ignore_ascii_case("sec-websocket-accept")
            .then(|| value.trim().to_string())
    });
    match accept {
        Some(got) if got == want => Ok(()),
        Some(got) => Err(format!(
            "WebSocket handshake: accept value `{got}`, expected `{want}`"
        )),
        None => Err("WebSocket handshake: no Sec-WebSocket-Accept header".to_string()),
    }
}

/// One frame; a client passes a mask (RFC 6455 § 5.3), a server none.
pub fn encode_frame(fin: bool, opcode: u8, payload: &[u8], mask: Option<[u8; 4]>) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 14);
    out.push(if fin { 0x80 } else { 0 } | (opcode & 0x0f));
    let mask_bit = if mask.is_some() { 0x80 } else { 0 };
    match payload.len() {
        n if n < 126 => out.push(mask_bit | n as u8),
        n if n <= 0xffff => {
            out.push(mask_bit | 126);
            out.extend_from_slice(&(n as u16).to_be_bytes());
        }
        n => {
            out.push(mask_bit | 127);
            out.extend_from_slice(&(n as u64).to_be_bytes());
        }
    }
    match mask {
        Some(m) => {
            out.extend_from_slice(&m);
            out.extend(payload.iter().enumerate().map(|(i, b)| b ^ m[i % 4]));
        }
        None => out.extend_from_slice(payload),
    }
    out
}

/// The first frame in `buf` and the bytes it took, or `None` when `buf` holds only part of it.
pub fn decode_frame(buf: &[u8]) -> Result<Option<(Frame, usize)>, String> {
    if buf.len() < 2 {
        return Ok(None);
    }
    let fin = buf[0] & 0x80 != 0;
    if buf[0] & 0x70 != 0 {
        return Err(
            "WebSocket frame with reserved bits set: no extension was negotiated".to_string(),
        );
    }
    let opcode = buf[0] & 0x0f;
    let masked = buf[1] & 0x80 != 0;
    let (len, mut at) = match buf[1] & 0x7f {
        126 => {
            let Some(b) = buf.get(2..4) else {
                return Ok(None);
            };
            (u64::from(u16::from_be_bytes([b[0], b[1]])), 4)
        }
        127 => {
            let Some(b) = buf.get(2..10) else {
                return Ok(None);
            };
            let mut n = [0u8; 8];
            n.copy_from_slice(b);
            (u64::from_be_bytes(n), 10)
        }
        n => (u64::from(n), 2),
    };
    let len = usize::try_from(len).map_err(|_| "WebSocket frame too long".to_string())?;
    let mask = if masked {
        let Some(m) = buf.get(at..at + 4) else {
            return Ok(None);
        };
        at += 4;
        Some([m[0], m[1], m[2], m[3]])
    } else {
        None
    };
    let Some(data) = buf.get(at..at + len) else {
        return Ok(None);
    };
    let payload = match mask {
        Some(m) => data.iter().enumerate().map(|(i, b)| b ^ m[i % 4]).collect(),
        None => data.to_vec(),
    };
    Ok(Some((
        Frame {
            fin,
            opcode,
            payload,
        },
        at + len,
    )))
}

/// Joins fragments into messages; control frames pass through between fragments (§ 5.4).
#[derive(Default)]
pub struct Assembler {
    partial: Option<(u8, Vec<u8>)>,
}

impl Assembler {
    pub fn push(&mut self, frame: Frame) -> Result<Option<Message>, String> {
        match frame.opcode {
            OP_PING => return Ok(Some(Message::Ping(frame.payload))),
            OP_PONG => return Ok(Some(Message::Pong(frame.payload))),
            OP_CLOSE => {
                let code = (frame.payload.len() >= 2)
                    .then(|| u16::from_be_bytes([frame.payload[0], frame.payload[1]]));
                return Ok(Some(Message::Close(code)));
            }
            _ => {}
        }
        let (opcode, mut data) = match (frame.opcode, self.partial.take()) {
            (OP_CONTINUATION, Some((op, mut data))) => {
                data.extend_from_slice(&frame.payload);
                (op, data)
            }
            (OP_CONTINUATION, None) => {
                return Err("WebSocket continuation frame without a first fragment".to_string());
            }
            (OP_TEXT | OP_BINARY, None) => (frame.opcode, frame.payload),
            (OP_TEXT | OP_BINARY, Some(_)) => {
                return Err("WebSocket message started inside another one".to_string());
            }
            (op, _) => return Err(format!("WebSocket opcode {op} is not defined")),
        };
        if !frame.fin {
            self.partial = Some((opcode, data));
            return Ok(None);
        }
        if opcode == OP_TEXT {
            let text = String::from_utf8(std::mem::take(&mut data))
                .map_err(|_| "WebSocket text message that is not UTF-8".to_string())?;
            Ok(Some(Message::Text(text)))
        } else {
            Ok(Some(Message::Binary(data)))
        }
    }
}

/// `ws://host:port/path` into `host:port` and `/path`.
pub fn parse_url(url: &str) -> Result<(String, String), String> {
    let rest = url
        .strip_prefix("ws://")
        .ok_or_else(|| format!("`{url}` is not a ws:// URL"))?;
    let (host, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    if host.is_empty() {
        return Err(format!("`{url}` names no host"));
    }
    Ok((host.to_string(), path.to_string()))
}

/// Random bytes for keys and masks. RFC 6455 asks for unpredictability against a hostile
/// intermediary; the loopback DevTools endpoint has none, so the hasher's seed suffices.
fn random_bytes<const N: usize>() -> [u8; N] {
    let state = RandomState::new();
    let mut out = [0u8; N];
    for (i, chunk) in out.chunks_mut(8).enumerate() {
        let v = state.hash_one(i as u64).to_le_bytes();
        chunk.copy_from_slice(&v[..chunk.len()]);
    }
    out
}

pub struct WebSocket<S> {
    stream: S,
    buf: Vec<u8>,
    assembler: Assembler,
}

impl WebSocket<TcpStream> {
    /// Opens `ws://host:port/path` and runs the handshake.
    pub fn connect(url: &str, timeout: Duration) -> Result<Self, String> {
        let (host, path) = parse_url(url)?;
        let addr = std::net::ToSocketAddrs::to_socket_addrs(&host)
            .map_err(|e| format!("{host}: {e}"))?
            .next()
            .ok_or_else(|| format!("{host}: no address"))?;
        let mut stream =
            TcpStream::connect_timeout(&addr, timeout).map_err(|e| format!("{url}: {e}"))?;
        stream.set_nodelay(true).ok();
        let key = crate::base64::encode(&random_bytes::<16>());
        stream
            .write_all(handshake_request(&host, &path, &key).as_bytes())
            .map_err(|e| format!("{url}: {e}"))?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| e.to_string())?;
        let mut head = Vec::new();
        let mut byte = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            match stream.read(&mut byte) {
                Ok(0) => return Err(format!("{url}: closed during the handshake")),
                Ok(_) => head.push(byte[0]),
                Err(e) => return Err(format!("{url}: {e}")),
            }
            if head.len() > 16 * 1024 {
                return Err(format!("{url}: handshake answer too long"));
            }
        }
        check_response(&String::from_utf8_lossy(&head), &key)?;
        Ok(Self::over(stream))
    }
}

impl<S: Stream> WebSocket<S> {
    /// A socket over a stream whose handshake is done.
    pub fn over(stream: S) -> Self {
        WebSocket {
            stream,
            buf: Vec::new(),
            assembler: Assembler::default(),
        }
    }

    pub fn into_inner(self) -> S {
        self.stream
    }

    fn send_frame(&mut self, opcode: u8, payload: &[u8]) -> Result<(), String> {
        let frame = encode_frame(true, opcode, payload, Some(random_bytes::<4>()));
        self.stream
            .write_all(&frame)
            .and_then(|()| self.stream.flush())
            .map_err(|e| format!("WebSocket write: {e}"))
    }

    pub fn send_text(&mut self, text: &str) -> Result<(), String> {
        self.send_frame(OP_TEXT, text.as_bytes())
    }

    pub fn close(&mut self) {
        let _ = self.send_frame(OP_CLOSE, &1000u16.to_be_bytes());
    }

    /// The next text message, or `None` when none arrived by `deadline`. Pings are answered on the
    /// way; a close from the server is an error.
    pub fn recv(&mut self, deadline: Instant) -> Result<Option<String>, String> {
        loop {
            while let Some((frame, used)) = decode_frame(&self.buf)? {
                self.buf.drain(..used);
                match self.assembler.push(frame)? {
                    Some(Message::Text(text)) => return Ok(Some(text)),
                    Some(Message::Ping(payload)) => self.send_frame(OP_PONG, &payload)?,
                    Some(Message::Close(code)) => {
                        return Err(format!("WebSocket closed by the browser ({code:?})"));
                    }
                    Some(Message::Binary(_) | Message::Pong(_)) | None => {}
                }
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }
            let wait = (deadline - now).max(Duration::from_millis(1));
            self.stream
                .set_timeout(Some(wait))
                .map_err(|e| e.to_string())?;
            let mut chunk = [0u8; 64 * 1024];
            match self.stream.read(&mut chunk) {
                Ok(0) => return Err("WebSocket: the browser closed the connection".to_string()),
                Ok(n) => self.buf.extend_from_slice(&chunk[..n]),
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::Interrupted
                    ) => {}
                Err(e) => return Err(format!("WebSocket read: {e}")),
            }
        }
    }
}
