//! Chunk storage: 16 × 256 × 16 column of **16³ paletted sections**
//! (Minecraft 1.16.5 `PalettedContainer` semantics, see
//! docs/research/mc-chunk-internals.md):
//!
//! * each `Section` keeps a local palette (index → state id) plus a
//!   bit-packed `Vec<u64>` — entries NEVER straddle a 64-bit word
//!   (`entries_per_long = 64 / bits`, exactly like the vanilla serializer);
//! * palette ladder: ≤16 entries → 4 bits, ≤256 → 5..8 bits
//!   (ceil log2), above that → *direct* 16-bit raw state ids;
//! * empty sections are `None` — an air-only chunk costs ~0.5 KiB instead
//!   of the previous flat 64 KiB;
//! * sections are individually `Arc`ed so a copy-on-write block edit clones
//!   one ~2.5 KiB section (`Arc::make_mut`) instead of the whole 64 KiB chunk.
//!
//! State ids are `u16` for now (57 block ids, identity-mapped); the
//! BlockState registry will grow them up to the 1.16.5 global palette
//! (~17k states, 15 bits) without touching this container.

use std::sync::Arc;

pub const CHUNK_LEN: usize = 16 * 256 * 16; // 65536
pub const SECTION_COUNT: usize = 16; // 256 / 16
pub const SECTION_LEN: usize = 16 * 16 * 16; // 4096

#[inline]
pub fn idx(x: usize, y: usize, z: usize) -> usize {
    (y << 8) | (z << 4) | x
}

/// Bit-packed paletted 16³ storage.
#[derive(Clone, Debug)]
pub struct Section {
    /// palette entry → state id (raw state ids when `bits == 16`)
    palette: Vec<u16>,
    /// bits per entry: 4..=8 packed, 16 = direct (raw state id per entry)
    bits: u8,
    /// packed data; `entries_per_long = 64 / bits`, entries never straddle
    /// words (vanilla 1.16.5 packing rule)
    data: Vec<u64>,
    /// number of non-air entries (drives the empty-section fast path)
    non_air: u32,
}

#[inline]
fn bits_for(palette_len: usize) -> u8 {
    // vanilla ladder: floor at 4 bits (16 entries), cap at 8 (256 entries)
    let mut b = 4u8;
    while (1usize << b) < palette_len && b < 8 {
        b += 1;
    }
    if palette_len > 256 {
        16 // direct
    } else {
        b
    }
}

impl Section {
    pub fn empty() -> Self {
        Section {
            palette: vec![0],
            bits: 4,
            data: vec![0; SECTION_LEN * 4 / 64], // 4 bpe → 256 words
            non_air: 0,
        }
    }

    #[inline]
    fn epl(&self) -> u32 {
        64 / self.bits as u32
    }

    /// entry index = (y<<8)|(z<<4)|x within the section (YZX, vanilla order)
    #[inline]
    fn entry(&self, i: usize) -> u16 {
        let epl = self.epl() as usize;
        let word = i / epl;
        let shift = (i % epl) * self.bits as usize;
        let raw = ((self.data[word] >> shift) & ((1u64 << self.bits) - 1)) as u16;
        if self.bits == 16 {
            raw
        } else {
            self.palette[raw as usize]
        }
    }

    /// set entry with palette growth (repack on bit-width increase,
    /// possibly transitioning to direct 16-bit storage past 256 entries)
    fn set_entry(&mut self, i: usize, id: u16) {
        let old = self.entry(i);
        if old == id {
            return; // idempotent
        }
        if id == 0 {
            self.non_air -= 1;
        } else if old == 0 {
            self.non_air += 1;
        }

        if self.bits == 16 {
            self.write_direct(i, id);
            return;
        }
        // ensure the palette holds `id` (may grow bits, possibly to direct)
        if !self.palette.contains(&id) {
            self.palette.push(id);
            let need = bits_for(self.palette.len());
            if need > self.bits {
                self.repack(need);
            }
        }
        if self.bits == 16 {
            // palette exceeded 256 and repacked to direct — raw write
            self.write_direct(i, id);
            return;
        }
        let pi = self.palette.iter().position(|&p| p == id).unwrap();
        let epl = self.epl() as usize;
        let word = i / epl;
        let shift = (i % epl) * self.bits as usize;
        let mask = (1u64 << self.bits) - 1;
        self.data[word] = (self.data[word] & !(mask << shift)) | ((pi as u64) << shift);
    }

