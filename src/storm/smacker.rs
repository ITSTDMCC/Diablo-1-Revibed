//! Smacker (`.smk`) video decoder: the role of the `SmackerDecoder` library (libsmackerdec)
//! that DevilutionX's `storm_svid.cpp` uses. Its source is not in the DevilutionX 1.5.3 tree, so
//! this is written from the published Smacker format description (header, Huffman-coded
//! "big trees" with a 3-entry recent-value cache, block-coded video, delta palette, Huffman DPCM
//! audio). The functions mirror the `Smacker_*` API that `storm_svid.cpp` calls.

/// LSB-first bit reader over a byte slice (reads past the end return 0 bits).
struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BitReader { data, pos: 0 }
    }

    fn bit(&mut self) -> u32 {
        let byte = self.data.get(self.pos >> 3).copied().unwrap_or(0);
        let b = (byte >> (self.pos & 7)) & 1;
        self.pos += 1;
        b as u32
    }

    fn bits(&mut self, n: u32) -> u32 {
        let mut v = 0;
        for i in 0..n {
            v |= self.bit() << i;
        }
        v
    }
}

/// An 8-bit Huffman tree: `nodes[i]` is a leaf value or the index of the "1" child (the "0"
/// child follows the node directly).
#[derive(Clone, Debug, Default)]
struct Tree8 {
    nodes: Vec<Node8>,
}

#[derive(Clone, Copy, Debug)]
enum Node8 {
    Leaf(u8),
    Branch(usize),
}

impl Tree8 {
    fn read(br: &mut BitReader) -> Result<Tree8, String> {
        let mut t = Tree8::default();
        t.read_node(br, 0)?;
        Ok(t)
    }

    fn read_node(&mut self, br: &mut BitReader, depth: u32) -> Result<(), String> {
        if depth > 32 {
            return Err("Smacker: Huffman tree too deep".into());
        }
        if br.bit() == 0 {
            self.nodes.push(Node8::Leaf(br.bits(8) as u8));
            return Ok(());
        }
        let idx = self.nodes.len();
        self.nodes.push(Node8::Branch(0));
        self.read_node(br, depth + 1)?;
        let one = self.nodes.len();
        self.nodes[idx] = Node8::Branch(one);
        self.read_node(br, depth + 1)
    }

    fn decode(&self, br: &mut BitReader) -> u8 {
        let mut i = 0;
        loop {
            match self.nodes[i] {
                Node8::Leaf(v) => return v,
                Node8::Branch(one) => i = if br.bit() == 1 { one } else { i + 1 },
            }
        }
    }
}

const SMK_NODE: u32 = 0x8000_0000;

/// A 16-bit "big" Huffman tree with the three escape-code cache slots (`last`).
#[derive(Clone, Debug)]
struct BigTree {
    /// Leaves hold 16-bit values; nodes hold `SMK_NODE | size of the "0" subtree`.
    values: Vec<u32>,
    last: [usize; 3],
}

impl BigTree {
    /// A tree that is absent from the file: always decodes 0 without reading bits.
    fn empty() -> BigTree {
        BigTree { values: vec![0, 0], last: [1, 1, 1] }
    }

    fn read(br: &mut BitReader) -> Result<BigTree, String> {
        if br.bit() == 0 {
            return Ok(BigTree::empty());
        }
        let low = if br.bit() == 1 {
            let t = Tree8::read(br)?;
            br.bit();
            Some(t)
        } else {
            None
        };
        let high = if br.bit() == 1 {
            let t = Tree8::read(br)?;
            br.bit();
            Some(t)
        } else {
            None
        };
        let escapes = [br.bits(16), br.bits(16), br.bits(16)];
        let mut tree = BigTree { values: Vec::new(), last: [usize::MAX; 3] };
        tree.read_node(br, &low, &high, &escapes, 0)?;
        br.bit();
        for l in 0..3 {
            if tree.last[l] == usize::MAX {
                tree.last[l] = tree.values.len();
                tree.values.push(0);
            }
        }
        Ok(tree)
    }

