//! `Source/mpq/mpq_writer.cpp`: creating and editing the MPQ archives used for save files.
//!
//! Layout as in the original: a 104-byte header, then the 2048-entry block table and the
//! 2048-entry hash table (both encrypted), then file data. Files are stored in 4096-byte sectors,
//! each PKWARE-imploded when that makes it smaller.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::encrypt::{decrypt, encrypt, hash, pkware_compress};
use crate::platform::log;

pub const MaxMpqPathSize: usize = 256;

const HashEntriesCount: u32 = 2048;
const BlockEntriesCount: u32 = 2048;
const BlockEntrySize: u32 = HashEntriesCount * 16;
const HashEntrySize: u32 = BlockEntriesCount * 16;
/// `sizeof(MpqFileHeader)` (32 bytes of fields and 72 of padding)
const MpqFileHeaderSize: u32 = 104;
const MpqBlockEntryOffset: u32 = MpqFileHeaderSize;
const MpqHashEntryOffset: u32 = MpqBlockEntryOffset + BlockEntrySize;
const HashEntryNotFound: u32 = u32::MAX;
const BlockSizeFactor: u16 = 3;
const BlockSize: u32 = 512 << BlockSizeFactor;
const MinBlockSize: u32 = 1024;

const DiabloSignature: u32 = u32::from_le_bytes(*b"MPQ\x1A");
const DiabloSize: u32 = 32;
const NullBlock: u32 = u32::MAX;
const DeletedBlock: u32 = u32::MAX - 1;
const FlagExists: u32 = 0x8000_0000;
const CompressPkZip: u32 = 0x0000_0100;

/// `MpqFileHeader` (the fields; the 72 padding bytes are written as zeros)
#[derive(Clone, Copy, Debug, Default)]
struct MpqFileHeader {
    signature: u32,
    header_size: u32,
    file_size: u32,
    version: u16,
    block_size_factor: u16,
    hash_entries_offset: u32,
    block_entries_offset: u32,
    hash_entries_count: u32,
    block_entries_count: u32,
}

impl MpqFileHeader {
    fn to_bytes(self) -> [u8; MpqFileHeaderSize as usize] {
        let mut b = [0u8; MpqFileHeaderSize as usize];
        b[0..4].copy_from_slice(&self.signature.to_le_bytes());
        b[4..8].copy_from_slice(&self.header_size.to_le_bytes());
        b[8..12].copy_from_slice(&self.file_size.to_le_bytes());
        b[12..14].copy_from_slice(&self.version.to_le_bytes());
        b[14..16].copy_from_slice(&self.block_size_factor.to_le_bytes());
        b[16..20].copy_from_slice(&self.hash_entries_offset.to_le_bytes());
        b[20..24].copy_from_slice(&self.block_entries_offset.to_le_bytes());
        b[24..28].copy_from_slice(&self.hash_entries_count.to_le_bytes());
        b[28..32].copy_from_slice(&self.block_entries_count.to_le_bytes());
        b
    }

    fn from_bytes(b: &[u8]) -> MpqFileHeader {
        let u32_at = |o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
        let u16_at = |o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
        MpqFileHeader {
            signature: u32_at(0),
            header_size: u32_at(4),
            file_size: u32_at(8),
            version: u16_at(12),
            block_size_factor: u16_at(14),
            hash_entries_offset: u32_at(16),
            block_entries_offset: u32_at(20),
            hash_entries_count: u32_at(24),
            block_entries_count: u32_at(28),
        }
    }
}

/// `MpqHashEntry`
#[derive(Clone, Copy, Debug)]
struct MpqHashEntry {
    hash_a: u32,
    hash_b: u32,
    locale: u16,
    platform: u16,
    block: u32,
}

/// `MpqBlockEntry`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MpqBlockEntry {
    offset: u32,
    packed_size: u32,
    unpacked_size: u32,
    flags: u32,
}

