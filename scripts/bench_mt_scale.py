#!/usr/bin/env python3
"""
Parallel scale bench: for workers=k process exactly k items (cycle 5 sources).
Fair metric: realtime factor = sum(audio_duration) / wall_s  (higher = better).
"""

from __future__ import annotations

import csv
import json
import os
import re
import shutil
import subprocess
import sys
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLI = Path(os.environ.get("CARGO_TARGET_DIR", r"C:\Users\Le_Fa\.cache\dsee-hx-cargo-target"))
CLI = CLI / "x86_64-pc-windows-gnu" / "release" / "dsee-cli.exe"
IN_DIR = ROOT / "tmp" / "batch5_in"
OUT_ROOT = ROOT / "tmp" / "bench_mt_scale"
RESULT_CSV = OUT_ROOT / "scale_results.csv"
RESULT_JSON = OUT_ROOT / "scale_results.json"

BENCH_RE = re.compile(r"BENCH_JSON\s+(\{.*\})")
PROGRESS_RE = re.compile(r"^\s*(\d+)/(\d+)\s*$")


def find_album_track_dir() -> Path:
    desktop = Path(r"c:\Users\Le_Fa\Desktop\DSEE HX")
    for p in desktop.iterdir():
        if p.is_dir() and "陶喆" in p.name:
            t = p / "track"
            if t.is_dir():
                return t
    raise SystemExit("album track dir not found")


