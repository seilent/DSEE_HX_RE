//! cargo run -p dsee-kern --example process_one -- <in> <out.wav> <rate> <bits>

use dsee_kern::{process_file, set_ffmpeg_override, ProcessRequest};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let raw: Vec<String> = env::args().skip(1).collect();
    if raw.len() != 4 {
        eprintln!("usage: process_one <input> <output.wav> <rate> <bits>");
        std::process::exit(2);
    }
    let input = PathBuf::from(&raw[0]);
    let out = PathBuf::from(&raw[1]);
    let rate: i32 = raw[2].parse().expect("rate");
    let bits = raw[3].as_str();

    let ff = PathBuf::from(
        r"C:\Users\Le_Fa\Desktop\DSEE HX\DSEE_HX_RE\release\DSEE_HX_RE_v1.0.0\ffmpeg-master-latest-win64-gpl-shared\bin\ffmpeg.exe",
    );
    if ff.is_file() {
        set_ffmpeg_override(Some(ff));
    }

    eprintln!("in={}", input.display());
    eprintln!("out={}", out.display());
    let t0 = Instant::now();
    let report = process_file(&ProcessRequest {
        input: &input,
        output: &out,
        output_rate: rate,
        output_bits: bits,
        enable: true,
    })
    .unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });
    eprintln!(
        "ok rate {}->{} mode={} f76={} f80={} hops={} samples={} {:.2}s",
        report.input_rate,
        report.wav_rate,
        report.mode,
        report.f76,
        report.f80,
        report.hops,
        report.samples_out,
        t0.elapsed().as_secs_f64()
    );
}