/// Original: `IsAllocatedUnusedBlock` (mpq_writer.cpp).
// @port mpq/mpq_writer.cpp|devilution::IsAllocatedUnusedBlock(const MpqBlockEntry *block) sha=bef67dc53d13
fn is_allocated_unused_block(block: &MpqBlockEntry) -> bool {
    block.offset != 0 && block.flags == 0 && block.unpacked_size == 0
}

/// Original: `IsUnallocatedBlock` (mpq_writer.cpp).
// @port mpq/mpq_writer.cpp|devilution::IsUnallocatedBlock(const MpqBlockEntry *block) sha=f32fa8d61cdc
fn is_unallocated_block(block: &MpqBlockEntry) -> bool {
    block.offset == 0 && block.packed_size == 0 && block.unpacked_size == 0 && block.flags == 0
}

fn hash_table_bytes(t: &[MpqHashEntry]) -> Vec<u8> {
    let mut out = Vec::with_capacity(t.len() * 16);
    for e in t {
        out.extend_from_slice(&e.hash_a.to_le_bytes());
        out.extend_from_slice(&e.hash_b.to_le_bytes());
        out.extend_from_slice(&e.locale.to_le_bytes());
        out.extend_from_slice(&e.platform.to_le_bytes());
        out.extend_from_slice(&e.block.to_le_bytes());
    }
    out
}

fn block_table_bytes(t: &[MpqBlockEntry]) -> Vec<u8> {
    let mut out = Vec::with_capacity(t.len() * 16);
    for e in t {
        out.extend_from_slice(&e.offset.to_le_bytes());
        out.extend_from_slice(&e.packed_size.to_le_bytes());
        out.extend_from_slice(&e.unpacked_size.to_le_bytes());
        out.extend_from_slice(&e.flags.to_le_bytes());
    }
    out
}

fn u32s(b: &[u8]) -> Vec<u32> {
    b.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
}

/// `MpqWriter` (also `SaveWriter`: saves are packed MPQs in the Windows build)
pub struct MpqWriter {
    stream: Option<File>,
    name: String,
    size: u64,
    hash_table: Vec<MpqHashEntry>,
    block_table: Vec<MpqBlockEntry>,
}

pub type SaveWriter = MpqWriter;

impl MpqWriter {
    /// Original: `MpqWriter::MpqWriter(const char *path)` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::MpqWriter(const char *path) sha=4b135c651e7a
    pub fn new(ctx: &mut crate::ctx::Ctx, path: &str) -> MpqWriter {
        match MpqWriter::open(path) {
            Ok(w) => w,
            Err(error) => crate::appfat::app_fatal(ctx, &format!("{}\n{}\n{}", crate::utils::language::tr("Failed to open archive for writing."), path, error)),
        }
    }

    fn open(path: &str) -> Result<MpqWriter, String> {
        if let Some(dir) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        log::verbose!("Opening {}", path);
        let exists = std::path::Path::new(path).exists();
        let mut w = MpqWriter { stream: None, name: path.to_string(), size: 0, hash_table: Vec::new(), block_table: Vec::new() };
        let mut file = if exists {
            w.size = std::fs::metadata(path).map_err(|e| format!("GetFileSize failed: \"{path}\" {e}"))?.len();
            log::verbose!("GetFileSize(\"{}\") = {}", path, w.size);
            OpenOptions::new().read(true).write(true).open(path).map_err(|_| "Failed to open file".to_string())?
        } else {
            File::create(path).map_err(|_| "Failed to open file".to_string())?
        };

        let fhdr = if !exists { w.init_default_mpq_header() } else { w.read_mpq_header(&mut file).ok_or("Failed to read MPQ header")? };
        w.block_table = vec![MpqBlockEntry::default(); BlockEntriesCount as usize];
        if fhdr.block_entries_count > 0 {
            let mut buf = vec![0u8; fhdr.block_entries_count as usize * 16];
            file.read_exact(&mut buf).map_err(|_| "Failed to read block table")?;
            decrypt(&mut buf, hash(b"(block table)", 3));
            for (i, e) in u32s(&buf).chunks_exact(4).enumerate().take(BlockEntriesCount as usize) {
                w.block_table[i] = MpqBlockEntry { offset: e[0], packed_size: e[1], unpacked_size: e[2], flags: e[3] };
            }
        }
        w.hash_table = vec![MpqHashEntry { hash_a: u32::MAX, hash_b: u32::MAX, locale: 0xFFFF, platform: 0xFFFF, block: NullBlock }; HashEntriesCount as usize];
        if fhdr.hash_entries_count > 0 {
            let mut buf = vec![0u8; fhdr.hash_entries_count as usize * 16];
            file.read_exact(&mut buf).map_err(|_| "Failed to read hash entries")?;
            decrypt(&mut buf, hash(b"(hash table)", 3));
            for (i, e) in u32s(&buf).chunks_exact(4).enumerate().take(HashEntriesCount as usize) {
                w.hash_table[i] = MpqHashEntry { hash_a: e[0], hash_b: e[1], locale: e[2] as u16, platform: (e[2] >> 16) as u16, block: e[3] };
            }
        }
        w.stream = Some(file);
        Ok(w)
    }

