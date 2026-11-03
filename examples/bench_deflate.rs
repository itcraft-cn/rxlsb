use std::io::Write;
use std::time::Instant;

use flate2::{write::DeflateEncoder, Compression};
use zlib_rs::{compress_bound, Deflate, DeflateFlush, Status};

fn build_payload() -> Vec<u8> {
    // 近似 harness 的原始数据：重复波兰文本 + 数值流。
    let mut v = Vec::new();
    let tail = " z polskimi znakami: ąęśćńźółĄĘŚĆŃŹÓŁ oraz dłuższy tekst testowy.";
    for i in 0..5000u32 {
        v.extend_from_slice(format!("Produkt {i} żółć").as_bytes());
        v.extend_from_slice(format!("Opis produktu {i}{tail}").as_bytes());
        v.extend_from_slice(&((i as f64) * 1.5).to_le_bytes());
    }
    v
}

fn zlib_rs_deflate(input: &[u8], level: i32) -> usize {
    let mut out = vec![0u8; compress_bound(input.len())];
    let mut d = Deflate::new(level, false, 15);
    let mut in_off = 0;
    let mut out_off = 0;
    loop {
        let st = d
            .compress(&input[in_off..], &mut out[out_off..], DeflateFlush::Finish)
            .unwrap();
        in_off = d.total_in() as usize;
        out_off = d.total_out() as usize;
        match st {
            Status::StreamEnd => break,
            _ => {}
        }
    }
    out_off
}

fn flate2_deflate(input: &[u8], level: u32) -> usize {
    let mut e = DeflateEncoder::new(Vec::new(), Compression::new(level));
    e.write_all(input).unwrap();
    e.finish().unwrap().len()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let data = match args.next() {
        Some(path) => std::fs::read(&path).expect("read payload file"),
        None => build_payload(),
    };
    println!("payload = {} bytes", data.len());
    for level in [1i32, 4, 6, 9] {
        // zlib-rs
        let t = Instant::now();
        let n = zlib_rs_deflate(&data, level);
        let z = t.elapsed();

        // flate2 (miniz_oxide)
        let t = Instant::now();
        let m = flate2_deflate(&data, level as u32);
        let f = t.elapsed();

        println!(
            "level {level}: zlib-rs {:>7.2} ms ({} B) | flate2/miniz {:>7.2} ms ({} B)",
            z.as_secs_f64() * 1000.0,
            n,
            f.as_secs_f64() * 1000.0,
            m
        );
    }
}
