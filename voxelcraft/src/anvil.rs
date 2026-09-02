//! Anvil region file format (`.mca`) — reader + writer.
//!
//! Clean-room implementation of the publicly documented format
//! (https://minecraft.wiki/w/Region_file_format), the container vanilla
//! 1.16.5 uses for chunk storage:
//!
//! * a region covers 32×32 chunks; the file is an array of **4096-byte
//!   sectors**;
//! * header = 1024 location entries (3-byte big-endian sector offset +
//!   1-byte sector count; zero = chunk absent) followed by 1024
//!   big-endian u32 timestamps = 8 KiB = sectors 0 and 1;
//! * chunk payload record = 4-byte big-endian length (of what follows)
//!   + 1-byte compression type + `length − 1` compressed bytes;
//!   types: 1 = GZip, **2 = zlib (what vanilla 1.16.5 writes)**, 3 = none;
//! * chunk slot index = `(x & 31) + 32 * (z & 31)`.
//!
//! Writing strategy (native saves): **compact-and-rewrite** — the whole
//! file is rewritten with all live chunk payloads packed from sector 2
//! onward. This is O(file) per flush (a few MB at typical render
//! distances) at a 20-second autosave cadence, keeps the file bounded
//! (no append-only growth from overwrites), and needs no persistent
//! free-list. Sector allocation/compaction improvements are a future
//! optimization gated on profiling (§0.3), not correctness.
//!
//! Malformed/corrupt regions never panic (§46): reads return
//! `Ok(None)`/`Err` and the caller regenerates the chunk from terrain
//! gen, falling back to a safe default.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const SECTOR_BYTES: usize = 4096;
pub const CHUNKS_PER_SIDE: usize = 32;
/// 2 sectors of header (1024 + 1024 entries × 4 bytes = 8 KiB).
pub const FIRST_DATA_SECTOR: u32 = 2;

pub const COMPRESSION_GZIP: u8 = 1;
pub const COMPRESSION_ZLIB: u8 = 2;
pub const COMPRESSION_NONE: u8 = 3;

/// Vanilla 1.16.5 always writes zlib — we do too (load path still
/// accepts gzip + none for pack-tool interop).
const WRITE_COMPRESSION: u8 = COMPRESSION_ZLIB;

// -------------------------------------------------------------- helpers --

#[inline]
fn slot_index(x: i32, z: i32) -> usize {
    ((x & 31) as usize) + 32 * ((z & 31) as usize)
}

/// Region file path for world-relative chunk coords:
/// `region/r.<x.div_euclid(32)>.<z.div_euclid(32)>.mca`.
pub fn region_path(world_dir: &Path, cx: i32, cz: i32) -> PathBuf {
    let mut p = world_dir.to_path_buf();
    p.push("region");
    let _ = fs::create_dir_all(&p); // idempotent; failures surface at write
    p.push(format!("r.{}.{}.mca", cx.div_euclid(32), cz.div_euclid(32)));
    p
}

fn u32_to_be3(v: u32) -> [u8; 3] {
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

fn be3_to_u32(b: &[u8]) -> u32 {
    ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32
}

fn now_secs() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as u32)
        .unwrap_or(0)
}

fn compress(payload: &[u8], scheme: u8) -> std::io::Result<Vec<u8>> {
    match scheme {
        COMPRESSION_ZLIB => {
            let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            enc.write_all(payload)?;
            enc.finish()
        }
        COMPRESSION_GZIP => {
            let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            enc.write_all(payload)?;
            enc.finish()
        }
        COMPRESSION_NONE => Ok(payload.to_vec()),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("anvil: unsupported compression scheme {scheme}"),
        )),
    }
}

fn decompress(data: &[u8], scheme: u8) -> std::io::Result<Vec<u8>> {
    match scheme {
        COMPRESSION_ZLIB => {
            let mut out = Vec::new();
            flate2::read::ZlibDecoder::new(data).read_to_end(&mut out)?;
            Ok(out)
        }
        COMPRESSION_GZIP => {
            let mut out = Vec::new();
            flate2::read::GzDecoder::new(data).read_to_end(&mut out)?;
            Ok(out)
        }
        COMPRESSION_NONE => Ok(data.to_vec()),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("anvil: unknown compression type {scheme}"),
        )),
    }
}

// --------------------------------------------------------------- reading --

/// One raw chunk record: compression byte + (still compressed) payload.
#[derive(Clone)]
struct RawChunk {
    scheme: u8,
    data: Vec<u8>,
}