    /// Original: `MpqWriter::FetchHandle` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::FetchHandle(const char *filename) sha=ba3ff8372394
    fn fetch_handle(&self, filename: &str) -> u32 {
        let f = filename.as_bytes();
        self.get_hash_index(hash(f, 0), hash(f, 1), hash(f, 2))
    }

    /// Original: `MpqWriter::InitDefaultMpqHeader` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::InitDefaultMpqHeader(MpqFileHeader *hdr) sha=49ebb8fe9b58
    fn init_default_mpq_header(&mut self) -> MpqFileHeader {
        self.size = (MpqHashEntryOffset + HashEntrySize) as u64;
        MpqFileHeader { signature: DiabloSignature, header_size: DiabloSize, block_size_factor: BlockSizeFactor, version: 0, ..Default::default() }
    }

    /// Original: `MpqWriter::IsValidMpqHeader` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::IsValidMpqHeader(MpqFileHeader *hdr) sha=394cffedf9bc
    fn is_valid_mpq_header(&self, hdr: &MpqFileHeader) -> bool {
        hdr.signature == DiabloSignature
            && hdr.header_size == DiabloSize
            && hdr.version == 0
            && hdr.block_size_factor == BlockSizeFactor
            && hdr.file_size as u64 == self.size
            && hdr.hash_entries_offset == MpqHashEntryOffset
            && hdr.block_entries_offset == MpqFileHeaderSize
            && hdr.hash_entries_count == HashEntriesCount
            && hdr.block_entries_count == BlockEntriesCount
    }

    /// Original: `MpqWriter::ReadMPQHeader` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::ReadMPQHeader(MpqFileHeader *hdr) sha=5f337da33205
    fn read_mpq_header(&mut self, file: &mut File) -> Option<MpqFileHeader> {
        let has_hdr = self.size >= MpqFileHeaderSize as u64;
        let mut hdr = MpqFileHeader::default();
        if has_hdr {
            let mut b = [0u8; MpqFileHeaderSize as usize];
            file.read_exact(&mut b).ok()?;
            hdr = MpqFileHeader::from_bytes(&b);
        }
        if !has_hdr || !self.is_valid_mpq_header(&hdr) {
            hdr = self.init_default_mpq_header();
        }
        Some(hdr)
    }

    /// Original: `MpqWriter::NewBlock` (mpq_writer.cpp). Returns the block index.
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::NewBlock(uint32_t *blockIndex) sha=ac4680a9f4d0
    fn new_block(&mut self) -> usize {
        for (i, block) in self.block_table.iter().enumerate() {
            if is_unallocated_block(block) {
                return i;
            }
        }
        panic!("Out of free block entries");
    }

