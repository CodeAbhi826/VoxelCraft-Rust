//! Named Binary Tag (NBT) codec — big-endian, Java edition semantics.
//!
//! Clean-room implementation of the publicly documented format
//! (https://minecraft.wiki/w/NBT_format). Used by the Anvil save system
//! (region chunks + `level.dat`).
//!
//! Design constraints:
//! * **no panics on malformed input** (Master Spec §46): every read is
//!   `Result`-based; truncated / oversize / over-deep data returns an
//!   error and the caller falls back to a safe default;
//! * owned tree (`String` / `Vec`) — chunks are tens of KB, simplicity
//!   beats zero-copy here (§0.3: no ideology-driven optimization);
//! * deterministic write order (insertion order) — byte-exact golden
//!   tests rely on it;
//! * compounds are `Vec<(String, Nbt)>` (order-preserving map);
//! * byte arrays are `Vec<i8>` (Java's *signed* bytes) so the wire
//!   representation round-trips exactly;
//! * strings: Java "modified UTF-8" differs from real UTF-8 only for
//!   U+0000 and supplementary code points — we write pure ASCII names,
//!   and read with `from_utf8_lossy` (block/flattening names are ASCII).
//!
//! Not implemented (out of scope, documented): TAG type 0 "TAG_End" is a
//! stream-only sentinel handled internally by list/compound readers.

use std::fmt;

/// Maximum nesting depth accepted by the reader (guards stack overflow;
/// real chunk data nests ~6 levels).
const MAX_DEPTH: u32 = 128;
/// Hard cap on a single string length (bytes) — malformed/corrupt files
/// must not cause huge allocations (§46).
const MAX_STRING: usize = 1 << 16;
/// Hard cap on any single array length (entries).
const MAX_ARRAY: usize = 1 << 24;

#[derive(Clone, Debug, PartialEq)]
pub enum Nbt {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    /// Java byte arrays: signed bytes.
    ByteArray(Vec<i8>),
    String(String),
    /// Homogeneous list — the codec accepts mixed lists on read (a
    /// corrupt-file tolerance) but always writes a valid tag id (mixed
    /// reads get the first element's type, empty → TAG_End-payload-free).
    List(Vec<Nbt>),
    Compound(Vec<(String, Nbt)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum NbtError {
    /// Ran out of bytes mid-tag.
    Truncated,
    /// Unknown tag id encountered.
    BadTagId(u8),
    /// Nesting deeper than `MAX_DEPTH`.
    TooDeep,
    /// Length prefix beyond `MAX_STRING`/`MAX_ARRAY`.
    Oversize,
    /// Root payload was not a compound (Anvil requires it).
    RootNotCompound,
    /// I/O layer failure (region file read etc.).
    Io(String),
}

impl fmt::Display for NbtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NbtError::Truncated => write!(f, "nbt: truncated input"),
            NbtError::BadTagId(t) => write!(f, "nbt: bad tag id {t}"),
            NbtError::TooDeep => write!(f, "nbt: nesting deeper than {MAX_DEPTH}"),
            NbtError::Oversize => write!(f, "nbt: oversize length prefix"),
            NbtError::RootNotCompound => write!(f, "nbt: root is not a compound"),
            NbtError::Io(e) => write!(f, "nbt io: {e}"),
        }
    }
}

impl std::error::Error for NbtError {}

// ------------------------------------------------------------------ ids --

pub const TAG_END: u8 = 0;
pub const TAG_BYTE: u8 = 1;
pub const TAG_SHORT: u8 = 2;
pub const TAG_INT: u8 = 3;
pub const TAG_LONG: u8 = 4;
pub const TAG_FLOAT: u8 = 5;
pub const TAG_DOUBLE: u8 = 6;
pub const TAG_BYTE_ARRAY: u8 = 7;
pub const TAG_STRING: u8 = 8;
pub const TAG_LIST: u8 = 9;
pub const TAG_COMPOUND: u8 = 10;
pub const TAG_INT_ARRAY: u8 = 11;
pub const TAG_LONG_ARRAY: u8 = 12;

impl Nbt {
    fn tag_id(&self) -> u8 {
        match self {
            Nbt::Byte(_) => TAG_BYTE,
            Nbt::Short(_) => TAG_SHORT,
            Nbt::Int(_) => TAG_INT,
            Nbt::Long(_) => TAG_LONG,
            Nbt::Float(_) => TAG_FLOAT,
            Nbt::Double(_) => TAG_DOUBLE,
            Nbt::ByteArray(_) => TAG_BYTE_ARRAY,
            Nbt::String(_) => TAG_STRING,
            Nbt::List(_) => TAG_LIST,
            Nbt::Compound(_) => TAG_COMPOUND,
            Nbt::IntArray(_) => TAG_INT_ARRAY,
            Nbt::LongArray(_) => TAG_LONG_ARRAY,
        }
    }