/// Read a single chunk's **decompressed NBT bytes** from the region file
/// containing world chunk (cx, cz). `Ok(None)` = chunk not present (or
/// region file absent) → caller generates from terrain.
pub fn read_chunk(world_dir: &Path, cx: i32, cz: i32) -> std::io::Result<Option<Vec<u8>>> {
    let path = region_path(world_dir, cx, cz);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)?;
    // header needs 8 KiB; anything smaller is an empty/corrupt region
    if bytes.len() < 2 * SECTOR_BYTES {
        return Ok(None);
    }
    let slot = slot_index(cx, cz);
    let loc = &bytes[slot * 4..slot * 4 + 4];
    let sector = be3_to_u32(loc);
    let count = loc[3] as u32;
    if sector == 0 || count == 0 {
        return Ok(None); // chunk absent
    }
    let start = sector as usize * SECTOR_BYTES;
    // bounds + truncation guards (§46 — never panic, treat as absent).
    // A record needs at least 5 bytes past the sector start: 4 length +
    // 1 compression byte — anything less is a corrupt/torn tail.
    let available = bytes.len().saturating_sub(start);
    if available < 5 {
        return Ok(None);
    }
    let record_len = u32::from_be_bytes(
        bytes[start..start + 4].try_into().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, format!("anvil: short length field: {e}"))
        })?,
    ) as usize;
    if record_len == 0 || record_len.saturating_add(4) > available {
        return Ok(None); // corrupt length — treat chunk as absent
    }
    let record = &bytes[start + 4..start + 4 + record_len];
    if record.is_empty() {
        return Ok(None);
    }
    let scheme = record[0];
    let payload = &record[1..];
    let decompressed = decompress(payload, scheme)?;
    Ok(Some(decompressed))
}

// --------------------------------------------------------------- writing --

/// Write one chunk's (uncompressed NBT) bytes into its region file,
/// preserving all other chunks. Compact-and-rewrite (see module doc):
/// every live record is re-packed from sector 2, timestamps refreshed
/// for the touched slot only, others preserved.
///
/// Atomic-ish: writes `path.tmp` then renames over the target, so a crash
/// mid-save leaves either the old or the new file, never a torn header.
pub fn write_chunk(world_dir: &Path, cx: i32, cz: i32, nbt_bytes: &[u8]) -> std::io::Result<()> {
    let path = region_path(world_dir, cx, cz);

    // 1. collect existing records (raw, still compressed)
    let mut records: Vec<Option<RawChunk>> = vec![None; CHUNKS_PER_SIDE * CHUNKS_PER_SIDE];
    let mut timestamps = vec![0u32; CHUNKS_PER_SIDE * CHUNKS_PER_SIDE];
    let mut existing: Vec<u8> = Vec::new();
    if path.exists() {
        existing = fs::read(&path)?;
        if existing.len() >= 2 * SECTOR_BYTES {
            for slot in 0..CHUNKS_PER_SIDE * CHUNKS_PER_SIDE {
                let loc = &existing[slot * 4..slot * 4 + 4];
                let sector = be3_to_u32(loc);
                let count = loc[3] as u32;
                timestamps[slot] = u32::from_be_bytes(
                    existing[SECTOR_BYTES + slot * 4..SECTOR_BYTES + slot * 4 + 4]
                        .try_into()
                        .expect("4-byte slice"),
                );
                if sector == 0 || count == 0 {
                    continue;
                }
                let start = sector as usize * SECTOR_BYTES;
                if start + 4 >= existing.len() {
                    continue; // corrupt — drop the record
                }
                let record_len =
                    u32::from_be_bytes(existing[start..start + 4].try_into().expect("4 bytes"))
                        as usize;
                if record_len == 0 || start + 4 + record_len > existing.len() {
                    continue; // corrupt — drop
                }
                let record = &existing[start + 4..start + 4 + record_len];
                if record.is_empty() {
                    continue;
                }
                records[slot] = Some(RawChunk {
                    scheme: record[0],
                    data: record[1..].to_vec(),
                });
            }
        }
    }

    // 2. compress + place the new record
    let compressed = compress(nbt_bytes, WRITE_COMPRESSION)?;
    let capacity_hint = existing.len() + compressed.len() + SECTOR_BYTES;
    records[slot_index(cx, cz)] = Some(RawChunk {
        scheme: WRITE_COMPRESSION,
        data: compressed,
    });
    timestamps[slot_index(cx, cz)] = now_secs();

    // 3. serialize: header then packed payloads from sector 2
    let mut out: Vec<u8> = Vec::with_capacity(capacity_hint);
    out.resize(2 * SECTOR_BYTES, 0);
    let mut next_sector = FIRST_DATA_SECTOR;
    for slot in 0..CHUNKS_PER_SIDE * CHUNKS_PER_SIDE {
        let Some(rec) = &records[slot] else { continue };
        let record_len = 1 + rec.data.len();
        // whole on-disk record incl. the 4-byte length prefix
        let sectors = (4 + record_len).div_ceil(SECTOR_BYTES) as u32;
        // location-table sector count is ONE byte: a record spanning more
        // than 255 sectors (> ~1 MiB compressed) cannot be represented —
        // reject instead of silently wrapping the count (§46).
        if sectors > u8::MAX as u32 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::FileTooLarge,
                format!(
                    "anvil: chunk ({cx},{cz}) compressed record spans {sectors} sectors (max 255)"
                ),
            ));
        }
        // header entry
        let be3 = u32_to_be3(next_sector);
        out[slot * 4..slot * 4 + 3].copy_from_slice(&be3);
        out[slot * 4 + 3] = sectors as u8;
        // timestamp table lives at bytes 4096..8192 (second header sector)
        let ts = timestamps[slot].to_be_bytes();
        out[SECTOR_BYTES + slot * 4..SECTOR_BYTES + slot * 4 + 4].copy_from_slice(&ts);
        // payload — the on-disk record is [4-byte BE length][scheme][data];
        // `record_len` counts what FOLLOWS the length field, so the buffer
        // must hold 4 + record_len bytes, and the sector count (and the
        // location-table count byte) covers the WHOLE record incl. prefix.
        let start = out.len();
        out.resize(start + 4 + record_len, 0);
        out[start..start + 4].copy_from_slice(&(record_len as u32).to_be_bytes());
        out[start + 4] = rec.scheme;
        out[start + 5..start + 5 + rec.data.len()].copy_from_slice(&rec.data);
        // zero padding to sector alignment (resize above zero-filled)
        out.resize((next_sector as usize + sectors as usize) * SECTOR_BYTES, 0);
        next_sector += sectors;
    }

    // 4. atomic replace
    let tmp = path.with_extension("mca.tmp");
    fs::write(&tmp, &out)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