    fn read_node(&mut self, br: &mut BitReader, low: &Option<Tree8>, high: &Option<Tree8>, escapes: &[u32; 3], depth: u32) -> Result<u32, String> {
        if depth > 64 {
            return Err("Smacker: big tree too deep".into());
        }
        if br.bit() == 0 {
            let lo = low.as_ref().map_or(0, |t| t.decode(br)) as u32;
            let hi = high.as_ref().map_or(0, |t| t.decode(br)) as u32;
            let mut val = lo | (hi << 8);
            if let Some(k) = escapes.iter().position(|&e| e == val) {
                self.last[k] = self.values.len();
                val = 0;
            }
            self.values.push(val);
            return Ok(1);
        }
        let t = self.values.len();
        self.values.push(0);
        let r = self.read_node(br, low, high, escapes, depth + 1)?;
        self.values[t] = SMK_NODE | r;
        let r_new = self.read_node(br, low, high, escapes, depth + 1)?;
        Ok(r + 1 + r_new)
    }

    /// Clears the cache slots (done at the start of every frame).
    fn reset(&mut self) {
        for l in self.last {
            self.values[l] = 0;
        }
    }

    fn decode(&mut self, br: &mut BitReader) -> u32 {
        let mut i = 0;
        while self.values[i] & SMK_NODE != 0 {
            if br.bit() == 1 {
                i += (self.values[i] & !SMK_NODE) as usize;
            }
            i += 1;
        }
        let v = self.values[i];
        if v != self.values[self.last[0]] {
            self.values[self.last[2]] = self.values[self.last[1]];
            self.values[self.last[1]] = self.values[self.last[0]];
            self.values[self.last[0]] = v;
        }
        v
    }
}

/// Run lengths for the block-type codes.
const BLOCK_RUNS: [usize; 64] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43,
    44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 128, 256, 512, 1024, 2048,
];

const FLAG_RING_FRAME: u32 = 1;
const FLAG_Y_INTERLACE: u32 = 2;
const FLAG_Y_DOUBLE: u32 = 4;

/// `Smacker_GetAudioTrackDetails` result.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioInfo {
    pub sample_rate: u32,
    pub n_channels: u32,
    pub bits_per_sample: u32,
    pub ideal_buffer_size: u32,
}

#[derive(Clone, Debug, Default)]
struct AudioTrack {
    rate_flags: u32,
    buffer_size: u32,
    /// Decoded bytes of the current frame (16-bit samples little-endian, or unsigned 8-bit).
    data: Vec<u8>,
}

impl AudioTrack {
    fn present(&self) -> bool {
        self.rate_flags & 0x4000_0000 != 0
    }
    fn compressed(&self) -> bool {
        self.rate_flags & 0x8000_0000 != 0
    }
    fn is_16bit(&self) -> bool {
        self.rate_flags & 0x2000_0000 != 0
    }
    fn stereo(&self) -> bool {
        self.rate_flags & 0x1000_0000 != 0
    }
}

/// `SmackerHandle`: an open video with its decoding state.
pub struct Smacker {
    data: Vec<u8>,
    smk4: bool,
    width: u32,
    height: u32,
    num_frames: u32,
    frame_rate: f32,
    flags: u32,
    frame_offsets: Vec<usize>,
    frame_sizes: Vec<usize>,
    frame_types: Vec<u8>,
    mmap: BigTree,
    mclr: BigTree,
    full: BigTree,
    typ: BigTree,
    audio: [AudioTrack; 7],
    current_frame: u32,
    frame: Vec<u8>,
    palette: [u8; 768],
    palette_changed: bool,
}

fn le32(d: &[u8], p: usize) -> Result<u32, String> {
    d.get(p..p + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).ok_or_else(|| "Smacker: file truncated".to_string())
}

