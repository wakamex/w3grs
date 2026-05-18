#!/usr/bin/env node

import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";
import { spawnSync } from "node:child_process";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const defaultReplay = resolve(
  repoRoot,
  "upstream/w3gjs/test/replays/132/reforged1.w3g",
);

const options = parseArgs(process.argv.slice(2));
const replayPath = resolve(repoRoot, options.replay ?? defaultReplay);
const iterations = options.iterations ?? 25;
const warmup = options.warmup ?? 5;
const upstreamDir = resolve(repoRoot, "upstream/w3gjs");
const w3gjsEntry = resolve(upstreamDir, "dist/esm/index.js");
const rustBenchBin = resolve(repoRoot, "target/release/w3grs-bench");

if (!existsSync(replayPath)) {
  fail(`Replay file does not exist: ${replayPath}`);
}

if (options.prepare || !existsSync(w3gjsEntry)) {
  run("npm", ["ci"], { cwd: upstreamDir });
  run("npm", ["run", "build"], { cwd: upstreamDir });
}

if (options.prepare || !existsSync(rustBenchBin)) {
  run("cargo", ["build", "--release", "--bin", "w3grs-bench"], {
    cwd: repoRoot,
  });
}

if (!existsSync(w3gjsEntry)) {
  fail(
    `Missing ${w3gjsEntry}. Re-run with --prepare or build upstream/w3gjs manually.`,
  );
}

if (!existsSync(rustBenchBin)) {
  fail(`Missing ${rustBenchBin}. Re-run with --prepare or build the Rust binary.`);
}

const jsStats = await benchmarkW3gjs({
  entry: w3gjsEntry,
  replayPath,
  iterations,
  warmup,
});
const rustStats = benchmarkW3grs({ replayPath, iterations, warmup });

if (options.json) {
  console.log(
    JSON.stringify(
      {
        replay: replayPath,
        iterations,
        warmup,
        results: [jsStats, rustStats],
        speedup: jsStats.meanMs / rustStats.meanMs,
      },
      null,
      2,
    ),
  );
} else {
  printSummary(replayPath, iterations, warmup, jsStats, rustStats);
}

async function benchmarkW3gjs({ entry, replayPath, iterations, warmup }) {
  const module = await import(pathToFileURL(entry));
  const W3GReplay = module.default;
  const parser = new W3GReplay();
  const buffer = readFileSync(replayPath);

  for (let i = 0; i < warmup; i++) {
    const output = await parser.parse(buffer);
    blackHole(output.players.length);
  }

  const samples = [];
  let lastPlayers = 0;
  for (let i = 0; i < iterations; i++) {
    const started = performance.now();
    const output = await parser.parse(buffer);
    const elapsed = performance.now() - started;
    lastPlayers = output.players.length;
    blackHole(output.id);
    samples.push(elapsed);
  }

  return summarize("w3gjs", iterations, warmup, samples, lastPlayers);
}

function benchmarkW3grs({ replayPath, iterations, warmup }) {
  const result = spawnSync(
    rustBenchBin,
    [replayPath, String(iterations), String(warmup)],
    {
      cwd: repoRoot,
      encoding: "utf8",
    },
  );

  if (result.status !== 0) {
    fail(
      [
        "w3grs benchmark failed.",
        result.stdout.trim(),
        result.stderr.trim(),
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }

  return JSON.parse(result.stdout);
}

function summarize(parser, iterations, warmup, samples, lastPlayers) {
  const totalMs = samples.reduce((sum, sample) => sum + sample, 0);
  return {
    parser,
    iterations,
    warmup,
    totalMs,
    meanMs: totalMs / samples.length,
    minMs: Math.min(...samples),
    maxMs: Math.max(...samples),
    lastPlayers,
  };
}

function printSummary(replayPath, iterations, warmup, jsStats, rustStats) {
  const speedup = jsStats.meanMs / rustStats.meanMs;
  console.log(`Replay: ${replayPath}`);
  console.log(`Iterations: ${iterations} timed, ${warmup} warmup`);
  console.log("");
  console.log("Parser   total ms   mean ms   min ms    max ms    players");
  for (const stats of [jsStats, rustStats]) {
    console.log(
      `${stats.parser.padEnd(8)} ${fmt(stats.totalMs)} ${fmt(stats.meanMs)} ${fmt(stats.minMs)} ${fmt(stats.maxMs)} ${String(stats.lastPlayers).padStart(8)}`,
    );
  }
  console.log("");
  console.log(`w3grs mean speedup vs w3gjs: ${speedup.toFixed(2)}x`);
}

function parseArgs(args) {
  const parsed = {};
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--prepare") {
      parsed.prepare = true;
    } else if (arg === "--json") {
      parsed.json = true;
    } else if (arg === "--iterations" || arg === "-n") {
      parsed.iterations = parsePositiveInt(args[++i], arg);
    } else if (arg === "--warmup" || arg === "-w") {
      parsed.warmup = parsePositiveInt(args[++i], arg);
    } else if (arg === "--help" || arg === "-h") {
      printHelp();
      process.exit(0);
    } else if (!parsed.replay) {
      parsed.replay = arg;
    } else {
      fail(`Unknown argument: ${arg}`);
    }
  }
  return parsed;
}

function parsePositiveInt(value, flag) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    fail(`${flag} requires a positive integer`);
  }
  return parsed;
}

function run(command, args, options) {
  console.log(`$ ${[command, ...args].join(" ")}`);
  const result = spawnSync(command, args, {
    ...options,
    stdio: "inherit",
  });
  if (result.status !== 0) {
    fail(`${command} ${args.join(" ")} failed`);
  }
}

function fmt(value) {
  return value.toFixed(3).padStart(9);
}

function blackHole(value) {
  globalThis.__w3grsBenchSink = value;
}

function printHelp() {
  console.log(`Usage: node scripts/benchmark.mjs [replay] [options]

Options:
  -n, --iterations N   Timed parses per parser (default: 25)
  -w, --warmup N       Warmup parses per parser (default: 5)
  --prepare            Run npm ci/build for w3gjs and cargo release build for w3grs
  --json               Print machine-readable JSON

Default replay:
  ${defaultReplay}`);
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