    /// Original: `MpqWriter::AllocBlock` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::AllocBlock(uint32_t blockOffset, uint32_t blockSize) sha=09f262c464b5
    fn alloc_block(&mut self, mut block_offset: u32, mut block_size: u32) {
        loop {
            let mut expand = false;
            for block in self.block_table.iter_mut() {
                if !is_allocated_unused_block(block) {
                    continue;
                }
                if block.offset.wrapping_add(block.packed_size) == block_offset {
                    block_offset = block.offset;
                    block_size = block_size.wrapping_add(block.packed_size);
                    *block = MpqBlockEntry::default();
                    expand = true;
                    break;
                }
                if block_offset.wrapping_add(block_size) == block.offset {
                    block_size = block_size.wrapping_add(block.packed_size);
                    *block = MpqBlockEntry::default();
                    expand = true;
                    break;
                }
            }
            if !expand {
                break;
            }
        }
        if block_offset as u64 + block_size as u64 > self.size {
            panic!("MPQ free list error");
        }
        if block_offset as u64 + block_size as u64 == self.size {
            self.size = block_offset as u64;
        } else {
            let i = self.new_block();
            self.block_table[i] = MpqBlockEntry { offset: block_offset, packed_size: block_size, unpacked_size: 0, flags: 0 };
        }
    }

    /// Original: `MpqWriter::FindFreeBlock` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::FindFreeBlock(uint32_t size) sha=15b67ab29f62
    fn find_free_block(&mut self, size: u32) -> u32 {
        for block in self.block_table.iter_mut() {
            if !is_allocated_unused_block(block) || block.packed_size < size {
                continue;
            }
            let result = block.offset;
            block.offset += size;
            block.packed_size -= size;
            if block.packed_size == 0 {
                *block = MpqBlockEntry::default();
            }
            return result;
        }
        let result = self.size as u32;
        self.size += size as u64;
        result
    }

    /// Original: `MpqWriter::GetHashIndex` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::GetHashIndex(uint32_t index, uint32_t hashA, uint32_t hashB) sha=13164a30d101
    fn get_hash_index(&self, index: u32, hash_a: u32, hash_b: u32) -> u32 {
        let mut i = HashEntriesCount;
        let mut idx = index & 0x7FF;
        while self.hash_table[idx as usize].block != NullBlock {
            if i == 0 {
                break;
            }
            i -= 1;
            let e = &self.hash_table[idx as usize];
            if e.hash_a == hash_a && e.hash_b == hash_b && e.block != DeletedBlock {
                return idx;
            }
            idx = (idx + 1) & 0x7FF;
        }
        HashEntryNotFound
    }

    /// Original: `MpqWriter::WriteHeaderAndTables` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::WriteHeaderAndTables() sha=d04d210eb68c
    fn write_header_and_tables(&mut self) -> bool {
        self.write_header() && self.write_block_table() && self.write_hash_table()
    }

    /// Original: `MpqWriter::AddFile` (mpq_writer.cpp). Returns the block index.
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::AddFile(const char *filename, MpqBlockEntry *block, uint32_t blockIndex) sha=d17ec0eff1a0
    fn add_file(&mut self, filename: &str, block: Option<usize>) -> usize {
        let f = filename.as_bytes();
        let h1 = hash(f, 0);
        let h2 = hash(f, 1);
        let h3 = hash(f, 2);
        if self.get_hash_index(h1, h2, h3) != HashEntryNotFound {
            panic!("Hash collision between \"{filename}\" and existing file");
        }
        let mut h_idx = h1 & 0x7FF;
        let mut has_space = false;
        for _ in 0..HashEntriesCount {
            let b = self.hash_table[h_idx as usize].block;
            if b == NullBlock || b == DeletedBlock {
                has_space = true;
                break;
            }
            h_idx = (h_idx + 1) & 0x7FF;
        }
        if !has_space {
            panic!("Out of hash space");
        }
        let block_index = block.unwrap_or_else(|| self.new_block());
        self.hash_table[h_idx as usize] = MpqHashEntry { hash_a: h2, hash_b: h3, locale: 0, platform: 0, block: block_index as u32 };
        block_index
    }