impl Smacker {
    /// `Smacker_Open`
    pub fn open(data: Vec<u8>) -> Result<Smacker, String> {
        let sig = data.get(0..4).ok_or("Smacker: file truncated")?;
        let smk4 = match sig {
            b"SMK2" => false,
            b"SMK4" => true,
            _ => return Err("Smacker: not a Smacker file".into()),
        };
        let width = le32(&data, 4)?;
        let height = le32(&data, 8)?;
        let num_frames = le32(&data, 12)?;
        let rate = le32(&data, 16)? as i32;
        let frame_rate = if rate > 0 {
            1000.0 / rate as f32
        } else if rate < 0 {
            100000.0 / (-rate) as f32
        } else {
            10.0
        };
        let flags = le32(&data, 20)?;
        let mut audio: [AudioTrack; 7] = Default::default();
        for (i, t) in audio.iter_mut().enumerate() {
            t.buffer_size = le32(&data, 24 + 4 * i)?;
            t.rate_flags = le32(&data, 72 + 4 * i)?;
        }
        let trees_size = le32(&data, 52)? as usize;
        let mut p = 104;
        let n = num_frames as usize + if flags & FLAG_RING_FRAME != 0 { 1 } else { 0 };
        let mut frame_sizes = Vec::with_capacity(n);
        for _ in 0..n {
            frame_sizes.push((le32(&data, p)? & !3) as usize);
            p += 4;
        }
        let frame_types = data.get(p..p + n).ok_or("Smacker: file truncated")?.to_vec();
        p += n;
        let trees = data.get(p..p + trees_size).ok_or("Smacker: file truncated")?;
        let mut br = BitReader::new(trees);
        let mmap = BigTree::read(&mut br)?;
        let mclr = BigTree::read(&mut br)?;
        let full = BigTree::read(&mut br)?;
        let typ = BigTree::read(&mut br)?;
        p += trees_size;
        let mut frame_offsets = Vec::with_capacity(n);
        for s in &frame_sizes {
            frame_offsets.push(p);
            p += s;
        }
        Ok(Smacker {
            data,
            smk4,
            width,
            height,
            num_frames,
            frame_rate,
            flags,
            frame_offsets,
            frame_sizes,
            frame_types,
            mmap,
            mclr,
            full,
            typ,
            audio,
            current_frame: 0,
            frame: vec![0; (width * height) as usize],
            palette: [0; 768],
            palette_changed: false,
        })
    }

    /// `Smacker_GetNumFrames`
    pub fn num_frames(&self) -> u32 {
        self.num_frames
    }

    /// `Smacker_GetCurrentFrameNum`
    pub fn current_frame_num(&self) -> u32 {
        self.current_frame
    }

    /// `Smacker_GetFrameRate` (frames per second)
    pub fn frame_rate(&self) -> f32 {
        self.frame_rate
    }

    /// `Smacker_GetFrameSize`: the output size (doubled height for Y-interlaced/Y-doubled files).
    pub fn frame_size(&self) -> (u32, u32) {
        let h = if self.flags & (FLAG_Y_INTERLACE | FLAG_Y_DOUBLE) != 0 { self.height * 2 } else { self.height };
        (self.width, h)
    }

    /// `Smacker_GetAudioTrackDetails`
    pub fn audio_track_details(&self, track: usize) -> AudioInfo {
        let t = &self.audio[track];
        if !t.present() {
            return AudioInfo::default();
        }
        AudioInfo {
            sample_rate: t.rate_flags & 0x00FF_FFFF,
            n_channels: if t.stereo() { 2 } else { 1 },
            bits_per_sample: if t.is_16bit() { 16 } else { 8 },
            ideal_buffer_size: t.buffer_size,
        }
    }

    /// `Smacker_GetAudioData`: the bytes decoded for `track` with the current frame.
    pub fn audio_data(&self, track: usize) -> &[u8] {
        &self.audio[track].data
    }

    /// `Smacker_DidPaletteChange`
    pub fn did_palette_change(&self) -> bool {
        self.palette_changed
    }

    /// `Smacker_GetPalette`: 256 RGB triplets.
    pub fn palette(&self) -> &[u8; 768] {
        &self.palette
    }

    /// `Smacker_GetFrame`: the current frame, `frame_size()` pixels.
    pub fn get_frame(&self, out: &mut [u8]) {
        let w = self.width as usize;
        if self.flags & FLAG_Y_DOUBLE != 0 {
            for (y, row) in self.frame.chunks_exact(w).enumerate() {
                out[2 * y * w..(2 * y + 1) * w].copy_from_slice(row);
                out[(2 * y + 1) * w..(2 * y + 2) * w].copy_from_slice(row);
            }
        } else if self.flags & FLAG_Y_INTERLACE != 0 {
            for (y, row) in self.frame.chunks_exact(w).enumerate() {
                out[2 * y * w..(2 * y + 1) * w].copy_from_slice(row);
                out[(2 * y + 1) * w..(2 * y + 2) * w].fill(0);
            }
        } else {
            out[..self.frame.len()].copy_from_slice(&self.frame);
        }
    }