// ------------------------------------------------------------------ tests --

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nbt;
    use std::time::SystemTime;

    fn tmp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "vc-anvil-{tag}-{}-{}",
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(),
            std::process::id()
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn demo_chunk_nbt(x: i32, z: i32, marker: i32) -> Vec<u8> {
        let mut root = nbt::Nbt::compound();
        root.set("DataVersion", nbt::Nbt::Int(2586));
        let mut level = nbt::Nbt::compound();
        level.set("xPos", nbt::Nbt::Int(x));
        level.set("zPos", nbt::Nbt::Int(z));
        level.set("marker", nbt::Nbt::Int(marker));
        root.set("Level", level);
        nbt::write_root("", &root).unwrap()
    }

    #[test]
    fn roundtrip_single_chunk() {
        let dir = tmp_dir("single");
        let bytes = demo_chunk_nbt(0, 0, 42);
        write_chunk(&dir, 0, 0, &bytes).unwrap();
        let back = read_chunk(&dir, 0, 0).unwrap().expect("chunk present");
        assert_eq!(back, bytes);
    }

    #[test]
    fn region_file_layout_is_exact() {
        let dir = tmp_dir("layout");
        let bytes = demo_chunk_nbt(3, 5, 1);
        write_chunk(&dir, 3, 5, &bytes).unwrap();
        let raw = fs::read(region_path(&dir, 3, 5)).unwrap();
        // sector-aligned total size
        assert_eq!(raw.len() % SECTOR_BYTES, 0);
        assert!(raw.len() >= 3 * SECTOR_BYTES); // header + at least 1 data sector
        // header location: slot (3 & 31) + 32*(5 & 31) = 163
        let slot = slot_index(3, 5);
        let sector = be3_to_u32(&raw[slot * 4..slot * 4 + 4]);
        let count = raw[slot * 4 + 3] as u32;
        assert_eq!(sector, FIRST_DATA_SECTOR); // first record packs at sector 2
        assert!(count >= 1);
        // record: 4-byte BE length = 1 + compressed len; type = 2 (zlib)
        let start = sector as usize * SECTOR_BYTES;
        let record_len = u32::from_be_bytes(raw[start..start + 4].try_into().unwrap()) as usize;
        assert_eq!(raw[start + 4], COMPRESSION_ZLIB);
        // decompress the stored record — it must equal the original NBT bytes
        let payload = &raw[start + 5..start + 4 + record_len];
        let mut dec = flate2::read::ZlibDecoder::new(payload);
        let mut nbt_back = Vec::new();
        dec.read_to_end(&mut nbt_back).unwrap();
        assert_eq!(nbt_back, bytes);
        // record fits inside its allocated sectors, file stays sector-aligned
        assert!(start + 4 + record_len <= (sector as usize + count as usize) * SECTOR_BYTES);
        // timestamp non-zero (table at 4096..8192)
        let ts = u32::from_be_bytes(
            raw[SECTOR_BYTES + slot * 4..SECTOR_BYTES + slot * 4 + 4]
                .try_into()
                .unwrap(),
        );
        assert!(ts > 0);
    }

    #[test]
    fn absent_chunk_returns_none() {
        let dir = tmp_dir("absent");
        // no region file at all
        assert!(read_chunk(&dir, 0, 0).unwrap().is_none());
        // region exists, different slot empty
        write_chunk(&dir, 0, 0, &demo_chunk_nbt(0, 0, 1)).unwrap();
        assert!(read_chunk(&dir, 1, 0).unwrap().is_none());
        assert!(read_chunk(&dir, 0, 1).unwrap().is_none());
        assert!(read_chunk(&dir, 31, 31).unwrap().is_none());
    }

    #[test]
    fn overwrite_and_multi_chunk_preservation() {
        let dir = tmp_dir("multi");
        // three chunks in the same region, plus one neighbor region
        let a = demo_chunk_nbt(0, 0, 10);
        let b = demo_chunk_nbt(31, 31, 11);
        let c = demo_chunk_nbt(15, 15, 12);
        let d = demo_chunk_nbt(32, 0, 13); // next region over
        write_chunk(&dir, 0, 0, &a).unwrap();
        write_chunk(&dir, 31, 31, &b).unwrap();
        write_chunk(&dir, 15, 15, &c).unwrap();
        write_chunk(&dir, 32, 0, &d).unwrap();

        // overwrite A with new content — B, C must survive byte-identical
        let a2 = demo_chunk_nbt(0, 0, 99);
        write_chunk(&dir, 0, 0, &a2).unwrap();
        assert_eq!(read_chunk(&dir, 0, 0).unwrap().unwrap(), a2);
        assert_eq!(read_chunk(&dir, 31, 31).unwrap().unwrap(), b);
        assert_eq!(read_chunk(&dir, 15, 15).unwrap().unwrap(), c);
        assert_eq!(read_chunk(&dir, 32, 0).unwrap().unwrap(), d);

        // negative coords route to negative regions
        let n = demo_chunk_nbt(-1, -1, 14);
        write_chunk(&dir, -1, -1, &n).unwrap();
        assert_eq!(read_chunk(&dir, -1, -1).unwrap().unwrap(), n);
        assert_eq!(
            region_path(&dir, -1, -1).file_name().unwrap().to_str().unwrap(),
            "r.-1.-1.mca"
        );
        // (-1, -1) and (31, 31) share that region but different slots
        assert_eq!(read_chunk(&dir, 31, 31).unwrap().unwrap(), b);
    }

    #[test]
    fn overwrite_keeps_file_bounded() {
        // repeated overwrites of the same slot must not grow the file
        // without bound (compact-and-rewrite property)
        let dir = tmp_dir("bounded");
        let payload = demo_chunk_nbt(0, 0, 0);
        write_chunk(&dir, 0, 0, &payload).unwrap();
        let first = fs::read(region_path(&dir, 0, 0)).unwrap().len();
        for i in 0..8 {
            write_chunk(&dir, 0, 0, &demo_chunk_nbt(0, 0, i)).unwrap();
        }
        let last = fs::read(region_path(&dir, 0, 0)).unwrap().len();
        assert!(last <= first + SECTOR_BYTES, "grew: {first} -> {last}");
    }

    #[test]
    fn corrupt_file_degrades_to_absent() {
        // garbage file → chunk reads as absent, no panic (§46)
        let dir = tmp_dir("corrupt");
        let path = region_path(&dir, 0, 0);
        fs::write(&path, b"not a region file at all").unwrap();
        assert!(read_chunk(&dir, 0, 0).unwrap().is_none());
        // truncated valid file (header only) → absent
        let dir2 = tmp_dir("trunc");
        write_chunk(&dir2, 0, 0, &demo_chunk_nbt(0, 0, 1)).unwrap();
        let p2 = region_path(&dir2, 0, 0);
        let full = fs::read(&p2).unwrap();
        fs::write(&p2, &full[..2 * SECTOR_BYTES]).unwrap();
        assert!(read_chunk(&dir2, 0, 0).unwrap().is_none());
        // write must still succeed over the corrupt file (recovery path)
        write_chunk(&dir, 0, 0, &demo_chunk_nbt(0, 0, 2)).unwrap();
        assert!(read_chunk(&dir, 0, 0).unwrap().is_some());
    }
}
