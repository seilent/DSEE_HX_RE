//! FFI to the vendored C kernel (`native/kern` + `native/config`).
#![allow(non_camel_case_types, dead_code)]

use std::os::raw::{c_double, c_float, c_int};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DseeCfg {
    pub f76: c_int,
    pub f80: c_int,
    pub upmode: c_int,
    pub src: c_int,
    pub dst: c_int,
    pub step_src: c_int,
    pub step_dst: c_int,
    pub n12: c_int,
    pub f18992: c_int,
    pub nblk: c_int,
    pub off_up: c_int,
    pub off_low: c_int,
    pub channels: c_int,
    pub nch_gate: c_int,
    pub st1: c_int,
    pub active: c_int,
    pub rs_in: c_int,
    pub rs_out: c_int,
    pub rate: c_int,
    pub gain_a: c_float,
    pub gain_b: c_float,
    pub gain_c: c_float,
    pub gain_e: c_float,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DseeSource {
    pub mode: c_int,
    pub channels: c_int,
    pub codec: c_int,
    pub input_fs: c_int,
    pub output_fs: c_int,
    pub input_bits: c_int,
    pub output_bits: c_int,
    pub vbr: c_int,
    pub abr: c_int,
    pub sb: c_int,
    pub head: c_int,
    pub dcr: c_int,
    pub gain: c_int,
    pub offset: c_int,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DseeMediaFacts {
    pub rate: c_int,
    pub channels: c_int,
    pub bits: c_int,
    pub bit_rate: c_int,
    pub codec_name: [u8; 64],
    pub path_has_cbr: c_int,
}

unsafe extern "C" {
    pub fn dsee_mem_size() -> c_int;
    pub fn dsee_mode(enable: c_int, device_rate: c_int, codec: c_int) -> c_int;
    pub fn dsee_open_source(
        mem: *mut u8,
        mem_size: c_int,
        s: *const DseeSource,
        out: *mut DseeCfg,
    ) -> c_int;
    pub fn dsee_heq_delays(rate: c_int, delay: *mut c_int) -> c_int;
    pub fn dsee_heq_gain(bits: c_int, gain: *mut c_float) -> c_int;
    pub fn dsee_heq(
        ring0: *mut c_float,
        ring1: *mut c_float,
        hist0: *mut c_float,
        hist1: *mut c_float,
        lag: *mut c_int,
        hidx: *mut c_int,
        delay: *const c_int,
        gain: c_double,
        inn: *const i32,
        out: *mut i32,
        nframe: c_int,
    );
    pub fn dsee_process_i32(
        mem: *mut u8,
        ring0: *mut c_float,
        ring1: *mut c_float,
        hist0: *mut c_float,
        hist1: *mut c_float,
        lag: *mut c_int,
        hidx: *mut c_int,
        delay: *const c_int,
        gain: c_double,
        pcm: *const i32,
        hops: c_int,
        n_in: c_int,
        n_out: c_int,
        n_in_ch: c_int,
        n_out_ch: c_int,
        out: *mut c_float,
    ) -> c_int;
    pub fn dsee_fill_source(
        facts: *const DseeMediaFacts,
        output_fs: c_int,
        output_bits: c_int,
        enable: c_int,
        out: *mut DseeSource,
    ) -> c_int;
    /// 0 = auto; 8..=16 force F76 level (16 closes same-rate path).
    pub fn dsee_set_level_override(level: c_int);
    /// 1 = HEQ on; 0 = bypass.
    pub fn dsee_set_heq_enabled(enabled: c_int);
    /// 0 = auto; 1 = force F80 off; 2/4 = force F80 on.
    pub fn dsee_set_f80_override(up: c_int);
}
