use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use crate::io::{BufferWriter, BufferReader};
use crate::format::RecordType;
use bytes::Bytes;
use crate::error::Result;

/// 恒等哈希器：键本身已是 FNV-1a 得到的 64 位哈希，无需再哈希。
#[derive(Default)]
struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // 兜底：非 u64 键（不会用到，但保持正确性）。
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.0 = h;
    }
    #[inline]
    fn write_u64(&mut self, v: u64) {
        self.0 = v;
    }
}

type IndexMap = HashMap<u64, u32, BuildHasherDefault<IdentityHasher>>;

const CHUNK_FLUSH: usize = 64 * 1024;

/// 轻量 64 位哈希（FNV-1a）。用于 SST 去重索引，避免引入额外依赖。
#[inline]
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[inline]
fn push_varint(buf: &mut Vec<u8>, value: u32) {
    if value >= 128 {
        buf.push(((value & 0x7F) as u8) | 0x80);
        buf.push(((value >> 7) as u8) & 0x7F);
    } else {
        buf.push(value as u8);
    }
}

#[inline]
fn push_varsize(buf: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// 高效的 UTF-8 → UTF-16 追加。ASCII 走快路径，避免 `encode_utf16` 迭代器开销。
#[inline]
fn encode_utf16_into(s: &str, out: &mut Vec<u16>) {
    out.reserve(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b < 0x80 {
            out.push(b as u16);
            i += 1;
        } else {
            // i 始终位于字符边界。
            let ch = s[i..].chars().next().unwrap();
            let cp = ch as u32;
            if cp <= 0xFFFF {
                out.push(cp as u16);
            } else {
                let v = cp - 0x10000;
                out.push(0xD800 + (v >> 10) as u16);
                out.push(0xDC00 + (v & 0x3FF) as u16);
            }
            i += ch.len_utf8();
        }
    }
}

/// 共享字符串表（SST）。
///
/// 写入路径以「移动」方式接收 `String`，每个唯一字符串只分配一次；
/// 去重索引使用哈希 + 内容校验，既避免二次拷贝，也保证不会误合并不同字符串。
#[derive(Clone)]
pub struct SstTable {
    strings: Vec<String>,
    /// FNV 哈希 -> 首个匹配索引（仅写入路径需要；读取构建时留空）。
    index: IndexMap,
    total_count: u32,
}

impl SstTable {
    pub fn new() -> Self {
        Self { strings: vec![], index: IndexMap::default(), total_count: 0 }
    }

    #[allow(dead_code)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            strings: Vec::with_capacity(capacity),
            index: IndexMap::with_capacity_and_hasher(capacity, Default::default()),
            total_count: 0,
        }
    }

    fn locate(&self, s: &str) -> Option<u32> {
        let hash = fnv1a(s.as_bytes());
        if let Some(&idx) = self.index.get(&hash) {
            if self.strings.get(idx as usize).map(|v| v.as_str()) == Some(s) {
                return Some(idx);
            }
        }
        None
    }

    pub fn add_string(&mut self, s: &str) -> u32 {
        self.total_count += 1;
        if let Some(idx) = self.locate(s) {
            return idx;
        }
        let idx = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.index.insert(fnv1a(s.as_bytes()), idx);
        idx
    }

    /// 以「所有权」方式加入字符串：直接把 `String` 移入表内，避免再拷贝一次。
    pub fn add_string_owned(&mut self, s: String) -> u32 {
        self.total_count += 1;
        let hash = fnv1a(s.as_bytes());
        if let Some(&idx) = self.index.get(&hash) {
            if self.strings.get(idx as usize).map(|v| v.as_str()) == Some(s.as_str()) {
                return idx;
            }
        }
        let idx = self.strings.len() as u32;
        self.strings.push(s);
        self.index.insert(hash, idx);
        idx
    }

    pub fn get_string(&self, idx: u32) -> Option<&str> {
        self.strings.get(idx as usize).map(|s| s.as_str())
    }

    pub fn find_string(&self, s: &str) -> Option<u32> {
        self.locate(s)
    }

    pub fn count(&self) -> usize { self.strings.len() }
    pub fn total_count(&self) -> u32 { self.total_count }

    /// 预留容量，减少写入过程中的 rehash 与扩容。
    pub fn reserve(&mut self, additional: usize) {
        self.strings.reserve(additional);
        self.index.reserve(additional);
    }

    pub fn serialize(&self) -> Result<Bytes> {
        let mut out = BufferWriter::new(64);
        let mut sink = |chunk: &[u8]| {
            out.write_bytes(chunk);
            Ok(())
        };
        self.serialize_chunks(&mut sink)?;
        Ok(out.into_bytes())
    }

    /// 以分块方式序列化，供容器流式压缩写入，避免构建整块中间缓冲。
    pub fn serialize_chunks(
        &self,
        sink: &mut dyn FnMut(&[u8]) -> Result<()>,
    ) -> Result<()> {
        let mut buf: Vec<u8> = Vec::with_capacity(CHUNK_FLUSH);

        // BrtBeginSst
        push_varint(&mut buf, RecordType::BrtBeginSst.to_u32());
        push_varsize(&mut buf, 8);
        buf.extend_from_slice(&self.total_count.to_le_bytes());
        buf.extend_from_slice(&(self.strings.len() as u32).to_le_bytes());

        let mut utf16: Vec<u16> = Vec::new();
        for s in &self.strings {
            utf16.clear();
            encode_utf16_into(s, &mut utf16);
            let record_size = 1 + 4 + utf16.len() * 2;

            push_varint(&mut buf, RecordType::BrtSstItem.to_u32());
            push_varsize(&mut buf, record_size as u32);
            buf.push(0);
            buf.extend_from_slice(&(utf16.len() as u32).to_le_bytes());
            #[cfg(target_endian = "little")]
            {
                // SAFETY: 小端平台 [u16] 的内存布局即 UTF-16LE 字节序列。
                let bytes = unsafe {
                    std::slice::from_raw_parts(utf16.as_ptr() as *const u8, utf16.len() * 2)
                };
                buf.extend_from_slice(bytes);
            }
            #[cfg(not(target_endian = "little"))]
            {
                for &c in &utf16 {
                    buf.extend_from_slice(&c.to_le_bytes());
                }
            }

            if buf.len() >= CHUNK_FLUSH {
                sink(&buf)?;
                buf.clear();
            }
        }

        push_varint(&mut buf, RecordType::BrtEndSst.to_u32());
        push_varsize(&mut buf, 0);
        sink(&buf)?;
        Ok(())
    }

    pub fn deserialize(data: Bytes) -> Result<Self> {
        let mut reader = BufferReader::new(data);
        let mut strings: Vec<String> = Vec::new();

        while reader.has_remaining() {
            let record_type_code = reader.read_varint()?;
            let size = reader.read_varsize()?;

            let record_type = RecordType::from_u32(record_type_code);

            match record_type {
                Some(RecordType::BrtBeginSst) => {
                    reader.skip(size as usize)?;
                }

                Some(RecordType::BrtSstItem) => {
                    reader.skip(1)?;
                    let char_count = reader.read_u32_le()? as usize;
                    let mut chars = Vec::with_capacity(char_count);
                    for _ in 0..char_count {
                        chars.push(reader.read_u16_le()?);
                    }
                    let s = String::from_utf16(&chars)
                        .map_err(|_| crate::error::XlsbError::InvalidUtf16)?;
                    strings.push(s);
                }

                Some(RecordType::BrtEndSst) => {
                    break;
                }

                _ => {
                    reader.skip(size as usize)?;
                }
            }
        }

        let total_count = strings.len() as u32;

        Ok(Self {
            strings,
            index: IndexMap::default(),
            total_count,
        })
    }
}
