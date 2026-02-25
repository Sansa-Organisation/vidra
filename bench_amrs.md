# Vidra Benchmark AMRs (Assumptions, Measurement, Risks)

Last updated: 2026-02-25  
Status: Internal benchmark governance document

## 1) Scope
This document defines what we can honestly claim about performance today, based on repository evidence.

## 2) Evidence Sources
- Benchmark runner implementation: [crates/vidra-cli/src/bench_runner.rs](crates/vidra-cli/src/bench_runner.rs)
- Current committed benchmark baseline: [tests/snapshots/benchmarks.json](tests/snapshots/benchmarks.json)
- Benchmark scenes present in repo: [benchmark](benchmark)

## 3) Current Baseline (Committed)
The following are the latest committed values in [tests/snapshots/benchmarks.json](tests/snapshots/benchmarks.json):

| Profile | Resolution | Frames | Duration (ms) | FPS |
|---|---:|---:|---:|---:|
| 720p | 1280x720 | 120 | 1684.48 | 71.24 |
| 1080p | 1920x1080 | 120 | 3092.58 | 38.80 |
| 4K | 3840x2160 | 120 | 13600.47 | 8.82 |

Raw snapshot precision values in [tests/snapshots/benchmarks.json](tests/snapshots/benchmarks.json):
- 720p: `1684.4821250000002 ms`, `71.23851195512091 fps`
- 1080p: `3092.583416 ms`, `38.80251034754951 fps`
- 4K: `13600.465041 ms`, `8.823227708629643 fps`

## 4) Assumptions (A)
These values are only valid under these assumptions:
- Hardware, OS, drivers, thermal state, and background load materially affect results.
- Render duration in the benchmark runner includes full render execution for the project variant.
- Scene complexity is tied to the benchmark input used when baseline was captured.
- Build profile and dependency versions match the environment used to record baseline.

## 5) Measurement Method (M)
Benchmark behavior from [crates/vidra-cli/src/bench_runner.rs](crates/vidra-cli/src/bench_runner.rs):
- Parses/compiles input project.
- Runs 3 target profiles: 720p, 1080p, 4K.
- Measures wall-clock render time using `Instant` and computes FPS as `frame_count / seconds`.
- Optional regression check compares against committed baseline and flags >5% slower runs.

## 6) Risks / Gaps (R)
- Baseline file is a snapshot, not a continuous production telemetry stream.
- No guarantee that baseline represents every project class (motion graphics, heavy effects, long timelines, etc.).
- No cross-hardware percentile distribution is published yet.
- If benchmark scenes or runner logic change, historical comparability can break.
- Recent same-day reruns on the same machine showed material variance (notably 720p) and regression gate failures, indicating benchmark noise and/or perf drift that must be investigated before external performance updates.

## 6.1) Fresh Evidence (2026-02-25)
Command run:

```bash
cargo run -p vidra-cli -- bench benchmark/1080p_solid.vidra
```

Observed runs (same day):

| Run | 720p (ms/fps) | 1080p (ms/fps) | 4K (ms/fps) | Bench Gate |
|---|---|---|---|---|
| A | 2674.9 / 67 | 7898.5 / 23 | 21329.0 / 8 | Failed |
| B | 3052.6 / 59 | 3351.9 / 54 | 13691.0 / 13 | Failed |

Interpretation:
- Run A appears severely degraded and inconsistent with both baseline and Run B.
- Run B improves 1080p and 4K materially vs Run A but still fails the built-in >5% regression threshold for 720p and 1080p relative to baseline.
- This indicates a current benchmarking reliability/perf gap; do not refresh external performance claims until additional controlled reruns are captured and reviewed.

## 7) Claim Policy (Do / Don’t)
### Allowed claims
- “Current committed benchmark baseline in this repo shows the values above.”
- “Benchmarks are reproducible via `vidra bench <file>` and baseline snapshots are versioned.”

### Disallowed claims (unless freshly measured and evidenced)
- “Vidra is always X times faster than competitor Y.”
- “These FPS numbers are guaranteed in customer environments.”
- Any absolute latency/FPS promises without hardware + scene qualification.

## 8) Reproduction Steps
```bash
# Run benchmark on a benchmark scene
cargo run -p vidra-cli -- bench benchmark/1080p_solid.vidra

# Optional: update baseline snapshot after review
cargo run -p vidra-cli -- bench benchmark/1080p_solid.vidra --update-baseline
```

## 9) Release Gate (Recommended)
Before publishing performance claims:
- Re-run benchmarks on at least 2 hardware profiles.
- Store raw output and environment metadata (CPU/GPU/OS/build profile).
- Ensure claims in external docs exactly match measured evidence.
- For the current branch, require at least 3 stable reruns (same command + machine) before accepting a new baseline.
