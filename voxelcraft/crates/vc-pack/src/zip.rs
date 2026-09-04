//! Minimal ZIP archive reader for data packs (Phase 9).
//!
//! Vanilla accepts data packs as folders OR `.zip` archives in
//! `<world>/datapacks/` (wiki Data pack page, live-verified). The engine
//! already depends on flate2 (region gzip), so zip reading is built on
//! it — zero new dependencies. This is a READER for the standard zip
//! structure (PKWARE APPNOTE): EOCD scan → central directory → local
//! header → data, with method 0 (stored) and method 8 (deflate).
//!
//! Deliberately out of scope: zip64 (>4 GB packs), encryption, spanning,
//! and non-deflate exotic codecs — real-world data packs never use them;
//! unknown methods are skipped with an honest reason string.

/// one zip entry, ready to read.
struct ZipEntry {
    name: String,
    method: u16,
    comp_size: u32,
    uncomp_size: u32,
    crc32: u32,
    local_offset: u32,
}

/// a parsed zip archive: names + entry metadata (data read on demand).
pub struct ZipFiles {
    bytes: Vec<u8>,
    entries: Vec<ZipEntry>,
}

impl ZipFiles {
    /// parse the central directory of a zip byte stream. Returns None
    /// when the archive is not a readable zip (no EOCD / truncated).
    pub fn from_bytes(bytes: &[u8]) -> Option<ZipFiles> {
        const EOCD_SIG: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];
        const CD_SIG: [u8; 4] = [0x50, 0x4B, 0x01, 0x02];
        // EOCD lives at the end; the comment may be up to 65535 bytes —
        // scan backwards for the signature
        if bytes.len() < 22 {
            return None;
        }
        let mut eocd = None;
        let scan_start = bytes.len().saturating_sub(22 + 65535);
        let mut i = bytes.len() - 22;
        loop {
            if bytes[i..i + 4] == EOCD_SIG {
                eocd = Some(i);
                break;
            }
            if i == scan_start {
                break;
            }
            i -= 1;
        }
        let eocd = eocd?;
        let rd16 = |o: usize| u16::from_le_bytes([bytes[o], bytes[o + 1]]);
        let rd32 = |o: usize| {
            u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]])
        };
        let total_entries = rd16(eocd + 10) as usize;
        let cd_size = rd32(eocd + 12) as usize;
        let cd_offset = rd32(eocd + 16) as usize;
        if cd_offset + cd_size > bytes.len() {
            return None; // truncated central directory
        }
        let mut entries = Vec::new();
        let mut p = cd_offset;
        for _ in 0..total_entries {
            if p + 46 > bytes.len() || bytes[p..p + 4] != CD_SIG {
                break; // corrupt CD — read what we can
            }
            let method = rd16(p + 10);
            let crc32 = rd32(p + 16);
            let comp_size = rd32(p + 20);
            let uncomp_size = rd32(p + 24);
            let name_len = rd16(p + 28) as usize;
            let extra_len = rd16(p + 30) as usize;
            let comment_len = rd16(p + 32) as usize;
            let local_offset = rd32(p + 42);
            let name_end = p + 46 + name_len;
            if name_end > bytes.len() {
                break;
            }
            let name = String::from_utf8_lossy(&bytes[p + 46..name_end]).to_string();
            entries.push(ZipEntry {
                name,
                method,
                comp_size,
                uncomp_size,
                crc32,
                local_offset,
            });
            p = name_end + extra_len + comment_len;
        }
        Some(ZipFiles {
            bytes: bytes.to_vec(),
            entries,
        })
    }

    /// read + inflate one entry. Returns None on unknown method, size
    /// mismatch or CRC mismatch (vanilla rejects corrupt packs; here the
    /// caller skips the file with an honest reason).
    fn read_entry(&self, entry: &ZipEntry) -> Option<Vec<u8>> {
        const LOCAL_SIG: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
        let b = &self.bytes;
        let lo = entry.local_offset as usize;
        if lo + 30 > b.len() || b[lo..lo + 4] != LOCAL_SIG {
            return None;
        }
        let name_len = u16::from_le_bytes([b[lo + 26], b[lo + 27]]) as usize;
        let extra_len = u16::from_le_bytes([b[lo + 28], b[lo + 29]]) as usize;
        let data = lo + 30 + name_len + extra_len;
        let end = data + entry.comp_size as usize;
        if end > b.len() {
            return None;
        }
        let raw = &b[data..end];
        let out = match entry.method {
            0 => raw.to_vec(),
            8 => {
                use std::io::Read;
                let mut d = flate2::read::DeflateDecoder::new(raw);
                let mut out = Vec::with_capacity(entry.uncomp_size as usize);
                d.read_to_end(&mut out).ok()?;
                out
            }
            _ => return None, // unsupported codec — honest skip
        };
        if out.len() != entry.uncomp_size as usize {
            return None;
        }
        // CRC check (real-world packs are usually correct; mismatches mean
        // corruption — skip the file rather than feed garbage to the
        // JSON parser)
        let mut crc = flate2::Crc::new();
        crc.update(&out);
        if crc.sum() != entry.crc32 {
            return None;
        }
        Some(out)
    }
}