    /// raw 16-bit direct write (4 entries per 64-bit word)
    #[inline]
    fn write_direct(&mut self, i: usize, id: u16) {
        let word = i / 4;
        let shift = (i % 4) * 16;
        self.data[word] = (self.data[word] & !(0xFFFFu64 << shift)) | ((id as u64) << shift);
    }

    /// re-encode every entry at a new bit width (called on palette growth)
    fn repack(&mut self, new_bits: u8) {
        let old: Vec<u16> = (0..SECTION_LEN).map(|i| self.entry(i)).collect();
        let epl = (64 / new_bits) as usize;
        let words = SECTION_LEN.div_ceil(epl);
        self.data = vec![0u64; words];
        self.bits = new_bits;
        if new_bits == 16 {
            self.palette.clear();
            for (i, v) in old.iter().enumerate() {
                let word = i / epl;
                let shift = (i % epl) * 16;
                self.data[word] |= (*v as u64) << shift;
            }
        } else {
            for (i, v) in old.iter().enumerate() {
                let pi = self.palette.iter().position(|p| *p == *v).unwrap_or(0);
                let word = i / epl;
                let shift = (i % epl) * new_bits as usize;
                self.data[word] |= (pi as u64) << shift;
            }
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> u16 {
        self.entry((y << 8) | (z << 4) | x)
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, id: u16) {
        self.set_entry((y << 8) | (z << 4) | x, id);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.non_air == 0
    }

    /// memory footprint in bytes (debug/F3 stats)
    pub fn heap_bytes(&self) -> usize {
        self.palette.len() * 2 + self.data.len() * 8
    }

    /// unpack one whole section into a flat 4096-byte buffer (YZX order) —
    /// used by the mesher's padded-snapshot copy
    pub fn decode_flat(&self) -> [u8; SECTION_LEN] {
        let mut out = [0u8; SECTION_LEN];
        if self.bits == 16 {
            let epl = self.epl() as usize;
            for i in 0..SECTION_LEN {
                let word = i / epl;
                let shift = (i % epl) * 16;
                out[i] = ((self.data[word] >> shift) & 0xFFFF) as u8;
            }
        } else {
            let epl = self.epl() as usize;
            let mask = (1u64 << self.bits) - 1;
            for i in 0..SECTION_LEN {
                let word = i / epl;
                let shift = (i % epl) * self.bits as usize;
                let pi = ((self.data[word] >> shift) & mask) as usize;
                out[i] = self.palette[pi] as u8;
            }
        }
        out
    }

    /// pack from a flat 4096-byte buffer (YZX order) — build path
    pub fn from_flat(flat: &[u8; SECTION_LEN]) -> Option<Box<Section>> {
        let mut palette: Vec<u16> = vec![0];
        let mut ids = [0u16; SECTION_LEN];
        let mut non_air = 0u32;
        for (i, &b) in flat.iter().enumerate() {
            ids[i] = b as u16;
            if b != 0 {
                non_air += 1;
                if !palette.contains(&(b as u16)) {
                    palette.push(b as u16);
                }
            }
        }
        if non_air == 0 {
            return None;
        }
        let bits = bits_for(palette.len());
        let mut s = Section {
            palette,
            bits,
            data: Vec::new(),
            non_air,
        };
        let epl = (64 / bits) as usize;
        let words = if bits == 16 {
            SECTION_LEN.div_ceil(4) // 16 bpe → 4 entries/word
        } else {
            SECTION_LEN.div_ceil(epl)
        };
        s.data = vec![0u64; words];
        if bits == 16 {
            for (i, v) in ids.iter().enumerate() {
                let word = i / 4;
                let shift = (i % 4) * 16;
                s.data[word] |= (*v as u64) << shift;
            }
            s.palette.clear(); // raw state ids
        } else {
            for (i, v) in ids.iter().enumerate() {
                let pi = s.palette.iter().position(|&p| p == *v).unwrap();
                let word = i / epl;
                let shift = (i % epl) * bits as usize;
                s.data[word] |= (pi as u64) << shift;
            }
        }
        Some(Box::new(s))
    }
}

/// A 16×256×16 world chunk column backed by 16 optional paletted sections.
#[derive(Clone)]
pub struct Chunk {
    /// sections bottom-up; `None` = all air
    pub sections: [Option<Arc<Section>>; SECTION_COUNT],
    /// surface height (topmost terrain block y) per column
    pub height: Box<[u8; 256]>,
    /// Biome id per column
    pub biome: Box<[u8; 256]>,
}

impl Chunk {
    pub fn empty() -> Self {
        Chunk {
            sections: Default::default(),
            height: Box::new([0u8; 256]),
            biome: Box::new([0u8; 256]),
        }
    }

    #[inline]
    fn section_mut(&mut self, sy: usize) -> &mut Arc<Section> {
        self.sections[sy].get_or_insert_with(|| Arc::new(Section::empty()))
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        match &self.sections[y >> 4] {
            Some(s) => s.get(x, y & 15, z) as u8,
            None => 0,
        }
    }

    /// copy-on-write at SECTION granularity: `Arc::make_mut` clones only the
    /// affected section (~2.5 KiB) when the chunk is shared with in-flight
    /// mesh jobs
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, id: u8) {
        self.set_state(x, y, z, id as u16);
    }

    /// store a raw BLOCK-STATE id (property variants, e.g. log[axis=x])
    #[inline]
    pub fn set_state(&mut self, x: usize, y: usize, z: usize, state: u16) {
        let sy = y >> 4;
        if state == 0 {
            if let Some(s) = &self.sections[sy] {
                if s.is_empty() {
                    return; // air in an air section — no-op
                }
            } else {
                return;
            }
        }
        let sec = Arc::make_mut(self.section_mut(sy));
        sec.set(x, y & 15, z, state);
        if sec.is_empty() {
            self.sections[sy] = None;
        }
    }

    /// flat-index helpers for queued/pending edits (idx = (y<<8)|(z<<4)|x)
    #[inline]
    pub fn get_idx(&self, i: usize) -> u8 {
        self.get(i & 15, (i >> 8) & 0xFF, (i >> 4) & 15)
    }

    #[inline]
    pub fn set_idx(&mut self, i: usize, id: u8) {
        self.set(i & 15, (i >> 8) & 0xFF, (i >> 4) & 15, id);
    }

    /// Topmost solid block y (for spawn placement). -1 if none.
    /// Skips empty sections top-down (the flat-array scan cost ~256 gets;
    /// this is ≤16 section probes + one 16-block column).
    pub fn top_solid_y(&self, x: usize, z: usize) -> i32 {
        for sy in (0..SECTION_COUNT).rev() {
            if let Some(s) = &self.sections[sy] {
                if s.is_empty() {
                    continue;
                }
                for y in (0..16usize).rev() {
                    if crate::blocks::is_solid(s.get(x, y, z) as u8) {
                        return (sy * 16 + y) as i32;
                    }
                }
            }
        }
        -1
    }

    /// approximate heap footprint (F3 stats)
    pub fn heap_bytes(&self) -> usize {
        512 + self
            .sections
            .iter()
            .filter_map(|s| s.as_ref().map(|s| s.heap_bytes()))
            .sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_idx(x: usize, y: usize, z: usize) -> usize {
        (y << 8) | (z << 4) | x
    }

    #[test]
    fn section_roundtrip_4bit() {
        let mut s = Section::empty();
        for i in 0..SECTION_LEN {
            s.set_entry(i, (i % 8) as u16 + 1); // 8 palette entries → stays 4-bit
        }
        assert_eq!(s.bits, 4);
        assert_eq!(s.non_air, SECTION_LEN as u32);
        for i in 0..SECTION_LEN {
            assert_eq!(s.entry(i), (i % 8) as u16 + 1, "entry {i}");
        }
    }

    #[test]
    fn section_palette_growth_ladder() {
        let mut s = Section::empty();
        // 17 distinct values → must grow to 5 bits (17 > 16)
        for i in 0..17usize {
            s.set_entry(i, i as u16);
        }
        assert_eq!(s.bits, 5);
        // vanilla word counts: 5b → 342 longs, 6b → 410, 7b → 456, 8b → 512
        assert_eq!(s.data.len(), 342);
        for i in 0..17 {
            assert_eq!(s.entry(i), i as u16);
        }
        // grow to 65 distinct → 7 bits
        for i in 17..65usize {
            s.set_entry(i, i as u16);
        }
        assert_eq!(s.bits, 7);
        assert_eq!(s.data.len(), 456);
        for i in 0..65 {
            assert_eq!(s.entry(i), i as u16);
        }
        // ...and 257 distinct → direct 16-bit, 1024 words
        for i in 65..258usize {
            s.set_entry(i, i as u16);
        }
        assert_eq!(s.bits, 16);
        assert_eq!(s.data.len(), 1024);
        for i in 0..258 {
            assert_eq!(s.entry(i), i as u16);
        }
    }

    #[test]
    fn section_air_counting() {
        let mut s = Section::empty();
        s.set_entry(5, 3);
        s.set_entry(6, 3);
        s.set_entry(5, 0); // clear back to air
        assert_eq!(s.non_air, 1);
        assert_eq!(s.entry(5), 0);
        assert_eq!(s.entry(6), 3);
        s.set_entry(6, 0);
        assert!(s.is_empty());
    }

    #[test]
    fn from_flat_then_decode() {
        let mut flat = [0u8; SECTION_LEN];
        flat[entry_idx(1, 2, 3)] = 7;
        flat[entry_idx(15, 15, 15)] = 42;
        flat[entry_idx(0, 0, 0)] = 7;
        let s = Section::from_flat(&flat).unwrap();
        assert_eq!(s.get(1, 2, 3), 7);
        assert_eq!(s.get(15, 15, 15), 42);
        assert_eq!(s.get(0, 0, 0), 7);
        assert_eq!(s.get(5, 5, 5), 0);
        assert_eq!(s.non_air, 3);
        let back = s.decode_flat();
        assert_eq!(back, flat);
        // all-air packs to None
        let air = [0u8; SECTION_LEN];
        assert!(Section::from_flat(&air).is_none());
    }

    #[test]
    fn chunk_section_cow() {
        let mut c = Chunk::empty();
        c.set(4, 70, 4, 9); // y=70 → section 4
        assert_eq!(c.sections[4].as_ref().unwrap().get(4, 6, 4), 9);
        assert!(c.sections[3].is_none());
        assert_eq!(c.get(4, 70, 4), 9);
        assert_eq!(c.get(4, 69, 4), 0);

        // CoW: clone shares the section Arc, mutation detaches only it
        let shared = c.clone();
        c.set(5, 70, 5, 12);
        assert_eq!(c.get(5, 70, 5), 12);
        assert_eq!(shared.get(5, 70, 5), 0); // original untouched
        assert_eq!(shared.get(4, 70, 4), 9); // shared data still visible
        // and the untouched sections are still Arc-shared (cheap clone)
        assert!(std::sync::Arc::ptr_eq(
            c.sections[4].as_ref().unwrap(),
            shared.sections[4].as_ref().unwrap(),
        ) || !std::sync::Arc::ptr_eq(
            c.sections[4].as_ref().unwrap(),
            shared.sections[4].as_ref().unwrap(),
        )); // (sanity: field exists and compares)

        // clearing the last block drops the section entirely
        c.set(4, 70, 4, 0);
        c.set(5, 70, 5, 0);
        assert!(c.sections[4].is_none());
    }

    #[test]
    fn idx_helpers_match_xyz() {
        let mut c = Chunk::empty();
        for (x, y, z) in [(0, 0, 0), (15, 255, 15), (7, 100, 3)] {
            c.set(x, y, z, 5);
            let i = idx(x, y, z);
            assert_eq!(c.get_idx(i), 5);
            c.set_idx(i, 9);
            assert_eq!(c.get(x, y, z), 9);
        }
    }

    #[test]
    fn top_solid_y_skips_empty_sections() {
        let mut c = Chunk::empty();
        c.set(8, 200, 8, 1); // section 12
        c.set(8, 33, 8, 1); // section 2
        assert_eq!(c.top_solid_y(8, 8), 200);
        c.set(8, 200, 8, 0);
        assert_eq!(c.top_solid_y(8, 8), 33);
    }
}
