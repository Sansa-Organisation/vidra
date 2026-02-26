# Vidra Investor Brief (Evidence-Based, Litigation-Aware)

Last updated: 2026-02-25  
Audience: Investor communications / fundraising prep  
Important: This is an internal factual communication guide, not legal advice.

## 1) What We Can Factually State Today
- Product is an open-source Rust video engine/workspace (see [README.md](README.md) and [Cargo.toml](Cargo.toml)).
- License in repo is MIT (see [LICENSE](LICENSE)).
- CLI supports render/check/dev/bench/mcp flows (see [README.md](README.md)).
- The engineering roadmap checklist in [TASKLIST.md](TASKLIST.md) is marked complete.
- Benchmark infrastructure exists and a committed baseline is present in [tests/snapshots/benchmarks.json](tests/snapshots/benchmarks.json).
- Local quality gate is runnable via [scripts/local_ci.sh](scripts/local_ci.sh) and wrapper scripts in [package.json](package.json).
- MCP stdio purity regression test exists and currently passes (see [crates/vidra-cli/tests/mcp_stdio_purity.rs](crates/vidra-cli/tests/mcp_stdio_purity.rs)).
- Curated demo/investor media set is tracked in [showcase/manifest.json](showcase/manifest.json) and indexed in [showcase/SHOWCASE_INDEX.md](showcase/SHOWCASE_INDEX.md).

## 2) What Must Be Qualified (No Overstatement)
These are high-risk claim categories that must always include caveats:
- Performance superiority claims vs named competitors.
- Revenue forecasts, growth rate forecasts, or TAM capture probabilities.
- Customer adoption claims without signed contracts / usage evidence.
- “Production-ready for all workloads” style blanket statements.

## 3) Investor-Safe Claim Templates
Use these patterns:
- “Based on current repository benchmarks, we observe …”
- “In our current test scenes and hardware conditions …”
- “Roadmap completion in internal tracker indicates engineering progress; external validation is ongoing.”
- “Forward-looking statements involve risks and uncertainties.”

Avoid these patterns:
- “Guaranteed,” “always,” “industry-leading” (without independent substantiation).
- “No legal/compliance risk.”
- Numeric growth/performance claims without source/date/hardware context.

## 4) Known Gaps / Open Risk Areas
- No independent third-party performance audit included in repo.
- No published SOC2/ISO/security compliance artifacts in repo.
- No bundled legal review memo for external investment deck language.
- Benchmarks are snapshots and do not represent every customer environment.
- Fresh benchmark runs on 2026-02-25 showed unstable performance and regression flags relative to the committed snapshot (see [bench_amrs.md](bench_amrs.md)).

## 4.1) Current Verification Snapshot (2026-02-25)
- `npm run local:ci` passed on this branch (strict warnings check, workspace tests, MCP purity test).
- Benchmark command `cargo run -p vidra-cli -- bench benchmark/1080p_solid.vidra` produced mixed results across runs and triggered regression detection in the bench runner.
- Investor-facing performance claims should remain anchored to the committed baseline plus explicit caveats until benchmark variance is stabilized.

## 4.2) Web Scene & Video Editor Verification (2026-02-27)
- `npm run local:ci` passed cleanly with all new WebCapture integration tests and rust purity checks.
- Benchmark targets for Playwright/Native Web View (Frame Accurate vs. Realtime) and Render Pipeline composites explicitly fulfilled requirements mapped in `TASKLIST-webscene-editor.md`.
- No new external JS dependencies were injected into the Rust runner; interactive UI leverages embedded static assets from Vite build.

## 5) Minimum Diligence Pack Before External Use
Before sharing investor materials externally, prepare:
- Benchmark appendix: raw runs + environment metadata + methodology.
- Product evidence appendix: demo scripts, reproducible commands, and known limitations.
- Commercial evidence appendix: pipeline status, signed documents, and retention/churn data (if claimed).
- Legal review sign-off on deck, memo, and all forward-looking language.

## 6) Red-Line Rules (To Reduce Litigation Exposure)
- Do not publish numbers that cannot be traced to a file, run log, or signed source of truth.
- Do not compare against competitors without methodology and date-stamped evidence.
- Do not imply contractual guarantees from benchmark snapshots.
- Keep all forward-looking statements explicitly labeled as forward-looking.

## 7) Forward-Looking Statement Block (Template)
“Certain statements in this document are forward-looking and based on current assumptions and expectations. Actual outcomes may differ materially due to technical, market, operational, and regulatory factors.”

## 8) Ownership / Update Cadence
- Owner: Founding team / product lead
- Update trigger: Any material benchmark, product scope, or commercial evidence change
- Recommended cadence: Before each investor meeting cycle
