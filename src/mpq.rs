//! MPQ archive reading (`Source/mpq/mpq_reader.cpp`).
//!
//! DevilutionX wraps libmpq; the port reimplements the parts of libmpq it uses from the MPQ v1
//! format rules (header, encrypted hash/block tables, sector offset tables, per-sector keys,
//! PKWARE implode, single-method zlib/implode sectors). Diablo's archives use implode;
//! devilutionx.mpq uses zlib. Other methods (bzip2, combinations) fail loudly naming the method.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::encrypt::{decrypt, hash};
use crate::{bzip2, inflate, pkware};

const MPQ_FILE_IMPLODE: u32 = 0x0000_0100;
const MPQ_FILE_COMPRESS: u32 = 0x0000_0200;
const MPQ_FILE_ENCRYPTED: u32 = 0x0001_0000;
const MPQ_FILE_FIX_KEY: u32 = 0x0002_0000;
const MPQ_FILE_SINGLE_UNIT: u32 = 0x0100_0000;
const MPQ_FILE_EXISTS: u32 = 0x8000_0000;
const HASH_ENTRY_EMPTY: u32 = 0xFFFF_FFFF;
const HASH_ENTRY_DELETED: u32 = 0xFFFF_FFFE;

/// Errors, mirroring the libmpq error codes DevilutionX reports.
#[derive(Debug)]
pub enum MpqError {
    Open(std::io::Error),
    Format(&'static str),
    Read(std::io::Error),
    /// LIBMPQ_ERROR_EXIST: no such file in the archive.
    NotFound,
    Decompress(String),
    Unsupported(String),
}

impl std::fmt::Display for MpqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MpqError::Open(e) => write!(f, "error on opening file: {e}"),
            MpqError::Format(m) => write!(f, "file is not an mpq or is corrupted: {m}"),
            MpqError::Read(e) => write!(f, "error on reading file: {e}"),
            MpqError::NotFound => write!(f, "file or block does not exist in archive"),
            MpqError::Decompress(m) => write!(f, "error on decompressing block: {m}"),
            MpqError::Unsupported(m) => write!(f, "unsupported: {m}"),
        }
    }
}

impl std::error::Error for MpqError {}

#[derive(Clone, Copy, Debug)]
struct HashEntry {
    hash_a: u32,
    hash_b: u32,
    block_index: u32,
}

#[derive(Clone, Copy, Debug)]
struct BlockEntry {
    offset: u32,
    packed_size: u32,
    unpacked_size: u32,
    flags: u32,
}

pub type FileHash = [u32; 3];

pub struct MpqArchive {
    path: PathBuf,
    file: File,
    archive_offset: u64,
    sector_size: u32,
    hash_table: Vec<HashEntry>,
    block_table: Vec<BlockEntry>,
}

fn read_u32s(bytes: &[u8]) -> Vec<u32> {
    bytes.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect()
}

