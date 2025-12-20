# RepoLens – Orientation

## What this project is
A local Git engine that exposes bounded, observable views for a UI.

## What currently works
- Engine request dispatch with telemetry
- Status implemented via git CLI
- Benchmark harness with clean JSON output
- Oracle harness comparing engine vs git
- Synthetic fixture repo for correctness tests

## What is NOT implemented yet
- DiffSummary (in progress / planned)
- Patch diff streaming
- Commit log
- Watch/event streaming
- UI

## How to run key things
- cargo test
- cargo run -p rl_cli -- status --repo <path>
- cargo run -p rl_bench -- run

## Mental model
Frontend asks for bounded views.
Engine executes git work.
Oracle asserts correctness.
Bench asserts cost.
Telemetry explains why.