impl super::datapack::PackFiles for ZipFiles {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        let entry = self.entries.iter().find(|e| e.name == path)?;
        self.read_entry(entry)
    }
    fn list(&self, prefix: &str) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| e.name.starts_with(prefix) && !e.name.ends_with('/'))
            .map(|e| e.name.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datapack::PackFiles;

    /// hand-build a minimal (stored-method) zip so the reader is tested
    /// against bytes we construct from the spec, not a library round-trip
    fn build_stored_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        let mut crc = flate2::Crc::new();
        for (name, data) in files {
            crc.reset();
            crc.update(data);
            let crc = crc.sum();
            let offset = out.len() as u32;
            // local file header
            out.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
            out.extend_from_slice(&20u16.to_le_bytes()); // version needed
            out.extend_from_slice(&0u16.to_le_bytes()); // flags
            out.extend_from_slice(&0u16.to_le_bytes()); // method: stored
            out.extend_from_slice(&0u16.to_le_bytes()); // time
            out.extend_from_slice(&0u16.to_le_bytes()); // date
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes()); // extra len
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(data);
            // central directory entry
            central.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
            central.extend_from_slice(&20u16.to_le_bytes()); // version made by
            central.extend_from_slice(&20u16.to_le_bytes()); // version needed
            central.extend_from_slice(&0u16.to_le_bytes()); // flags
            central.extend_from_slice(&0u16.to_le_bytes()); // method
            central.extend_from_slice(&0u16.to_le_bytes()); // time
            central.extend_from_slice(&0u16.to_le_bytes()); // date
            central.extend_from_slice(&crc.to_le_bytes());
            central.extend_from_slice(&(data.len() as u32).to_le_bytes());
            central.extend_from_slice(&(data.len() as u32).to_le_bytes());
            central.extend_from_slice(&(name.len() as u16).to_le_bytes());
            central.extend_from_slice(&0u16.to_le_bytes()); // extra
            central.extend_from_slice(&0u16.to_le_bytes()); // comment
            central.extend_from_slice(&0u16.to_le_bytes()); // disk
            central.extend_from_slice(&0u16.to_le_bytes()); // int attrs
            central.extend_from_slice(&0u32.to_le_bytes()); // ext attrs
            central.extend_from_slice(&offset.to_le_bytes());
            central.extend_from_slice(name.as_bytes());
        }
        let cd_offset = out.len() as u32;
        let cd_size = central.len() as u32;
        let n = files.len() as u16;
        out.extend_from_slice(&central);
        // EOCD
        out.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
        out.extend_from_slice(&0u16.to_le_bytes()); // disk
        out.extend_from_slice(&0u16.to_le_bytes()); // cd disk
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&cd_size.to_le_bytes());
        out.extend_from_slice(&cd_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // comment len
        out
    }

    /// same zip, but the payload is deflate-compressed (method 8) — the
    /// method every real-world pack zip actually uses
    fn build_deflate_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Read;
        let mut deflated: Vec<(String, Vec<u8>, Vec<u8>, u32)> = Vec::new();
        for (name, data) in files {
            let mut enc = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
            use std::io::Write;
            enc.write_all(data).unwrap();
            let comp = enc.finish().unwrap();
            let mut crc = flate2::Crc::new();
            crc.update(data);
            deflated.push((name.to_string(), comp, data.to_vec(), crc.sum()));
        }
        let mut out = Vec::new();
        let mut central = Vec::new();
        for (name, comp, data, crc) in &deflated {
            let offset = out.len() as u32;
            out.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
            out.extend_from_slice(&20u16.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&8u16.to_le_bytes()); // method: deflate
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&(comp.len() as u32).to_le_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(comp);
            central.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
            central.extend_from_slice(&20u16.to_le_bytes());
            central.extend_from_slice(&20u16.to_le_bytes());
            central.extend_from_slice(&0u16.to_le_bytes());
            central.extend_from_slice(&8u16.to_le_bytes()); // deflate
            central.extend_from_slice(&0u16.to_le_bytes());
            central.extend_from_slice(&0u16.to_le_bytes());
            central.extend_from_slice(&crc.to_le_bytes());
            central.extend_from_slice(&(comp.len() as u32).to_le_bytes());
            central.extend_from_slice(&(data.len() as u32).to_le_bytes());
            central.extend_from_slice(&(name.len() as u16).to_le_bytes());
            central.extend_from_slice(&0u16.to_le_bytes());
            central.extend_from_slice(&0u16.to_le_bytes());
            central.extend_from_slice(&0u16.to_le_bytes());
            central.extend_from_slice(&0u16.to_le_bytes());
            central.extend_from_slice(&0u32.to_le_bytes());
            central.extend_from_slice(&offset.to_le_bytes());
            central.extend_from_slice(name.as_bytes());
        }
        let cd_offset = out.len() as u32;
        let cd_size = central.len() as u32;
        let n = deflated.len() as u16;
        out.extend_from_slice(&central);
        out.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&cd_size.to_le_bytes());
        out.extend_from_slice(&cd_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out
    }

    #[test]
    fn stored_zip_roundtrip() {
        let zip = build_stored_zip(&[
            ("pack.mcmeta", b"{\"pack\":{\"pack_format\":6,\"description\":\"t\"}}"),
            ("data/demo/recipes/x.json", b"{\"type\":\"minecraft:crafting_shapeless\",\"ingredients\":[{\"item\":\"minecraft:stone\"}],\"result\":{\"item\":\"minecraft:cobblestone\"}}"),
        ]);
        let zf = ZipFiles::from_bytes(&zip).expect("zip parses");
        assert_eq!(zf.list("data/"), vec!["data/demo/recipes/x.json".to_string()]);
        assert!(zf.read("pack.mcmeta").is_some());
        assert!(zf.read("data/demo/recipes/x.json").is_some());
        assert!(zf.read("nope.json").is_none());
    }

    #[test]
    fn deflate_zip_roundtrip() {
        let zip = build_deflate_zip(&[
            ("pack.mcmeta", b"{\"pack\":{\"pack_format\":6}}"),
            ("data/demo/tags/items/t.json", b"{\"replace\":false,\"values\":[\"minecraft:bone\"]}"),
        ]);
        let zf = ZipFiles::from_bytes(&zip).expect("zip parses");
        let tag = zf.read("data/demo/tags/items/t.json").expect("deflate entry inflates");
        assert!(tag.starts_with(b"{\"replace\""));
        assert_eq!(zf.list("data/").len(), 1);
    }

    #[test]
    fn corrupt_and_garbage_inputs_rejected() {
        assert!(ZipFiles::from_bytes(b"").is_none());
        assert!(ZipFiles::from_bytes(b"not a zip at all, just text").is_none());
        // EOCD claiming a central directory past the end → None
        let mut zip = build_stored_zip(&[("a.txt", b"hello")]);
        let len = zip.len();
        zip[len - 6] = 0xFF; // cd_offset = huge
        zip[len - 5] = 0xFF;
        zip[len - 4] = 0xFF;
        zip[len - 3] = 0xFF;
        assert!(ZipFiles::from_bytes(&zip).is_none());
    }
}
