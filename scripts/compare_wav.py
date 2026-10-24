#!/usr/bin/env python3
"""Byte / sample compare two WAVs (FINAL cli vs Tauri/dsee-cli)."""
from __future__ import annotations

import struct
import sys
from pathlib import Path


def read_wav_pcm(path: Path) -> tuple[int, int, bytes]:
    data = path.read_bytes()
    if data[0:4] != b"RIFF" or data[8:12] != b"WAVE":
        raise SystemExit(f"not wav: {path}")
    # naive: find 'data' chunk
    i = 12
    rate = 0
    bps = 0
    pcm = b""
    while i + 8 <= len(data):
        tag = data[i : i + 4]
        size = struct.unpack_from("<I", data, i + 4)[0]
        body = data[i + 8 : i + 8 + size]
        if tag == b"fmt ":
            _, _, rate, _, _, bps = struct.unpack_from("<HHIIHH", body, 0)
        elif tag == b"data":
            pcm = body
            break
        i += 8 + size + (size & 1)
    return rate, bps, pcm


def main() -> None:
    if len(sys.argv) != 3:
        print("usage: compare_wav.py a.wav b.wav")
        raise SystemExit(2)
    a, b = Path(sys.argv[1]), Path(sys.argv[2])
    ra, ba, pa = read_wav_pcm(a)
    rb, bb, pb = read_wav_pcm(b)
    print(f"A: {a.name} rate={ra} bits={ba} bytes={len(pa)}")
    print(f"B: {b.name} rate={rb} bits={bb} bytes={len(pb)}")
    if (ra, ba) != (rb, bb):
        print("FAIL: format mismatch")
        raise SystemExit(1)
    if pa == pb:
        print("OK: identical PCM")
        raise SystemExit(0)
    n = min(len(pa), len(pb))
    diff = sum(1 for i in range(n) if pa[i] != pb[i])
    print(f"FAIL: {diff} differing bytes (len A={len(pa)} B={len(pb)})")
    raise SystemExit(1)


if __name__ == "__main__":
    main()
