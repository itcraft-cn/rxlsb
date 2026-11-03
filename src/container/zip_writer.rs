//! 自研最小 ZIP 写入器。
//!
//! 仅支持 XLSB 需要的子集：deflate（或 stored）单盘、无注释、无加密。
//! 使用 `zlib-rs` 作为私有 deflate 后端（独立于 `flate2`，不会被 feature 统一），
//! 以获得更高的写入吞吐。读取仍由 `zip` crate 负责。
//!
//! 统一采用 data descriptor（flag bit 3）写入：本地头中的 crc/大小置零，真实值
//! 写在数据之后、中央目录使用真实值。这样可以在不预先知道大小的情况下**流式压缩**
//! 写入，避免为每个条目分配一整块压缩缓冲。
//!
//! 限制：单个条目或整体偏移超过 4 GiB 时返回错误（不支持 zip64）。

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use bytes::Bytes;
use zlib_rs::{crc32::crc32, Deflate, DeflateFlush, Status};

use crate::error::{Result, XlsbError};

/// 默认压缩级别（速度优先）。
pub const DEFAULT_COMPRESSION_LEVEL: i32 = 1;

/// 输出缓冲区大小。
const OUTPUT_BUFFER_SIZE: usize = 256 * 1024;
/// 压缩输出分块大小。
const CHUNK: usize = 64 * 1024;

const LOCAL_SIG: u32 = 0x0403_4b50;
const CENTRAL_SIG: u32 = 0x0201_4b50;
const EOCD_SIG: u32 = 0x0605_4b50;
const DESCRIPTOR_SIG: u32 = 0x0807_4b50;
/// DOS 日期：1980-01-01。
const DOS_DATE: u16 = 0x0021;
const VERSION: u16 = 20;
/// data descriptor 标志位。
const FLAG_DESCRIPTOR: u16 = 0x0008;

struct EntryMeta {
    crc: u32,
    comp_size: u32,
    uncomp_size: u32,
    offset: u32,
    method: u16,
}

/// 正在写入的条目状态（begin_entry 与 end_entry 之间）。
struct Pending {
    name: String,
    method: u16,
    offset: u32,
    crc: u32,
    uncomp: u64,
    comp: u64,
    prev_out: u64,
    deflate: Option<Deflate>,
}

pub struct XlsbContainerWriter {
    out: BufWriter<File>,
    offset: u64,
    names: Vec<String>,
    metas: Vec<EntryMeta>,
    pending: Option<Pending>,
    compression_level: i32,
    chunk: Box<[u8]>,
}

#[inline]
fn le16(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}

#[inline]
fn le32(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}

impl XlsbContainerWriter {
    pub fn create(path: &Path) -> Result<Self> {
        Self::create_with_level(path, DEFAULT_COMPRESSION_LEVEL)
    }

