# w3grs

`w3grs` is a Rust port of [`w3gjs`](https://github.com/PBug90/w3gjs), intended to be a clean library for parsing Warcraft III replay files in other Rust projects.

The upstream TypeScript source is tracked as a Git submodule in `upstream/w3gjs` for parity tests, benchmarks, and future maintenance. The published crate contains the Rust library and a small fixture subset, not the full upstream replay corpus.

## Installation

`w3grs` requires Rust 1.85 or newer.

```sh
cargo add w3grs
```

## Development Checkout

Clone with submodules when you want to run the repository's `w3gjs` parity and speed comparison scripts:

```sh
git clone --recurse-submodules git@github.com:wakamex/w3grs.git
```

For an existing checkout:

```sh
git submodule update --init --recursive
```

## Usage

```rust
use w3grs::W3GReplay;

fn main() -> w3grs::Result<()> {
    let mut parser = W3GReplay::new();
    let replay = parser.parse_file("replay.w3g")?;

    println!("{} on {}", replay.matchup, replay.map.file);
    println!("players: {}", replay.players.len());

    Ok(())
}
```

Lower-level parser layers are also public:

- `RawParser` for replay headers and compressed data blocks
- `MetadataParser` for lobby/map/player setup data
- `GameDataParser` and `ActionParser` for timeslots, chat, leave events, and player actions
- `ReplayParser` for the combined low-level parse output

Consumers that need both the high-level summary and the raw action stream can parse once and use the timed action helper:

```rust
use w3grs::{W3GReplay, action::format_fourcc_or_hex};

fn main() -> w3grs::Result<()> {
    let mut parser = W3GReplay::new();
    let parsed = parser.parse_file_detailed("replay.w3g")?;

    println!("{} on {}", parsed.summary.matchup, parsed.summary.map.file);
    for timed in parsed.low_level.timed_actions() {
        println!(
            "time_ms={} player={} action=0x{:02x}",
            timed.time_ms,
            timed.player_id,
            timed.action.id()
        );
    }

    let order = format_fourcc_or_hex(*b"trah");
    println!("formatted order id: {order}");

    Ok(())
}
```

## Benchmark

From a development checkout with the submodule initialized, compare local `w3gjs` and `w3grs` parsing speed on the same replay:

```sh
node scripts/benchmark.mjs --prepare upstream/w3gjs/test/replays/132/reforged1.w3g
```

Useful options:

```sh
node scripts/benchmark.mjs replay.w3g --iterations 100 --warmup 10
node scripts/benchmark.mjs replay.w3g --json
```

The benchmark reads the replay once per parser process, warms both parsers, then reports timed in-process parses. `--prepare` runs the local `w3gjs` install/build and builds the Rust benchmark binary in release mode.

Example smoke result on this repo's `reforged1.w3g` fixture with 2 timed iterations and 1 warmup:

```text
Replay: upstream/w3gjs/test/replays/132/reforged1.w3g
Iterations: 2 timed, 1 warmup

Parser   total ms   mean ms   min ms    max ms    players
w3gjs       61.179    30.589    30.201    30.977        2
w3grs        1.863     0.932     0.849     1.014        2

w3grs mean speedup vs w3gjs: 32.84x
```

Benchmark results vary by machine, replay, iteration count, and current CPU load. Use larger iteration counts for less noisy comparisons:

```sh
node scripts/benchmark.mjs upstream/w3gjs/test/replays/132/reforged1.w3g --iterations 100 --warmup 10
```

## Upstream Parity And Speed Sweep

From a development checkout, check every replay fixture in `upstream/w3gjs/test/replays` for output parity while timing both parsers:

```sh
node scripts/compare-all.mjs --prepare --iterations 3 --warmup 1
```

The sweep reports:

- exact canonical JSON byte parity, including `parseTime`
- normalized canonical JSON parity with `parseTime` removed
- `w3gjs` and `w3grs` mean parse time per replay
- aggregate mean/min/max speedup

`parseTime` is expected to break exact parity because it measures each parser's runtime. Normalized parity is the useful output-equivalence signal.

## Intentional Divergences

`w3grs` is stricter than `w3gjs` about required replay metadata structure. For example, the metadata parser returns an error when the lobby setup marker is not the expected `0x19` byte, while `w3gjs` logs the unknown chunk and keeps parsing. This is intentional: the marker is an internal W3G consistency check, not a user-facing replay metric, and treating an invalid marker as an error helps catch corrupt or out-of-sync replay metadata early. Nearby length fields such as `remainingBytes` are likewise internal format fields.

The low-level Rust API also returns parsed `game_data_blocks` directly instead of exposing them only through Node-style events. This keeps the same underlying replay data but presents it in a Rust-friendly result structure.

Useful options:

```sh
node scripts/compare-all.mjs --iterations 10 --warmup 2
node scripts/compare-all.mjs --json
node scripts/compare-all.mjs --fail-on-mismatch --write-mismatches tmp/parity
```

Recent local smoke result on the upstream submodule replay fixtures with 1 timed parse and no warmup:

```text
Replays: 50
Exact byte parity: 0/50 (mismatches include expected parseTime differences)
Normalized parity without parseTime: 50/50
Speedup mean/min/max: 17.99x / 2.81x / 78.62x
```
