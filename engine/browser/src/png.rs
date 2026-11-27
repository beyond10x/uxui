//! PNG images as the visual ratchet reads and writes them, with no third-party crate: a reader of
//! 8-bit, non-interlaced greyscale, grey and alpha, RGB and RGBA images (what Chromium's
//! `Page.captureScreenshot` writes) into RGBA pixels, with the inflate of RFC 1951; and a writer of
//! RGBA images, filtered by row (`Up`) and deflated with the fixed Huffman codes and a simple LZ77.

/// An image of `width` x `height` pixels, four bytes each (red, green, blue, alpha), row by row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Image {
    /// An image of one colour.
    pub fn filled(width: u32, height: u32, rgba: [u8; 4]) -> Image {
        let n = width as usize * height as usize;
        Image {
            width,
            height,
            rgba: rgba.iter().copied().cycle().take(n * 4).collect(),
        }
    }

    /// The four bytes of the pixel at `x`, `y`.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = (y as usize * self.width as usize + x as usize) * 4;
        [
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ]
    }

    /// Fills the rectangle from `x0`, `y0` to `x1`, `y1` (exclusive), cut to the image.
    pub fn fill(&mut self, x0: i64, y0: i64, x1: i64, y1: i64, rgba: [u8; 4]) {
        let (w, h) = (i64::from(self.width), i64::from(self.height));
        let (x0, x1) = (x0.clamp(0, w), x1.clamp(0, w));
        let (y0, y1) = (y0.clamp(0, h), y1.clamp(0, h));
        for y in y0..y1 {
            for x in x0..x1 {
                let i = (y * w + x) as usize * 4;
                self.rgba[i..i + 4].copy_from_slice(&rgba);
            }
        }
    }
}

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// Reads a PNG file into RGBA pixels.
pub fn decode(bytes: &[u8]) -> Result<Image, String> {
    if bytes.get(..8) != Some(&SIGNATURE[..]) {
        return Err("not a PNG file".to_string());
    }
    let mut at = 8;
    let mut header: Option<(u32, u32, u8)> = None;
    let mut data = Vec::new();
    while at + 8 <= bytes.len() {
        let len = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        let kind = &bytes[at + 4..at + 8];
        let body = bytes
            .get(at + 8..at + 8 + len)
            .ok_or("a PNG chunk runs past the end of the file")?;
        match kind {
            b"IHDR" => {
                if len != 13 {
                    return Err("the PNG header is not 13 bytes".to_string());
                }
                let word = |i: usize| u32::from_be_bytes(body[i..i + 4].try_into().unwrap());
                let (depth, colour, interlace) = (body[8], body[9], body[12]);
                if depth != 8 || interlace != 0 || ![0, 2, 4, 6].contains(&colour) {
                    return Err(format!(
                        "PNG of bit depth {depth}, colour type {colour}, interlace {interlace}: only 8-bit, non-interlaced grey, grey and alpha, RGB and RGBA are read"
                    ));
                }
                header = Some((word(0), word(4), colour));
            }
            b"IDAT" => data.extend_from_slice(body),
            b"IEND" => break,
            _ => {}
        }
        at += 12 + len;
    }
    let (width, height, colour) = header.ok_or("the PNG has no header")?;
    if data.len() < 2 {
        return Err("the PNG has no image data".to_string());
    }
    let raw = inflate(&data[2..])?;
    let channels = match colour {
        0 => 1,
        4 => 2,
        2 => 3,
        _ => 4,
    };
    let stride = width as usize * channels;
    if raw.len() < (stride + 1) * height as usize {
        return Err("the PNG's image data is shorter than its size".to_string());
    }
    let mut rows = vec![0u8; stride * height as usize];
    for y in 0..height as usize {
        let filter = raw[y * (stride + 1)];
        let line = &raw[y * (stride + 1) + 1..(y + 1) * (stride + 1)];
        let (done, rest) = rows.split_at_mut(y * stride);
        let prior = if y == 0 {
            None
        } else {
            Some(&done[(y - 1) * stride..])
        };
        let out = &mut rest[..stride];
        for i in 0..stride {
            let a = if i >= channels { out[i - channels] } else { 0 };
            let b = prior.map_or(0, |p| p[i]);
            let c = if i >= channels {
                prior.map_or(0, |p| p[i - channels])
            } else {
                0
            };
            let predicted = match filter {
                0 => 0,
                1 => a,
                2 => b,
                3 => ((u16::from(a) + u16::from(b)) / 2) as u8,
                4 => paeth(a, b, c),
                f => return Err(format!("PNG row filter {f} is not one of 0 to 4")),
            };
            out[i] = line[i].wrapping_add(predicted);
        }
    }
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for px in rows.chunks_exact(channels) {
        match channels {
            1 => rgba.extend_from_slice(&[px[0], px[0], px[0], 255]),
            2 => rgba.extend_from_slice(&[px[0], px[0], px[0], px[1]]),
            3 => rgba.extend_from_slice(&[px[0], px[1], px[2], 255]),
            _ => rgba.extend_from_slice(px),
        }
    }
    Ok(Image {
        width,
        height,
        rgba,
    })
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = i16::from(a) + i16::from(b) - i16::from(c);
    let (pa, pb, pc) = (
        (p - i16::from(a)).abs(),
        (p - i16::from(b)).abs(),
        (p - i16::from(c)).abs(),
    );
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

// ---------------------------------------------------------------- inflate (RFC 1951)

struct Bits<'a> {
    data: &'a [u8],
    at: usize,
    bit: u32,
}