    /// Original: `MpqWriter::WriteFileContents` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::WriteFileContents(const char *filename, const byte *fileData, size_t fileSize, MpqBlockEntry *block) sha=3ccd5004514e
    fn write_file_contents(&mut self, data: &[u8], bi: usize) -> std::io::Result<()> {
        let mut file_size = data.len() as u32;
        let num_sectors = (file_size + (BlockSize - 1)) / BlockSize;
        let offset_table_byte_size = 4 * (num_sectors + 1);
        let offset = self.find_free_block(file_size + offset_table_byte_size);
        self.block_table[bi] =
            MpqBlockEntry { offset, packed_size: file_size + offset_table_byte_size, unpacked_size: file_size, flags: FlagExists | CompressPkZip };

        let mut offset_table = vec![0u32; num_sectors as usize + 1];
        let stream = self.stream.as_mut().unwrap();
        stream.seek(SeekFrom::Start((offset + offset_table_byte_size) as u64))?;

        let mut dest_size = offset_table_byte_size;
        let mut cur_sector = 0;
        let mut pos = 0usize;
        loop {
            let len = file_size.min(BlockSize) as usize;
            let mut mpq_buf = data[pos..pos + len].to_vec();
            pos += len;
            let len = pkware_compress(&mut mpq_buf, len);
            stream.write_all(&mpq_buf[..len])?;
            offset_table[cur_sector] = dest_size;
            cur_sector += 1;
            dest_size += len as u32;
            if file_size <= BlockSize {
                break;
            }
            file_size -= BlockSize;
        }

        offset_table[num_sectors as usize] = dest_size;
        stream.seek(SeekFrom::Start(offset as u64))?;
        let table: Vec<u8> = offset_table.iter().flat_map(|v| v.to_le_bytes()).collect();
        stream.write_all(&table)?;
        stream.seek(SeekFrom::Current((dest_size - offset_table_byte_size) as i64))?;

        let block = self.block_table[bi];
        if dest_size < block.packed_size {
            let remaining_block_size = block.packed_size - dest_size;
            if remaining_block_size >= MinBlockSize {
                self.block_table[bi].packed_size = dest_size;
                self.alloc_block(dest_size + block.offset, remaining_block_size);
            }
        }
        Ok(())
    }

    /// Original: `MpqWriter::WriteHeader` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::WriteHeader() sha=f5769464fbaf
    fn write_header(&mut self) -> bool {
        let fhdr = MpqFileHeader {
            signature: DiabloSignature,
            header_size: DiabloSize,
            file_size: self.size as u32,
            version: 0,
            block_size_factor: BlockSizeFactor,
            hash_entries_offset: MpqHashEntryOffset,
            block_entries_offset: MpqBlockEntryOffset,
            hash_entries_count: HashEntriesCount,
            block_entries_count: BlockEntriesCount,
        };
        self.stream.as_mut().unwrap().write_all(&fhdr.to_bytes()).is_ok()
    }

    /// Original: `MpqWriter::WriteBlockTable` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::WriteBlockTable() sha=7e2cd3ef3329
    fn write_block_table(&mut self) -> bool {
        let mut buf = block_table_bytes(&self.block_table);
        encrypt(&mut buf, hash(b"(block table)", 3));
        self.stream.as_mut().unwrap().write_all(&buf).is_ok()
    }

    /// Original: `MpqWriter::WriteHashTable` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::WriteHashTable() sha=4ee61cddde61
    fn write_hash_table(&mut self) -> bool {
        let mut buf = hash_table_bytes(&self.hash_table);
        encrypt(&mut buf, hash(b"(hash table)", 3));
        self.stream.as_mut().unwrap().write_all(&buf).is_ok()
    }

    /// Original: `MpqWriter::RemoveHashEntry` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::RemoveHashEntry(const char *filename) sha=ab1277fbd409
    pub fn remove_hash_entry(&mut self, filename: &str) {
        let h_idx = self.fetch_handle(filename);
        if h_idx == HashEntryNotFound {
            return;
        }
        let bi = self.hash_table[h_idx as usize].block as usize;
        self.hash_table[h_idx as usize].block = DeletedBlock;
        let block = self.block_table[bi];
        self.block_table[bi] = MpqBlockEntry::default();
        self.alloc_block(block.offset, block.packed_size);
    }