    // -------- compound helpers (insertion-order map) --------

    pub fn compound() -> Nbt {
        Nbt::Compound(Vec::new())
    }

    pub fn set(&mut self, key: &str, value: Nbt) {
        if let Nbt::Compound(entries) = self {
            if let Some(slot) = entries.iter_mut().find(|(k, _)| k == key) {
                slot.1 = value;
            } else {
                entries.push((key.to_string(), value));
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&Nbt> {
        if let Nbt::Compound(entries) = self {
            entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
        } else {
            None
        }
    }

    /// Fetch a numeric tag as i64 (Byte/Short/Int/Long coerce).
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Nbt::Byte(v) => Some(*v as i64),
            Nbt::Short(v) => Some(*v as i64),
            Nbt::Int(v) => Some(*v as i64),
            Nbt::Long(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Nbt::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&[Nbt]> {
        if let Nbt::List(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn as_i64_slice(&self) -> Option<&[i64]> {
        if let Nbt::LongArray(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn as_i32_slice(&self) -> Option<&[i32]> {
        if let Nbt::IntArray(l) = self {
            Some(l)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------- writer --

/// Serialize a named root (file form: `TAG_Compound`, name, payload).
/// The root must be a compound — Anvil/level.dat always is.
pub fn write_root(root_name: &str, root: &Nbt) -> Result<Vec<u8>, NbtError> {
    let Nbt::Compound(_) = root else {
        return Err(NbtError::RootNotCompound);
    };
    let mut out = Vec::with_capacity(4096);
    out.push(TAG_COMPOUND);
    write_string(&mut out, root_name)?;
    write_payload(&mut out, root, 0)?;
    Ok(out)
}

fn write_string(out: &mut Vec<u8>, s: &str) -> Result<(), NbtError> {
    let bytes = s.as_bytes();
    if bytes.len() > MAX_STRING {
        return Err(NbtError::Oversize);
    }
    out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}

fn write_payload(out: &mut Vec<u8>, tag: &Nbt, depth: u32) -> Result<(), NbtError> {
    if depth > MAX_DEPTH {
        return Err(NbtError::TooDeep);
    }
    match tag {
        Nbt::Byte(v) => out.push(*v as u8),
        Nbt::Short(v) => out.extend_from_slice(&v.to_be_bytes()),
        Nbt::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
        Nbt::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
        Nbt::Float(v) => out.extend_from_slice(&v.to_be_bytes()),
        Nbt::Double(v) => out.extend_from_slice(&v.to_be_bytes()),
        Nbt::ByteArray(a) => {
            write_len_prefix(out, a.len())?;
            for &b in a {
                out.push(b as u8);
            }
        }
        Nbt::String(s) => write_string(out, s)?,
        Nbt::List(items) => {
            let id = items.first().map(|t| t.tag_id()).unwrap_or(TAG_END);
            out.push(id);
            write_len_prefix(out, items.len())?;
            for item in items {
                if item.tag_id() != id && id != TAG_END {
                    // mixed list on the write path: encode each element as a
                    // fully named payload is NOT valid NBT — instead fail
                    // loudly (callers build homogeneous lists)
                    return Err(NbtError::BadTagId(item.tag_id()));
                }
                if id == TAG_END {
                    // writing an empty list as [0][len=0] is already done;
                    // a non-empty list containing only End payloads cannot
                    // round-trip — reject
                    return Err(NbtError::BadTagId(TAG_END));
                }
                write_payload(out, item, depth + 1)?;
            }
        }
        Nbt::Compound(entries) => {
            for (k, v) in entries {
                out.push(v.tag_id());
                write_string(out, k)?;
                write_payload(out, v, depth + 1)?;
            }
            out.push(TAG_END);
        }
        Nbt::IntArray(a) => {
            write_len_prefix(out, a.len())?;
            for v in a {
                out.extend_from_slice(&v.to_be_bytes());
            }
        }
        Nbt::LongArray(a) => {
            write_len_prefix(out, a.len())?;
            for v in a {
                out.extend_from_slice(&v.to_be_bytes());
            }
        }
    }
    Ok(())
}

fn write_len_prefix(out: &mut Vec<u8>, len: usize) -> Result<(), NbtError> {
    if len > MAX_ARRAY {
        return Err(NbtError::Oversize);
    }
    out.extend_from_slice(&(len as i32).to_be_bytes());
    Ok(())
}

// ---------------------------------------------------------------- reader --

/// Reader cursor (immutable borrow, bounds-checked).
struct Cur<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cur<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], NbtError> {
        let end = self.pos.checked_add(n).ok_or(NbtError::Truncated)?;
        if end > self.data.len() {
            return Err(NbtError::Truncated);
        }
        let s = &self.data[self.pos..end];
        self.pos = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, NbtError> {
        Ok(self.take(1)?[0])
    }
    fn be_i16(&mut self) -> Result<i16, NbtError> {
        let b = self.take(2)?;
        Ok(i16::from_be_bytes([b[0], b[1]]))
    }
    fn be_i32(&mut self) -> Result<i32, NbtError> {
        let b = self.take(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn be_i64(&mut self) -> Result<i64, NbtError> {
        let b = self.take(8)?;
        Ok(i64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }
    fn string(&mut self) -> Result<String, NbtError> {
        let len = self.be_i16()? as usize;
        if len > MAX_STRING {
            return Err(NbtError::Oversize);
        }
        let bytes = self.take(len)?;
        // Java modified-UTF-8 ≈ UTF-8 for ASCII names; lossy keeps us
        // panic-free on hostile bytes (§46)
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }
    fn array_len(&mut self) -> Result<usize, NbtError> {
        let n = self.be_i32()?;
        if n < 0 || n as usize > MAX_ARRAY {
            return Err(NbtError::Oversize);
        }
        Ok(n as usize)
    }
}

/// Parse a named root; returns `(root_name, tag)`.
pub fn read_root(data: &[u8]) -> Result<(String, Nbt), NbtError> {
    let mut cur = Cur { data, pos: 0 };
    let id = cur.u8()?;
    if id != TAG_COMPOUND {
        return Err(NbtError::RootNotCompound);
    }
    let name = cur.string()?;
    let tag = read_payload(&mut cur, TAG_COMPOUND, 0)?;
    // trailing bytes are tolerated (padding / concatenated streams)
    Ok((name, tag))
}

fn read_payload(cur: &mut Cur, tag: u8, depth: u32) -> Result<Nbt, NbtError> {
    if depth > MAX_DEPTH {
        return Err(NbtError::TooDeep);
    }
    Ok(match tag {
        TAG_BYTE => Nbt::Byte(cur.u8()? as i8),
        TAG_SHORT => Nbt::Short(cur.be_i16()?),
        TAG_INT => Nbt::Int(cur.be_i32()?),
        TAG_LONG => Nbt::Long(cur.be_i64()?),
        TAG_FLOAT => Nbt::Float(f32::from_bits(cur.be_i32()? as u32)),
        TAG_DOUBLE => Nbt::Double(f64::from_bits(cur.be_i64()? as u64)),
        TAG_BYTE_ARRAY => {
            let n = cur.array_len()?;
            let bytes = cur.take(n)?;
            Nbt::ByteArray(bytes.iter().map(|&b| b as i8).collect())
        }
        TAG_STRING => Nbt::String(cur.string()?),
        TAG_LIST => {
            let id = cur.u8()?;
            let n = cur.array_len()?;
            if id == TAG_END {
                // empty list — n must be 0 for well-formed data; tolerate
                // nonzero by returning an empty list (data loss beats error
                // for corrupt-file resilience — §46)
                return Ok(Nbt::List(Vec::new()));
            }
            let mut items = Vec::with_capacity(n.min(4096));
            for _ in 0..n {
                items.push(read_payload(cur, id, depth + 1)?);
            }
            Nbt::List(items)
        }
        TAG_COMPOUND => {
            let mut entries = Vec::new();
            loop {
                let id = cur.u8()?;
                if id == TAG_END {
                    break;
                }
                let key = cur.string()?;
                let value = read_payload(cur, id, depth + 1)?;
                entries.push((key, value));
            }
            Nbt::Compound(entries)
        }
        TAG_INT_ARRAY => {
            let n = cur.array_len()?;
            let mut a = Vec::with_capacity(n.min(4096));
            for _ in 0..n {
                a.push(cur.be_i32()?);
            }
            Nbt::IntArray(a)
        }
        TAG_LONG_ARRAY => {
            let n = cur.array_len()?;
            let mut a = Vec::with_capacity(n.min(4096));
            for _ in 0..n {
                a.push(cur.be_i64()?);
            }
            Nbt::LongArray(a)
        }
        other => return Err(NbtError::BadTagId(other)),
    })
}

// ------------------------------------------------------------------ tests --

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Nbt {
        let mut level = Nbt::compound();
        level.set("DataVersion", Nbt::Int(2586));
        level.set("Name", Nbt::String("minecraft:stone".into()));
        level.set("Long", Nbt::Long(-0x0102_0304_0506_0708));
        level.set("Float", Nbt::Float(3.5));
        level.set("Double", Nbt::Double(-1.25));
        level.set("Bytes", Nbt::ByteArray(vec![0, 127, -128, -1]));
        level.set("Ints", Nbt::IntArray(vec![i32::MIN, 0, i32::MAX]));
        level.set("Longs", Nbt::LongArray(vec![i64::MIN, i64::MAX]));
        let mut inner = Nbt::compound();
        inner.set("nested", Nbt::Short(-300));
        inner.set("flag", Nbt::Byte(1));
        level.set("Inner", inner);
        let list = Nbt::List(vec![
            Nbt::String("a".into()),
            Nbt::String("b".into()),
            Nbt::String("".into()),
        ]);
        level.set("StrList", list);
        let empty = Nbt::List(vec![]);
        level.set("EmptyList", empty);
        level
    }

    #[test]
    fn roundtrip_all_types() {
        let root = sample();
        let bytes = write_root("Level", &root).unwrap();
        let (name, back) = read_root(&bytes).unwrap();
        assert_eq!(name, "Level");
        assert_eq!(back, root);
    }

    #[test]
    fn root_name_and_layout() {
        // exact wire form of a tiny root: tag, name, one Int payload, End
        let mut root = Nbt::compound();
        root.set("x", Nbt::Int(7));
        let bytes = write_root("", &root).unwrap();
        assert_eq!(
            bytes,
            vec![
                TAG_COMPOUND, 0, 0, // root name ""
                TAG_INT, 0, 1, b'x', // key "x"
                0, 0, 0, 7, TAG_END
            ]
        );
    }

    #[test]
    fn list_payload_has_no_keys() {
        // lists encode element payloads WITHOUT key strings (wire check)
        let list = Nbt::List(vec![Nbt::Int(1), Nbt::Int(2)]);
        let mut root = Nbt::compound();
        root.set("L", list);
        let bytes = write_root("r", &root).unwrap();
        // r(0,1) TAG_LIST(9) "L"(0,1,'L') elemType=3 len=2 then 8 raw bytes
        assert_eq!(
            bytes,
            vec![
                TAG_COMPOUND, 0, 1, b'r', TAG_LIST, 0, 1, b'L', TAG_INT,
                0, 0, 0, 2, // list: type Int, 2 entries
                0, 0, 0, 1, 0, 0, 0, 2, // payloads, no keys
                TAG_END, TAG_END
            ]
        );
    }

    #[test]
    fn truncation_is_an_error_never_a_panic() {
        let bytes = write_root("Level", &sample()).unwrap();
        for cut in 0..bytes.len() {
            // every prefix must either parse (if it happens to be complete)
            // or error — never panic
            let _ = read_root(&bytes[..cut]);
        }
        // single garbage byte
        let _ = read_root(&[0xFF]);
        // unknown tag id in root
        let _ = read_root(&[13, 0, 1, b'x']);
        // deep nesting guard: 200 nested lists
        let mut deep = Nbt::List(vec![Nbt::List(vec![])]);
        for _ in 0..200 {
            deep = Nbt::List(vec![deep]);
        }
        let mut root = Nbt::compound();
        root.set("d", deep);
        let b = write_root("r", &root);
        match b {
            Ok(bytes) => {
                assert!(matches!(read_root(&bytes), Err(NbtError::TooDeep)));
            }
            Err(e) => assert_eq!(e, NbtError::TooDeep),
        }
    }

    #[test]
    fn compound_set_overwrites_in_place() {
        let mut c = Nbt::compound();
        c.set("a", Nbt::Int(1));
        c.set("b", Nbt::Int(2));
        c.set("a", Nbt::Int(3)); // overwrite, order preserved
        let Nbt::Compound(entries) = &c else { panic!() };
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].1, Nbt::Int(3));
        assert_eq!(c.get("b").and_then(|v| v.as_i64()), Some(2));
    }

    #[test]
    fn long_array_extremes() {
        let mut root = Nbt::compound();
        root.set("a", Nbt::LongArray(vec![-1, 0, u64::MAX as i64]));
        let bytes = write_root("r", &root).unwrap();
        let (_, back) = read_root(&bytes).unwrap();
        assert_eq!(back.get("a").unwrap().as_i64_slice().unwrap(), &[-1i64, 0, -1i64]);
    }

    #[test]
    fn mixed_list_write_is_rejected() {
        let mut root = Nbt::compound();
        root.set("bad", Nbt::List(vec![Nbt::Int(1), Nbt::String("x".into())]));
        assert!(write_root("r", &root).is_err());
    }
}