impl Bits<'_> {
    fn bit(&mut self) -> Result<u32, String> {
        let byte = *self
            .data
            .get(self.at)
            .ok_or("the deflate stream ends early")?;
        let b = u32::from(byte >> self.bit) & 1;
        self.bit += 1;
        if self.bit == 8 {
            self.bit = 0;
            self.at += 1;
        }
        Ok(b)
    }

    fn bits(&mut self, n: u32) -> Result<u32, String> {
        let mut v = 0;
        for i in 0..n {
            v |= self.bit()? << i;
        }
        Ok(v)
    }
}

/// A canonical Huffman code: the number of codes of each length, and the symbols in code order.
struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Huffman {
        let mut counts = [0u16; 16];
        for &l in lengths {
            counts[l as usize] += 1;
        }
        counts[0] = 0;
        let mut offsets = [0u16; 16];
        for l in 1..16 {
            offsets[l] = offsets[l - 1] + counts[l - 1];
        }
        let mut symbols = vec![0u16; lengths.len()];
        for (s, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbols[offsets[l as usize] as usize] = s as u16;
                offsets[l as usize] += 1;
            }
        }
        Huffman { counts, symbols }
    }

    fn decode(&self, bits: &mut Bits) -> Result<u16, String> {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for len in 1..16 {
            code |= bits.bit()? as i32;
            let count = i32::from(self.counts[len]);
            if code - count < first {
                return Ok(self.symbols[(index + code - first) as usize]);
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err("a deflate code is not in its table".to_string())
    }
}

const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

fn fixed_lengths() -> (Vec<u8>, Vec<u8>) {
    let mut lit = vec![8u8; 288];
    lit[144..256].fill(9);
    lit[256..280].fill(7);
    (lit, vec![5u8; 30])
}

/// Inflates a raw deflate stream (RFC 1951).
pub fn inflate(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut bits = Bits {
        data,
        at: 0,
        bit: 0,
    };
    let mut out = Vec::new();
    loop {
        let last = bits.bit()?;
        match bits.bits(2)? {
            0 => {
                if bits.bit != 0 {
                    bits.bit = 0;
                    bits.at += 1;
                }
                let head = data
                    .get(bits.at..bits.at + 4)
                    .ok_or("a stored block ends early")?;
                let len = usize::from(u16::from_le_bytes([head[0], head[1]]));
                let nlen = u16::from_le_bytes([head[2], head[3]]);
                if nlen != !(len as u16) {
                    return Err("a stored block's length does not match its complement".to_string());
                }
                let body = data
                    .get(bits.at + 4..bits.at + 4 + len)
                    .ok_or("a stored block ends early")?;
                out.extend_from_slice(body);
                bits.at += 4 + len;
            }
            1 => {
                let (lit, dist) = fixed_lengths();
                block(
                    &mut bits,
                    &mut out,
                    &Huffman::new(&lit),
                    &Huffman::new(&dist),
                )?;
            }
            2 => {
                let hlit = bits.bits(5)? as usize + 257;
                let hdist = bits.bits(5)? as usize + 1;
                let hclen = bits.bits(4)? as usize + 4;
                const ORDER: [usize; 19] = [
                    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
                ];
                let mut code_lengths = [0u8; 19];
                for &i in &ORDER[..hclen] {
                    code_lengths[i] = bits.bits(3)? as u8;
                }
                let codes = Huffman::new(&code_lengths);
                let mut lengths = Vec::with_capacity(hlit + hdist);
                while lengths.len() < hlit + hdist {
                    let sym = codes.decode(&mut bits)?;
                    let (value, times) = match sym {
                        0..=15 => (sym as u8, 1),
                        16 => (
                            *lengths.last().ok_or("a repeat with no length before it")?,
                            3 + bits.bits(2)?,
                        ),
                        17 => (0, 3 + bits.bits(3)?),
                        _ => (0, 11 + bits.bits(7)?),
                    };
                    for _ in 0..times {
                        lengths.push(value);
                    }
                }
                if lengths.len() > hlit + hdist {
                    return Err("code lengths run past their count".to_string());
                }
                block(
                    &mut bits,
                    &mut out,
                    &Huffman::new(&lengths[..hlit]),
                    &Huffman::new(&lengths[hlit..]),
                )?;
            }
            _ => return Err("a deflate block of the reserved type 3".to_string()),
        }
        if last == 1 {
            return Ok(out);
        }
    }
}

fn block(bits: &mut Bits, out: &mut Vec<u8>, lit: &Huffman, dist: &Huffman) -> Result<(), String> {
    loop {
        let sym = lit.decode(bits)? as usize;
        match sym {
            0..=255 => out.push(sym as u8),
            256 => return Ok(()),
            257..=285 => {
                let i = sym - 257;
                let len =
                    usize::from(LENGTH_BASE[i]) + bits.bits(u32::from(LENGTH_EXTRA[i]))? as usize;
                let d = dist.decode(bits)? as usize;
                if d >= 30 {
                    return Err("a deflate distance code above 29".to_string());
                }
                let back =
                    usize::from(DIST_BASE[d]) + bits.bits(u32::from(DIST_EXTRA[d]))? as usize;
                if back > out.len() {
                    return Err("a deflate distance before the start of the stream".to_string());
                }
                let from = out.len() - back;
                for k in 0..len {
                    out.push(out[from + k]);
                }
            }
            _ => return Err("a deflate length code above 285".to_string()),
        }
    }
}

// ---------------------------------------------------------------- deflate (fixed codes) and PNG

struct Writer {
    out: Vec<u8>,
    acc: u32,
    n: u32,
}

impl Writer {
    /// Writes the low `n` bits of `v`, least significant first.
    fn put(&mut self, v: u32, n: u32) {
        self.acc |= v << self.n;
        self.n += n;
        while self.n >= 8 {
            self.out.push(self.acc as u8);
            self.acc >>= 8;
            self.n -= 8;
        }
    }

    /// Writes a Huffman code of `len` bits, most significant first.
    fn code(&mut self, code: u32, len: u32) {
        let mut rev = 0;
        for i in 0..len {
            rev |= ((code >> i) & 1) << (len - 1 - i);
        }
        self.put(rev, len);
    }

    fn literal(&mut self, sym: u32) {
        match sym {
            0..=143 => self.code(0x30 + sym, 8),
            144..=255 => self.code(0x190 + sym - 144, 9),
            256..=279 => self.code(sym - 256, 7),
            _ => self.code(0xc0 + sym - 280, 8),
        }
    }

    fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.out.push(self.acc as u8);
        }
        self.out
    }
}