    /// Original: `MpqWriter::RemoveHashEntries` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::RemoveHashEntries(bool (*fnGetName)(uint8_t, char *)) sha=6c450333b20a
    pub fn remove_hash_entries(&mut self, fn_get_name: &dyn Fn(u8) -> Option<String>) {
        let mut i: u8 = 0;
        while let Some(name) = fn_get_name(i) {
            self.remove_hash_entry(&name);
            i = i.wrapping_add(1);
        }
    }

    /// Original: `MpqWriter::WriteFile` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::WriteFile(const char *filename, const byte *data, size_t size) sha=8b1d982e43b9
    pub fn write_file(&mut self, filename: &str, data: &[u8]) -> bool {
        self.remove_hash_entry(filename);
        let bi = self.add_file(filename, None);
        if self.write_file_contents(data, bi).is_err() {
            self.remove_hash_entry(filename);
            return false;
        }
        true
    }

    /// Original: `MpqWriter::RenameFile` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::RenameFile(const char *name, const char *newName) sha=cb83002cb780
    pub fn rename_file(&mut self, name: &str, new_name: &str) {
        let index = self.fetch_handle(name);
        if index == HashEntryNotFound {
            return;
        }
        let block = self.hash_table[index as usize].block as usize;
        self.hash_table[index as usize].block = DeletedBlock;
        self.add_file(new_name, Some(block));
    }

    /// Original: `MpqWriter::HasFile` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::HasFile(const char *name) sha=e58a5c514a4a
    pub fn has_file(&self, name: &str) -> bool {
        self.fetch_handle(name) != HashEntryNotFound
    }
}

impl Drop for MpqWriter {
    /// Original: `MpqWriter::~MpqWriter` (mpq_writer.cpp).
    // @port mpq/mpq_writer.cpp|devilution::MpqWriter::~MpqWriter() sha=89f8336e4861
    fn drop(&mut self) {
        if self.stream.is_none() {
            return;
        }
        log::verbose!("Closing {}", self.name);
        let mut result = true;
        if !(self.stream.as_mut().unwrap().seek(SeekFrom::Start(0)).is_ok() && self.write_header_and_tables()) {
            result = false;
        }
        let file = self.stream.take().unwrap();
        if result && self.size != 0 {
            log::verbose!("ResizeFile(\"{}\", {})", self.name, self.size);
            result = file.set_len(self.size).is_ok();
        }
        drop(file);
        if !result {
            log::verbose!("Closing failed {}", self.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mpq::MpqArchive;

    #[test]
    fn written_archives_read_back() {
        let dir = std::env::temp_dir().join(format!("diablo1_rs_mpq_writer_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.sv");
        let _ = std::fs::remove_file(&path);
        let p = path.to_str().unwrap();
        let big: Vec<u8> = (0..10000u32).map(|i| (i % 251) as u8).collect();
        {
            let mut w = MpqWriter::open(p).unwrap();
            assert!(w.write_file("hero", b"hello hero"));
            assert!(w.write_file("perml00", &big));
            assert!(w.write_file("templ01", &big[..5000]));
        }
        {
            let mut w = MpqWriter::open(p).unwrap();
            assert!(w.has_file("templ01"));
            w.rename_file("templ01", "perml01");
            w.remove_hash_entry("hero");
            assert!(w.write_file("hero", b"second hero"));
        }
        let mut a = MpqArchive::open(&path).unwrap().unwrap();
        assert_eq!(a.read_file("hero").unwrap(), b"second hero");
        assert_eq!(a.read_file("perml00").unwrap(), big);
        assert_eq!(a.read_file("perml01").unwrap(), &big[..5000]);
        assert!(!a.has_file("templ01"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