impl MpqArchive {
    /// Original: `MpqArchive::Open` (mpq_reader.cpp:11). `Ok(None)` when the file does not exist,
    /// which the original reports as "no archive, no error".
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::Open(const char *path, int32_t &error) sha=b0ed14ca59a4
    pub fn open(path: impl AsRef<Path>) -> Result<Option<MpqArchive>, MpqError> {
        let path = path.as_ref();
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(MpqError::Open(e)),
        };
        // The header may follow other data at any 512-byte boundary.
        let len = file.metadata().map_err(MpqError::Read)?.len();
        let mut archive_offset = None;
        let mut header = [0u8; 32];
        let mut pos = 0u64;
        while pos + 32 <= len {
            file.seek(SeekFrom::Start(pos)).map_err(MpqError::Read)?;
            file.read_exact(&mut header).map_err(MpqError::Read)?;
            if &header[0..4] == b"MPQ\x1A" {
                archive_offset = Some(pos);
                break;
            }
            pos += 512;
        }
        let archive_offset = archive_offset.ok_or(MpqError::Format("no MPQ header"))?;
        let h = read_u32s(&header);
        let block_size_shift = u16::from_le_bytes([header[14], header[15]]);
        let (hash_pos, block_pos, hash_count, block_count) = (h[4], h[5], h[6], h[7]);
        if hash_count == 0 || !hash_count.is_power_of_two() {
            return Err(MpqError::Format("hash table size is not a power of two"));
        }
        let read_table = |file: &mut File, pos: u32, count: u32, key_name: &[u8]| -> Result<Vec<u32>, MpqError> {
            let mut buf = vec![0u8; count as usize * 16];
            file.seek(SeekFrom::Start(archive_offset + pos as u64)).map_err(MpqError::Read)?;
            file.read_exact(&mut buf).map_err(MpqError::Read)?;
            decrypt(&mut buf, hash(key_name, 3));
            Ok(read_u32s(&buf))
        };
        let ht = read_table(&mut file, hash_pos, hash_count, b"(hash table)")?;
        let bt = read_table(&mut file, block_pos, block_count, b"(block table)")?;
        let hash_table = ht.chunks_exact(4).map(|e| HashEntry { hash_a: e[0], hash_b: e[1], block_index: e[3] }).collect();
        let block_table = bt
            .chunks_exact(4)
            .map(|e| BlockEntry { offset: e[0], packed_size: e[1], unpacked_size: e[2], flags: e[3] })
            .collect();
        Ok(Some(MpqArchive {
            path: path.to_path_buf(),
            file,
            archive_offset,
            sector_size: 512u32 << block_size_shift,
            hash_table,
            block_table,
        }))
    }

    /// Original: `MpqArchive::Clone` (mpq_reader.cpp:23): an independent handle on the same archive
    /// (the original duplicates the libmpq archive so another thread can read).
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::Clone(int32_t &error) sha=a4ee963ff7b9
    pub fn try_clone(&self) -> Result<MpqArchive, MpqError> {
        Ok(MpqArchive {
            path: self.path.clone(),
            file: File::open(&self.path).map_err(MpqError::Open)?,
            archive_offset: self.archive_offset,
            sector_size: self.sector_size,
            hash_table: self.hash_table.clone(),
            block_table: self.block_table.clone(),
        })
    }

    /// Original: `MpqArchive::CalculateFileHash` (mpq_reader.cpp:37).
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::CalculateFileHash(const char *filename) sha=70965a6ed444
    pub fn calculate_file_hash(filename: &str) -> FileHash {
        let name = filename.as_bytes();
        [hash(name, 0), hash(name, 1), hash(name, 2)]
    }

    /// Original: `MpqArchive::GetFileNumber` (mpq_reader.cpp:60).
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::GetFileNumber(MpqArchive::FileHash fileHash, uint32_t &fileNumber) sha=22fe78b48116
    pub fn get_file_number(&self, file_hash: FileHash) -> Option<u32> {
        let mask = self.hash_table.len() as u32 - 1;
        let start = file_hash[0] & mask;
        let mut i = start;
        loop {
            let e = self.hash_table[i as usize];
            if e.block_index == HASH_ENTRY_EMPTY {
                return None;
            }
            if e.hash_a == file_hash[1]
                && e.hash_b == file_hash[2]
                && e.block_index != HASH_ENTRY_DELETED
                && (e.block_index as usize) < self.block_table.len()
                && self.block_table[e.block_index as usize].flags & MPQ_FILE_EXISTS != 0
            {
                return Some(e.block_index);
            }
            i = (i + 1) & mask;
            if i == start {
                return None;
            }
        }
    }

    /// Original: `MpqArchive::HasFile` (mpq_reader.cpp:147).
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::HasFile(const char *filename) sha=21515a09a62c
    pub fn has_file(&self, filename: &str) -> bool {
        self.get_file_number(Self::calculate_file_hash(filename)).is_some()
    }

    fn block(&self, file_number: u32) -> Result<BlockEntry, MpqError> {
        self.block_table.get(file_number as usize).copied().ok_or(MpqError::NotFound)
    }

    /// Original: `MpqArchive::GetUnpackedFileSize` (mpq_reader.cpp:115).
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::GetUnpackedFileSize(uint32_t fileNumber, int32_t &error) sha=51f3c458d62f
    pub fn unpacked_file_size(&self, file_number: u32) -> Result<usize, MpqError> {
        Ok(self.block(file_number)?.unpacked_size as usize)
    }

    /// Original: `MpqArchive::GetNumBlocks` (mpq_reader.cpp:122).
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::GetNumBlocks(uint32_t fileNumber, int32_t &error) sha=765e17258a62
    pub fn num_blocks(&self, file_number: u32) -> Result<u32, MpqError> {
        let b = self.block(file_number)?;
        Ok(if b.flags & MPQ_FILE_SINGLE_UNIT != 0 { 1 } else { b.unpacked_size.div_ceil(self.sector_size) })
    }

    /// Original: `MpqArchive::GetBlockSize` (mpq_reader.cpp:140): unpacked size of one block.
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::GetBlockSize(uint32_t fileNumber, uint32_t blockNumber, int32_t &error) sha=1c66ced1265e
    pub fn block_size(&self, file_number: u32, block_number: u32) -> Result<usize, MpqError> {
        let b = self.block(file_number)?;
        if b.flags & MPQ_FILE_SINGLE_UNIT != 0 {
            return if block_number == 0 { Ok(b.unpacked_size as usize) } else { Err(MpqError::NotFound) };
        }
        let n = self.num_blocks(file_number)?;
        if block_number >= n {
            return Err(MpqError::NotFound);
        }
        let start = block_number * self.sector_size;
        Ok((b.unpacked_size - start).min(self.sector_size) as usize)
    }

    /// The decryption key of a file: hash of its base name, adjusted for FIX_KEY files.
    fn file_key(filename: &str, b: &BlockEntry) -> u32 {
        let base = filename.rsplit(['\\', '/']).next().unwrap_or(filename);
        let mut key = hash(base.as_bytes(), 3);
        if b.flags & MPQ_FILE_FIX_KEY != 0 {
            key = key.wrapping_add(b.offset) ^ b.unpacked_size;
        }
        key
    }

    /// Original: `MpqArchive::OpenBlockOffsetTable` (mpq_reader.cpp:129): reads (and decrypts)
    /// the sector offset table. The original keeps it open across ReadBlock calls; callers here
    /// pass the returned table to `read_block_with_offsets`.
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::OpenBlockOffsetTable(uint32_t fileNumber, const char *filename) sha=c7e2502f3d28
    pub fn open_block_offset_table(&mut self, file_number: u32, filename: &str) -> Result<Vec<u32>, MpqError> {
        let b = self.block(file_number)?;
        let compressed = b.flags & (MPQ_FILE_IMPLODE | MPQ_FILE_COMPRESS) != 0;
        if b.flags & MPQ_FILE_SINGLE_UNIT != 0 {
            return Ok(vec![0, b.packed_size]);
        }
        let n = self.num_blocks(file_number)?;
        if !compressed {
            // Stored files: sectors are contiguous and full-size.
            return Ok((0..=n).map(|i| (i * self.sector_size).min(b.packed_size)).collect());
        }
        let mut buf = vec![0u8; (n as usize + 1) * 4];
        self.file.seek(SeekFrom::Start(self.archive_offset + b.offset as u64)).map_err(MpqError::Read)?;
        self.file.read_exact(&mut buf).map_err(MpqError::Read)?;
        if b.flags & MPQ_FILE_ENCRYPTED != 0 {
            decrypt(&mut buf, Self::file_key(filename, &b).wrapping_sub(1));
        }
        let table = read_u32s(&buf);
        if table[0] as usize != buf.len() || table.windows(2).any(|w| w[1] < w[0]) || *table.last().unwrap() > b.packed_size {
            return Err(MpqError::Format("bad sector offset table (wrong key?)"));
        }
        Ok(table)
    }

    /// Reads and unpacks block `block_number` into `out` (exactly `block_size` bytes).
    pub fn read_block_with_offsets(
        &mut self,
        file_number: u32,
        filename: &str,
        offsets: &[u32],
        block_number: u32,
        out: &mut [u8],
    ) -> Result<(), MpqError> {
        let b = self.block(file_number)?;
        let unpacked = self.block_size(file_number, block_number)?;
        if out.len() < unpacked {
            return Err(MpqError::Decompress(format!("output buffer {} < block {}", out.len(), unpacked)));
        }
        let (start, end) = (offsets[block_number as usize], offsets[block_number as usize + 1]);
        let mut buf = vec![0u8; (end - start) as usize];
        self.file
            .seek(SeekFrom::Start(self.archive_offset + b.offset as u64 + start as u64))
            .map_err(MpqError::Read)?;
        self.file.read_exact(&mut buf).map_err(MpqError::Read)?;
        if b.flags & MPQ_FILE_ENCRYPTED != 0 {
            decrypt(&mut buf, Self::file_key(filename, &b).wrapping_add(block_number));
        }
        let out = &mut out[..unpacked];
        if buf.len() == unpacked {
            out.copy_from_slice(&buf);
        } else if b.flags & MPQ_FILE_IMPLODE != 0 {
            let mut v = Vec::with_capacity(unpacked);
            pkware::explode(&buf, &mut v).map_err(|e| MpqError::Decompress(format!("{e:?}")))?;
            if v.len() != unpacked {
                return Err(MpqError::Decompress(format!("exploded {} bytes, expected {unpacked}", v.len())));
            }
            out.copy_from_slice(&v);
        } else if b.flags & MPQ_FILE_COMPRESS != 0 {
            // first byte: mask of methods; only single-method sectors occur in the game's archives
            let mut v = Vec::with_capacity(unpacked);
            match buf.first().copied().unwrap_or(0) {
                0x02 => inflate::inflate_zlib(&buf[1..], &mut v).map_err(|e| MpqError::Decompress(e.0.to_string()))?,
                0x10 => bzip2::decompress(&buf[1..], &mut v).map_err(|e| MpqError::Decompress(e.0.to_string()))?,
                0x08 => {
                    pkware::explode(&buf[1..], &mut v).map_err(|e| MpqError::Decompress(format!("{e:?}")))?;
                }
                m => {
                    return Err(MpqError::Unsupported(format!("compression methods {m:#04x} in {filename}")));
                }
            }
            if v.len() != unpacked {
                return Err(MpqError::Decompress(format!("unpacked {} bytes, expected {unpacked}", v.len())));
            }
            out.copy_from_slice(&v);
        } else {
            return Err(MpqError::Format("packed sector of an uncompressed file"));
        }
        Ok(())
    }

    /// Original: `MpqArchive::ReadBlock` (mpq_reader.cpp:106).
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::ReadBlock(uint32_t fileNumber, uint32_t blockNumber, uint8_t *out, uint32_t outSize) sha=d686e8e34d04
    pub fn read_block(&mut self, file_number: u32, filename: &str, block_number: u32, out: &mut [u8]) -> Result<(), MpqError> {
        let offsets = self.open_block_offset_table(file_number, filename)?;
        self.read_block_with_offsets(file_number, filename, &offsets, block_number, out)
    }

    /// Original: `MpqArchive::ReadFile` (mpq_reader.cpp:65).
    // @port mpq/mpq_reader.cpp|devilution::MpqArchive::ReadFile(const char *filename, std::size_t &fileSize, int32_t &error) sha=323cb867171f
    pub fn read_file(&mut self, filename: &str) -> Result<Vec<u8>, MpqError> {
        let n = self.get_file_number(Self::calculate_file_hash(filename)).ok_or(MpqError::NotFound)?;
        let size = self.unpacked_file_size(n)?;
        let offsets = self.open_block_offset_table(n, filename)?;
        let mut out = vec![0u8; size];
        let mut pos = 0;
        for i in 0..self.num_blocks(n)? {
            let len = self.block_size(n, i)?;
            self.read_block_with_offsets(n, filename, &offsets, i, &mut out[pos..pos + len])?;
            pos += len;
        }
        Ok(out)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_data::diabdat;

    #[test]
    fn reads_known_files_from_diabdat() {
        let Some(path) = diabdat() else { return };
        let mut mpq = MpqArchive::open(&path).unwrap().expect("DIABDAT.MPQ exists");
        assert!(mpq.has_file("ui_art\\title.pcx"));
        assert!(!mpq.has_file("ui_art\\no_such_file.pcx"));
        let pcx = mpq.read_file("ui_art\\title.pcx").unwrap();
        assert_eq!(pcx[0], 0x0A, "PCX manufacturer byte");
        // the town palette is 256 RGB triples
        let pal = mpq.read_file("levels\\towndata\\town.pal").unwrap();
        assert_eq!(pal.len(), 768);
        // an imploded, multi-sector file
        let cel = mpq.read_file("levels\\l1data\\l1.cel").unwrap();
        assert!(cel.len() > 4096);
        let frames = u32::from_le_bytes(cel[0..4].try_into().unwrap()) as usize;
        let end = u32::from_le_bytes(cel[4 + frames * 4..8 + frames * 4].try_into().unwrap()) as usize;
        assert_eq!(end, cel.len(), "CEL frame table ends at the file size");
    }

    #[test]
    fn missing_archive_is_not_an_error() {
        assert!(MpqArchive::open("Z:\\definitely\\not\\here.mpq").unwrap().is_none());
    }
}