def ffprobe_duration(path: Path) -> float:
    r = subprocess.run(
        [
            "ffprobe",
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            str(path),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    try:
        return float((r.stdout or "0").strip())
    except ValueError:
        return 0.0


def prepare_inputs() -> list[tuple[Path, float]]:
    src_dir = find_album_track_dir()
    files = sorted(
        f
        for f in src_dir.iterdir()
        if f.suffix.lower() == ".m4a"
        and "_dsee_" not in f.name
        and not f.name.lower().startswith("test")
    )[:5]
    if len(files) < 5:
        raise SystemExit(f"need 5 sources, got {len(files)}")
    if IN_DIR.exists():
        shutil.rmtree(IN_DIR)
    IN_DIR.mkdir(parents=True)
    out: list[tuple[Path, float]] = []
    for f in files:
        dst = IN_DIR / f.name
        shutil.copy2(f, dst)
        dur = ffprobe_duration(dst)
        out.append((dst, dur))
        print(f"  {f.name}: {dur:.2f}s", flush=True)
    return out


def plan_k_items(
    sources: list[tuple[Path, float]], k: int, out_dir: Path, rate: int = 176400
) -> tuple[list[tuple[Path, Path]], float]:
    """Cycle sources to make exactly k jobs; return (jobs, total_duration)."""
    jobs: list[tuple[Path, Path]] = []
    total_dur = 0.0
    out_dir.mkdir(parents=True, exist_ok=True)
    for i in range(k):
        src, dur = sources[i % len(sources)]
        stem = src.stem
        out = out_dir / f"{stem}_k{k:02d}_i{i:02d}_dsee_{rate}.wav"
        jobs.append((src, out))
        total_dur += dur
    return jobs, total_dur


def write_manifest(jobs: list[tuple[Path, Path]], path: Path) -> None:
    # Use hardlinks/copies into a temp indir with unique names? Simpler: call CLI per
    # item is slow. Instead create a staging dir of hardlinks with unique names and
    # use --batch --repeat 1 on that staging dir with exactly k files.
    stage = path
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True)
    for i, (src, _out) in enumerate(jobs):
        link = stage / f"{i:02d}_{src.name}"
        try:
            os.link(src, link)
        except OSError:
            shutil.copy2(src, link)


def sample_memory(pid: int, stop: threading.Event, samples: list[float]):
    while not stop.is_set():
        ps = (
            f"$p=Get-Process -Id {pid} -EA SilentlyContinue; "
            f"if(-not $p){{'0'}} else {{ "
            f"$sum=$p.WS; "
            f"Get-CimInstance Win32_Process | ? {{ $_.ParentProcessId -eq {pid} }} | % {{ "
            f"  try {{ $sum += (Get-Process -Id $_.ProcessId -EA SilentlyContinue).WS }} catch {{}} "
            f"}}; [math]::Round($sum/1MB,2) }}"
        )
        try:
            r = subprocess.run(
                ["powershell", "-NoProfile", "-Command", ps],
                capture_output=True,
                text=True,
                timeout=5,
            )
            samples.append(float((r.stdout or "0").strip().splitlines()[-1]))
        except Exception:
            samples.append(0.0)
        stop.wait(0.4)


def run_workers(
    sources: list[tuple[Path, float]], workers: int, rate: int = 176400
) -> dict:
    out_dir = OUT_ROOT / f"w{workers:02d}"
    stage = OUT_ROOT / f"stage_w{workers:02d}"
    if out_dir.exists():
        shutil.rmtree(out_dir)
    jobs, total_dur = plan_k_items(sources, workers, out_dir, rate)
    write_manifest(jobs, stage)

    # Map staged unique names → desired outputs by running batch on stage
    # plan_dir will emit {stem}_dsee_{rate}.wav; our stems are 00_name etc.
    # Remap: after batch, rename — or pass out_dir and accept staged names.
    # Accept staged output names in out_dir for simplicity.
    cmd = [
        str(CLI),
        "--batch",
        str(stage),
        str(out_dir),
        str(rate),
        "24",
        "--jobs",
        str(workers),
        "--repeat",
        "1",
    ]
    env = os.environ.copy()
    env.pop("DSEE_PROFILE", None)
    env.pop("F80_DETAIL", None)
    if workers > 1:
        env["OMP_NUM_THREADS"] = "2"

    print(
        f"\n=== workers={workers}  items={workers}  audio={total_dur:.1f}s ===",
        flush=True,
    )
    t0 = time.perf_counter()
    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
        bufsize=1,
    )
    stop = threading.Event()
    samples: list[float] = []
    mon = threading.Thread(target=sample_memory, args=(proc.pid, stop, samples), daemon=True)
    mon.start()

    stderr_lines: list[str] = []
    assert proc.stderr is not None
    assert proc.stdout is not None

    # Drain stderr for progress; stdout for BENCH_JSON.
    # Do not use communicate() — it closes stderr while the reader thread runs.
    def read_stderr():
        try:
            assert proc.stderr is not None
            for line in proc.stderr:
                stderr_lines.append(line)
                m = PROGRESS_RE.match(line.strip())
                if m:
                    done, total = int(m.group(1)), int(m.group(2))
                    pct = 100 * done // total if total else 0
                    bar = "█" * (pct // 5) + "░" * (20 - pct // 5)
                    elapsed = time.perf_counter() - t0
                    eta = (elapsed / done * (total - done)) if done else 0
                    print(
                        f"\r  [{bar}] {done}/{total}  {elapsed:.1f}s  ETA {eta:.0f}s   ",
                        end="",
                        flush=True,
                    )
        except (ValueError, OSError):
            pass

    th = threading.Thread(target=read_stderr, daemon=True)
    th.start()
    stdout = proc.stdout.read()
    proc.wait()
    th.join(timeout=5)
    wall = time.perf_counter() - t0
    stop.set()
    mon.join(timeout=2)
    print()  # newline after progress

    peak_mb = max(samples) if samples else 0.0
    avg_mb = sum(samples) / len(samples) if samples else 0.0

    bench = None
    for line in (stdout or "").splitlines():
        m = BENCH_RE.search(line)
        if m:
            bench = json.loads(m.group(1))
    if bench is None:
        err = "".join(stderr_lines)[-1500:]
        raise RuntimeError(f"no BENCH_JSON (exit={proc.returncode})\n{err}")

    rtf = total_dur / wall if wall > 0 else 0.0  # × realtime
    row = {
        "workers": workers,
        "items": workers,
        "audio_duration_s": round(total_dur, 3),
        "wall_s": round(wall, 3),
        "realtime_factor": round(rtf, 3),
        "ok": bench["ok"],
        "fail": bench["fail"],
        "sum_probe_s": round(bench["sum_probe_s"], 3),
        "sum_open_s": round(bench["sum_open_s"], 3),
        "sum_decode_s": round(bench["sum_decode_s"], 3),
        "sum_dsp_s": round(bench["sum_dsp_s"], 3),
        "sum_write_s": round(bench["sum_write_s"], 3),
        "sum_total_s": round(bench["sum_total_s"], 3),
        "hops": bench["hops"],
        "peak_working_set_mb": round(peak_mb, 1),
        "avg_working_set_mb": round(avg_mb, 1),
        "parallel_efficiency": round(
            bench["sum_total_s"] / (wall * workers), 3
        )
        if wall > 0 and workers > 0
        else 0.0,
        "exit_code": proc.returncode,
    }
    print(
        f"  wall={row['wall_s']}s  audio={row['audio_duration_s']}s  "
        f"RTF={row['realtime_factor']}x  "
        f"Σdsp={row['sum_dsp_s']}s Σdecode={row['sum_decode_s']}s Σwrite={row['sum_write_s']}s  "
        f"peakRSS={row['peak_working_set_mb']}MB  eff={row['parallel_efficiency']}",
        flush=True,
    )
    # free disk
    shutil.rmtree(out_dir, ignore_errors=True)
    shutil.rmtree(stage, ignore_errors=True)
    return row


def main() -> int:
    if not CLI.is_file():
        print(f"missing CLI: {CLI}", file=sys.stderr)
        return 2
    OUT_ROOT.mkdir(parents=True, exist_ok=True)
    print("sources (duration via ffprobe):", flush=True)
    sources = prepare_inputs()
    print(
        f"\nplan: workers=k → process k items (cycle {len(sources)} sources)\n"
        f"fair metric: realtime_factor = sum(duration) / wall\n"
        f"sweep: 1..16",
        flush=True,
    )

    rows: list[dict] = []
    for w in range(1, 17):
        print(f"\n---- sweep {w}/16 ----", flush=True)
        rows.append(run_workers(sources, w))
        # checkpoint
        with RESULT_CSV.open("w", newline="", encoding="utf-8") as f:
            wr = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
            wr.writeheader()
            wr.writerows(rows)
        RESULT_JSON.write_text(json.dumps(rows, indent=2), encoding="utf-8")

    print(f"\nwrote {RESULT_CSV}")
    print("\nworkers items audio_s  wall_s   RTF×  Σdsp  Σdecode Σwrite peakMB  eff")
    for r in rows:
        print(
            f"{r['workers']:7d} {r['items']:5d} {r['audio_duration_s']:7.1f} "
            f"{r['wall_s']:7.1f} {r['realtime_factor']:6.2f} "
            f"{r['sum_dsp_s']:6.1f} {r['sum_decode_s']:7.1f} {r['sum_write_s']:6.1f} "
            f"{r['peak_working_set_mb']:7.0f} {r['parallel_efficiency']:5.2f}"
        )
    best = max(rows, key=lambda r: r["realtime_factor"])
    print(
        f"\nbest RTF: workers={best['workers']}  {best['realtime_factor']}× realtime "
        f"(wall={best['wall_s']}s for {best['audio_duration_s']}s audio)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