/// Deflates `data` in one block with the fixed Huffman codes; matches of 3 to 258 bytes up to
/// 32 KiB back, found through the last position of each 3-byte prefix.
pub fn deflate(data: &[u8]) -> Vec<u8> {
    let mut w = Writer {
        out: Vec::new(),
        acc: 0,
        n: 0,
    };
    w.put(1, 1);
    w.put(1, 2);
    let mut head = vec![usize::MAX; 1 << 15];
    let hash = |i: usize| {
        ((usize::from(data[i]) << 10) ^ (usize::from(data[i + 1]) << 5) ^ usize::from(data[i + 2]))
            & 0x7fff
    };
    let mut i = 0;
    while i < data.len() {
        let mut best = (0, 0);
        if i + 3 <= data.len() {
            let h = hash(i);
            let cand = head[h];
            head[h] = i;
            // The byte before is the cheapest match of a run; else the prefix's last position.
            for c in [i.wrapping_sub(1), cand] {
                if c < i && i - c <= 32_768 {
                    let max = (data.len() - i).min(258);
                    let mut len = 0;
                    while len < max && data[c + len] == data[i + len] {
                        len += 1;
                    }
                    if len >= 3 && len > best.0 {
                        best = (len, i - c);
                    }
                }
            }
        }
        if best.0 == 0 {
            w.literal(u32::from(data[i]));
            i += 1;
            continue;
        }
        let (len, back) = best;
        let li = LENGTH_BASE
            .iter()
            .rposition(|&b| usize::from(b) <= len)
            .unwrap();
        w.literal(257 + li as u32);
        w.put(
            (len - usize::from(LENGTH_BASE[li])) as u32,
            u32::from(LENGTH_EXTRA[li]),
        );
        let di = DIST_BASE
            .iter()
            .rposition(|&b| usize::from(b) <= back)
            .unwrap();
        w.code(di as u32, 5);
        w.put(
            (back - usize::from(DIST_BASE[di])) as u32,
            u32::from(DIST_EXTRA[di]),
        );
        for k in i + 1..(i + len).min(data.len().saturating_sub(2)) {
            head[hash(k)] = k;
        }
        i += len;
    }
    w.literal(256);
    w.finish()
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for chunk in bytes.chunks(5552) {
        for &x in chunk {
            a += u32::from(x);
            b += a;
        }
        a %= 65_521;
        b %= 65_521;
    }
    (b << 16) | a
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(body);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// Writes `image` as an 8-bit RGBA PNG; the same pixels give the same bytes.
pub fn encode(image: &Image) -> Vec<u8> {
    let stride = image.width as usize * 4;
    let mut raw = Vec::with_capacity((stride + 1) * image.height as usize);
    for y in 0..image.height as usize {
        let row = &image.rgba[y * stride..(y + 1) * stride];
        if y == 0 {
            raw.push(0);
            raw.extend_from_slice(row);
        } else {
            let prior = &image.rgba[(y - 1) * stride..y * stride];
            raw.push(2);
            raw.extend(row.iter().zip(prior).map(|(a, b)| a.wrapping_sub(*b)));
        }
    }
    let mut z = vec![0x78, 0x01];
    z.extend(deflate(&raw));
    z.extend_from_slice(&adler32(&raw).to_be_bytes());
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&image.width.to_be_bytes());
    ihdr.extend_from_slice(&image.height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    let mut out = SIGNATURE.to_vec();
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    out
}
