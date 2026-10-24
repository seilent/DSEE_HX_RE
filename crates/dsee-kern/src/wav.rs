use crate::error::{DseeError, Result};
use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

pub fn normalize_bits(bits: &str) -> Result<u16> {
    match bits.trim() {
        "16" => Ok(16),
        "24" => Ok(24),
        "32" => Ok(32),
        other => Err(DseeError::Bits(other.to_string())),
    }
}

#[inline]
fn encode_one(bps: u16, x: f32, out: &mut Vec<u8>) {
    match bps {
        16 => {
            let v = (x.clamp(-1.0, 1.0) as f64 * 32767.0).round() as i32;
            let v = v.clamp(-32768, 32767) as i16;
            out.extend_from_slice(&v.to_le_bytes());
        }
        24 => {
            let v = (x.clamp(-1.0, 1.0) as f64 * 8_388_607.0).round() as i32;
            let v = v.clamp(-8_388_608, 8_388_607) as u32;
            out.push((v & 0xff) as u8);
            out.push(((v >> 8) & 0xff) as u8);
            out.push(((v >> 16) & 0xff) as u8);
        }
        32 => {
            let v = (x.clamp(-1.0, 1.0) as f64 * 2_147_483_647.0).round() as i64;
            let v = v.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
            out.extend_from_slice(&v.to_le_bytes());
        }
        _ => unreachable!(),
    }
}

fn bytes_per_sample(bps: u16) -> usize {
    match bps {
        16 => 2,
        24 => 3,
        32 => 4,
        _ => unreachable!(),
    }
}

fn write_header(w: &mut impl Write, rate: u32, bps: u16, data_len: u32) -> Result<()> {
    let nch = 2u16;
    let block = nch as u32 * bytes_per_sample(bps) as u32;
    w.write_all(b"RIFF")?;
    w.write_all(&(36 + data_len).to_le_bytes())?;
    w.write_all(b"WAVE")?;
    w.write_all(b"fmt ")?;
    w.write_all(&16u32.to_le_bytes())?;
    w.write_all(&1u16.to_le_bytes())?;
    w.write_all(&nch.to_le_bytes())?;
    w.write_all(&rate.to_le_bytes())?;
    w.write_all(&(rate * block).to_le_bytes())?;
    w.write_all(&(block as u16).to_le_bytes())?;
    w.write_all(&bps.to_le_bytes())?;
    w.write_all(b"data")?;
    w.write_all(&data_len.to_le_bytes())?;
    Ok(())
}

/// Streaming WAV writer: header with placeholder size, patch on finish.
pub struct WavWriter {
    file: BufWriter<File>,
    bps: u16,
    data_len: u32,
    scratch: Vec<u8>,
}

impl WavWriter {
    pub fn create(path: &Path, rate: u32, bits: &str) -> Result<Self> {
        let bps = normalize_bits(bits)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = BufWriter::with_capacity(256 * 1024, File::create(path)?);
        write_header(&mut file, rate, bps, 0)?;
        Ok(Self {
            file,
            bps,
            data_len: 0,
            scratch: Vec::with_capacity(64 * 1024),
        })
    }

    /// Append interleaved float samples (same quantization as `write_wav`).
    pub fn write_samples(&mut self, samples: &[f32]) -> Result<()> {
        self.scratch.clear();
        let need = samples.len() * bytes_per_sample(self.bps);
        if self.scratch.capacity() < need {
            self.scratch.reserve(need - self.scratch.capacity());
        }
        for &x in samples {
            encode_one(self.bps, x, &mut self.scratch);
        }
        self.file.write_all(&self.scratch)?;
        self.data_len = self
            .data_len
            .checked_add(self.scratch.len() as u32)
            .ok_or_else(|| DseeError::Msg("WAV data too large".into()))?;
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.file.flush()?;
        let mut f = self
            .file
            .into_inner()
            .map_err(|e| DseeError::Io(e.into_error()))?;
        f.seek(SeekFrom::Start(4))?;
        f.write_all(&(36 + self.data_len).to_le_bytes())?;
        f.seek(SeekFrom::Start(40))?;
        f.write_all(&self.data_len.to_le_bytes())?;
        Ok(())
    }
}

/// `samples`: interleaved stereo, float in [-1, 1]
pub fn write_wav(path: &Path, rate: u32, samples: &[f32], bits: &str) -> Result<()> {
    let mut w = WavWriter::create(path, rate, bits)?;
    w.write_samples(samples)?;
    w.finish()
}