    /// `Smacker_Rewind`
    pub fn rewind(&mut self) {
        self.current_frame = 0;
    }

    /// `Smacker_GetNextFrame`: decodes the palette, audio and video of the next frame.
    pub fn get_next_frame(&mut self) -> Result<(), String> {
        let f = self.current_frame as usize;
        if f >= self.frame_offsets.len() {
            return Err("Smacker: no more frames".into());
        }
        let start = self.frame_offsets[f];
        let size = self.frame_sizes[f];
        let ftype = self.frame_types[f];
        let data = std::mem::take(&mut self.data);
        let result = self.decode_frame(data.get(start..start + size).ok_or("Smacker: frame past end of file"), ftype);
        self.data = data;
        self.current_frame += 1;
        result
    }

    fn decode_frame(&mut self, chunk: Result<&[u8], &str>, ftype: u8) -> Result<(), String> {
        let chunk = chunk.map_err(|e| e.to_string())?;
        let mut p = 0;
        self.palette_changed = false;
        if ftype & 1 != 0 {
            let len = *chunk.first().ok_or("Smacker: empty frame")? as usize * 4;
            self.decode_palette(chunk.get(1..len).ok_or("Smacker: palette past end of frame")?);
            self.palette_changed = true;
            p = len;
        }
        for track in 0..7 {
            self.audio[track].data.clear();
            if ftype & (2 << track) == 0 {
                continue;
            }
            let len = le32(chunk, p)? as usize;
            let body = chunk.get(p + 4..p + len).ok_or("Smacker: audio past end of frame")?;
            self.decode_audio(track, body)?;
            p += len;
        }
        self.decode_video(chunk.get(p..).unwrap_or(&[]));
        Ok(())
    }

    fn decode_palette(&mut self, d: &[u8]) {
        let old = self.palette;
        let pal6 = |v: u8| (v << 2) | (v >> 4);
        let mut sz = 0usize;
        let mut i = 0usize;
        let mut byte = || {
            let b = d.get(i).copied().unwrap_or(0);
            i += 1;
            b
        };
        while sz < 256 {
            let t = byte();
            if t & 0x80 != 0 {
                // skip (keep) entries
                sz += (t & 0x7F) as usize + 1;
            } else if t & 0x40 != 0 {
                // copy entries from the previous palette
                let mut off = byte() as usize;
                let mut j = (t & 0x3F) as usize + 1;
                while j > 0 && sz < 256 && off < 256 {
                    self.palette[sz * 3..sz * 3 + 3].copy_from_slice(&old[off * 3..off * 3 + 3]);
                    sz += 1;
                    off += 1;
                    j -= 1;
                }
            } else {
                // new entry
                let g = byte();
                let b = byte();
                self.palette[sz * 3] = pal6(t & 0x3F);
                self.palette[sz * 3 + 1] = pal6(g & 0x3F);
                self.palette[sz * 3 + 2] = pal6(b & 0x3F);
                sz += 1;
            }
        }
    }