    pub fn create_with_level(path: &Path, compression_level: i32) -> Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            out: BufWriter::with_capacity(OUTPUT_BUFFER_SIZE, file),
            offset: 0,
            names: Vec::new(),
            metas: Vec::new(),
            pending: None,
            compression_level,
            chunk: vec![0u8; CHUNK].into_boxed_slice(),
        })
    }

    pub fn set_compression_level(&mut self, level: i32) {
        self.compression_level = level;
    }

    fn write_all_counted(&mut self, bytes: &[u8]) -> Result<()> {
        self.out.write_all(bytes)?;
        self.offset += bytes.len() as u64;
        Ok(())
    }

    /// 一次性写入条目（内部走 begin/chunk/end）。
    pub fn add_entry(&mut self, name: &str, data: &[u8]) -> Result<()> {
        self.begin_entry(name)?;
        self.write_entry_chunk(data)?;
        self.end_entry()
    }

    /// 开始一个条目（写入带 data descriptor 的本地头）。
    pub fn begin_entry(&mut self, name: &str) -> Result<()> {
        if self.pending.is_some() {
            return Err(XlsbError::Compression("previous entry not ended".into()));
        }
        let deflate = self.compression_level > 0;
        let method: u16 = if deflate { 8 } else { 0 };
        let offset = u32::try_from(self.offset)
            .map_err(|_| XlsbError::Compression("archive offset exceeds 4 GiB".into()))?;

        let name_bytes = name.as_bytes();
        let name_len = u16::try_from(name_bytes.len())
            .map_err(|_| XlsbError::Compression("entry name too long".into()))?;

        let mut header = Vec::with_capacity(30 + name_bytes.len());
        header.extend_from_slice(&le32(LOCAL_SIG));
        header.extend_from_slice(&le16(VERSION));
        header.extend_from_slice(&le16(FLAG_DESCRIPTOR));
        header.extend_from_slice(&le16(method));
        header.extend_from_slice(&le16(0)); // mod time
        header.extend_from_slice(&le16(DOS_DATE));
        header.extend_from_slice(&le32(0)); // crc (descriptor)
        header.extend_from_slice(&le32(0)); // comp size (descriptor)
        header.extend_from_slice(&le32(0)); // uncomp size (descriptor)
        header.extend_from_slice(&le16(name_len));
        header.extend_from_slice(&le16(0)); // extra len
        header.extend_from_slice(name_bytes);
        self.write_all_counted(&header)?;

        self.pending = Some(Pending {
            name: name.to_string(),
            method,
            offset,
            crc: 0,
            uncomp: 0,
            comp: 0,
            prev_out: 0,
            deflate: if deflate {
                Some(Deflate::new(self.compression_level, false, 15))
            } else {
                None
            },
        });
        Ok(())
    }

    /// 追加一段未压缩数据（会被压缩或原样写入）。
    pub fn write_entry_chunk(&mut self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }
        let Self { out, offset, pending, chunk, .. } = self;
        let p = pending
            .as_mut()
            .ok_or_else(|| XlsbError::Compression("entry not started".into()))?;
        p.crc = crc32(p.crc, data);
        p.uncomp += data.len() as u64;

        match p.deflate.as_mut() {
            None => {
                out.write_all(data)?;
                *offset += data.len() as u64;
                p.comp += data.len() as u64;
            }
            Some(deflate) => {
                let mut in_off = 0usize;
                while in_off < data.len() {
                    let status = deflate
                        .compress(&data[in_off..], chunk, DeflateFlush::NoFlush)
                        .map_err(|e| XlsbError::Compression(format!("{e:?}")))?;
                    in_off = deflate.total_in() as usize;
                    let total_out = deflate.total_out();
                    let produced = (total_out - p.prev_out) as usize;
                    p.prev_out = total_out;
                    if produced > 0 {
                        out.write_all(&chunk[..produced])?;
                        *offset += produced as u64;
                        p.comp += produced as u64;
                    }
                    if produced == 0 && status == Status::Ok {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// 结束当前条目（结束 deflate 流并写 data descriptor）。
    pub fn end_entry(&mut self) -> Result<()> {
        if self.pending.is_none() {
            return Err(XlsbError::Compression("entry not started".into()));
        }
        // 先结束 deflate 流（若有）。
        {
            let Self { out, offset, pending, chunk, .. } = self;
            let p = pending.as_mut().unwrap();
            if let Some(deflate) = p.deflate.as_mut() {
                loop {
                    let status = deflate
                        .compress(&[], chunk, DeflateFlush::Finish)
                        .map_err(|e| XlsbError::Compression(format!("{e:?}")))?;
                    let total_out = deflate.total_out();
                    let produced = (total_out - p.prev_out) as usize;
                    p.prev_out = total_out;
                    if produced > 0 {
                        out.write_all(&chunk[..produced])?;
                        *offset += produced as u64;
                        p.comp += produced as u64;
                    }
                    match status {
                        Status::StreamEnd => break,
                        _ if produced == 0 => break,
                        _ => {}
                    }
                }
            }
        }

        let p = self.pending.take().unwrap();
        let comp_size = u32::try_from(p.comp)
            .map_err(|_| XlsbError::Compression("entry compressed size exceeds 4 GiB".into()))?;
        let uncomp_size = u32::try_from(p.uncomp)
            .map_err(|_| XlsbError::Compression("entry size exceeds 4 GiB".into()))?;

        let mut desc = [0u8; 16];
        desc[0..4].copy_from_slice(&le32(DESCRIPTOR_SIG));
        desc[4..8].copy_from_slice(&le32(p.crc));
        desc[8..12].copy_from_slice(&le32(comp_size));
        desc[12..16].copy_from_slice(&le32(uncomp_size));
        self.write_all_counted(&desc)?;

        self.names.push(p.name);
        self.metas.push(EntryMeta {
            crc: p.crc,
            comp_size,
            uncomp_size,
            offset: p.offset,
            method: p.method,
        });
        Ok(())
    }

    pub fn add_entry_from_bytes(&mut self, name: &str, data: &Bytes) -> Result<()> {
        self.add_entry(name, data.as_ref())
    }

    pub fn add_entry_from_str(&mut self, name: &str, data: &str) -> Result<()> {
        self.add_entry(name, data.as_bytes())
    }

    pub fn finish(&mut self) -> Result<()> {
        if self.pending.is_some() {
            return Err(XlsbError::Compression("unfinished entry".into()));
        }
        let count = u16::try_from(self.metas.len())
            .map_err(|_| XlsbError::Compression("too many entries".into()))?;

        let cd_offset = u32::try_from(self.offset)
            .map_err(|_| XlsbError::Compression("central directory offset exceeds 4 GiB".into()))?;
        let mut cd: Vec<u8> = Vec::with_capacity(self.metas.len() * 64);

        for (meta, name) in self.metas.iter().zip(self.names.iter()) {
            let name_bytes = name.as_bytes();
            let name_len = u16::try_from(name_bytes.len())
                .map_err(|_| XlsbError::Compression("entry name too long".into()))?;

            cd.extend_from_slice(&le32(CENTRAL_SIG));
            cd.extend_from_slice(&le16(VERSION)); // version made by
            cd.extend_from_slice(&le16(VERSION)); // version needed
            cd.extend_from_slice(&le16(FLAG_DESCRIPTOR));
            cd.extend_from_slice(&le16(meta.method));
            cd.extend_from_slice(&le16(0)); // mod time
            cd.extend_from_slice(&le16(DOS_DATE));
            cd.extend_from_slice(&le32(meta.crc));
            cd.extend_from_slice(&le32(meta.comp_size));
            cd.extend_from_slice(&le32(meta.uncomp_size));
            cd.extend_from_slice(&le16(name_len));
            cd.extend_from_slice(&le16(0)); // extra
            cd.extend_from_slice(&le16(0)); // comment
            cd.extend_from_slice(&le16(0)); // disk start
            cd.extend_from_slice(&le16(0)); // internal attrs
            cd.extend_from_slice(&le32(0)); // external attrs
            cd.extend_from_slice(&le32(meta.offset));
            cd.extend_from_slice(name_bytes);
        }

        let cd_size = u32::try_from(cd.len())
            .map_err(|_| XlsbError::Compression("central directory size exceeds 4 GiB".into()))?;
        self.write_all_counted(&cd)?;

        let mut eocd = Vec::with_capacity(22);
        eocd.extend_from_slice(&le32(EOCD_SIG));
        eocd.extend_from_slice(&le16(0)); // this disk
        eocd.extend_from_slice(&le16(0)); // cd disk
        eocd.extend_from_slice(&le16(count)); // entries this disk
        eocd.extend_from_slice(&le16(count)); // total entries
        eocd.extend_from_slice(&le32(cd_size));
        eocd.extend_from_slice(&le32(cd_offset));
        eocd.extend_from_slice(&le16(0)); // comment len
        self.write_all_counted(&eocd)?;

        self.out.flush()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn entries(&self) -> &[String] {
        &self.names
    }

    #[allow(dead_code)]
    pub fn entry_count(&self) -> usize {
        self.names.len()
    }
}