    fn decode_audio(&mut self, track: usize, body: &[u8]) -> Result<(), String> {
        let t = &mut self.audio[track];
        if !t.compressed() {
            t.data.extend_from_slice(body);
            return Ok(());
        }
        let unp_size = le32(body, 0)? as usize;
        let mut br = BitReader::new(&body[4..]);
        if br.bit() == 0 {
            return Ok(());
        }
        let stereo = br.bit() as usize;
        let bits16 = br.bit() as usize;
        let mut trees = Vec::new();
        for _ in 0..(1 << (bits16 + stereo)) {
            br.bit();
            trees.push(Tree8::read(&mut br)?);
            br.bit();
        }
        let out = &mut t.data;
        out.reserve(unp_size);
        if bits16 == 1 {
            let mut pred = [0i16; 2];
            for i in (0..=stereo).rev() {
                let hi = br.bits(8);
                let lo = br.bits(8);
                pred[i] = ((hi << 8) | lo) as u16 as i16;
            }
            for &v in &pred[..=stereo] {
                out.extend_from_slice(&v.to_le_bytes());
            }
            for i in (stereo + 1)..(unp_size / 2) {
                let c = i & stereo;
                let lo = trees[2 * c].decode(&mut br) as u16;
                let hi = trees[2 * c + 1].decode(&mut br) as u16;
                pred[c] = pred[c].wrapping_add((lo | (hi << 8)) as i16);
                out.extend_from_slice(&pred[c].to_le_bytes());
            }
        } else {
            let mut pred = [0u8; 2];
            for i in (0..=stereo).rev() {
                pred[i] = br.bits(8) as u8;
            }
            out.extend_from_slice(&pred[..=stereo]);
            for i in (stereo + 1)..unp_size {
                let c = i & stereo;
                let d = trees[c].decode(&mut br);
                pred[c] = pred[c].wrapping_add(d);
                out.push(pred[c]);
            }
        }
        Ok(())
    }

    fn decode_video(&mut self, d: &[u8]) {
        self.mmap.reset();
        self.mclr.reset();
        self.full.reset();
        self.typ.reset();
        let mut br = BitReader::new(d);
        let stride = self.width as usize;
        let bw = stride / 4;
        let bh = self.height as usize / 4;
        let blocks = bw * bh;
        let frame = &mut self.frame;
        let at = |blk: usize| (blk / bw) * stride * 4 + (blk % bw) * 4;
        let mut blk = 0;
        while blk < blocks {
            let ty = self.typ.decode(&mut br);
            let mut run = BLOCK_RUNS[((ty >> 2) & 0x3F) as usize];
            match ty & 3 {
                0 => {
                    // MONO: two colours and a 16-bit pattern
                    while run > 0 && blk < blocks {
                        let clr = self.mclr.decode(&mut br);
                        let mut map = self.mmap.decode(&mut br);
                        let (hi, lo) = ((clr >> 8) as u8, clr as u8);
                        let mut o = at(blk);
                        for _ in 0..4 {
                            for x in 0..4 {
                                frame[o + x] = if map & (1 << x) != 0 { hi } else { lo };
                            }
                            map >>= 4;
                            o += stride;
                        }
                        blk += 1;
                        run -= 1;
                    }
                }
                1 => {
                    // FULL
                    let mut mode = 0;
                    if self.smk4 {
                        if br.bit() == 1 {
                            mode = 1;
                        } else if br.bit() == 1 {
                            mode = 2;
                        }
                    }
                    while run > 0 && blk < blocks {
                        let mut o = at(blk);
                        let mut put2 = |o: usize, pix: u32| {
                            frame[o] = pix as u8;
                            frame[o + 1] = (pix >> 8) as u8;
                        };
                        match mode {
                            0 => {
                                for _ in 0..4 {
                                    let p1 = self.full.decode(&mut br);
                                    put2(o + 2, p1);
                                    let p2 = self.full.decode(&mut br);
                                    put2(o, p2);
                                    o += stride;
                                }
                            }
                            1 => {
                                for _ in 0..2 {
                                    let pix = self.full.decode(&mut br);
                                    for _ in 0..2 {
                                        put2(o, (pix & 0xFF) * 0x101);
                                        put2(o + 2, (pix >> 8) * 0x101);
                                        o += stride;
                                    }
                                }
                            }
                            _ => {
                                for _ in 0..2 {
                                    let pix2 = self.full.decode(&mut br);
                                    let pix1 = self.full.decode(&mut br);
                                    for _ in 0..2 {
                                        put2(o, pix1);
                                        put2(o + 2, pix2);
                                        o += stride;
                                    }
                                }
                            }
                        }
                        blk += 1;
                        run -= 1;
                    }
                }
                2 => {
                    // VOID: keep the previous frame's pixels
                    blk = (blk + run).min(blocks);
                }
                _ => {
                    // SOLID
                    let col = (ty >> 8) as u8;
                    while run > 0 && blk < blocks {
                        let mut o = at(blk);
                        for _ in 0..4 {
                            frame[o..o + 4].fill(col);
                            o += stride;
                        }
                        blk += 1;
                        run -= 1;
                    }
                }
            }
        }
    }
}
