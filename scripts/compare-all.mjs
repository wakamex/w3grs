#!/usr/bin/env node

import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";
import { spawnSync } from "node:child_process";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const upstreamDir = resolve(repoRoot, "upstream/w3gjs");
const replayRoot = resolve(upstreamDir, "test/replays");
const w3gjsEntry = resolve(upstreamDir, "dist/esm/index.js");
const rustCompareBin = resolve(repoRoot, "target/release/w3grs-compare-one");

const options = parseArgs(process.argv.slice(2));
const iterations = options.iterations ?? 3;
const warmup = options.warmup ?? 1;
const mismatchDir = options.writeMismatches
  ? resolve(repoRoot, options.writeMismatches)
  : null;

if (options.prepare || !existsSync(w3gjsEntry)) {
  run("npm", ["ci"], { cwd: upstreamDir });
  run("npm", ["run", "build"], { cwd: upstreamDir });
}

if (options.prepare || !existsSync(rustCompareBin)) {
  run("cargo", ["build", "--release", "--bin", "w3grs-compare-one"], {
    cwd: repoRoot,
  });
}

if (!existsSync(w3gjsEntry)) {
  fail(`Missing ${w3gjsEntry}. Re-run with --prepare or build upstream/w3gjs.`);
}
if (!existsSync(rustCompareBin)) {
  fail(`Missing ${rustCompareBin}. Re-run with --prepare or build the Rust helper.`);
}
if (mismatchDir) {
  mkdirSync(mismatchDir, { recursive: true });
}

const W3GReplay = (await import(pathToFileURL(w3gjsEntry))).default;
const replays = listReplayFiles(replayRoot);
const selected = options.limit ? replays.slice(0, options.limit) : replays;
const rows = [];

for (const [index, replayPath] of selected.entries()) {
  const rel = relative(repoRoot, replayPath);
  process.stderr.write(`[${index + 1}/${selected.length}] ${rel}\n`);
  const js = await parseAndTimeW3gjs(W3GReplay, replayPath, iterations, warmup);
  const rust = parseAndTimeW3grs(replayPath, iterations, warmup);
  const jsNormalizedOutput = normalizeForParity(js.output);
  const rustNormalizedOutput = normalizeForParity(rust.output);
  const jsExact = canonicalStringify(js.output);
  const rustExact = canonicalStringify(rust.output);
  const jsNormalized = canonicalStringify(jsNormalizedOutput);
  const rustNormalized = canonicalStringify(rustNormalizedOutput);
  const exactParity = Buffer.from(jsExact).equals(Buffer.from(rustExact));
  const normalizedParity = Buffer.from(jsNormalized).equals(Buffer.from(rustNormalized));
  const speedup = js.stats.meanMs / rust.stats.meanMs;
  const row = {
    replay: rel,
    exactParity,
    normalizedParity,
    firstDiff: normalizedParity
      ? null
      : firstDiff(jsNormalizedOutput, rustNormalizedOutput),
    w3gjsMeanMs: js.stats.meanMs,
    w3grsMeanMs: rust.stats.meanMs,
    speedup,
    players: {
      w3gjs: js.output.players?.length ?? null,
      w3grs: rust.output.players?.length ?? null,
    },
  };
  rows.push(row);

  if (mismatchDir && !normalizedParity) {
    const safeName = rel.replaceAll("/", "__").replaceAll(" ", "_");
    writeFileSync(join(mismatchDir, `${safeName}.w3gjs.json`), `${jsNormalized}\n`);
    writeFileSync(join(mismatchDir, `${safeName}.w3grs.json`), `${rustNormalized}\n`);
  }
}

const summary = summarize(rows, iterations, warmup);
if (options.json) {
  console.log(JSON.stringify({ ...summary, rows }, null, 2));
} else {
  printSummary(summary, rows);
}

if (options.failOnMismatch && summary.normalizedMismatches > 0) {
  process.exit(2);
}

async function parseAndTimeW3gjs(W3GReplay, replayPath, iterations, warmup) {
  const parser = new W3GReplay();
  const buffer = readFileSync(replayPath);

  for (let i = 0; i < warmup; i++) {
    const output = await parser.parse(buffer);
    blackHole(output.players.length);
  }

  const samples = [];
  let output = null;
  for (let i = 0; i < iterations; i++) {
    const started = performance.now();
    output = await parser.parse(buffer);
    samples.push(performance.now() - started);
    blackHole(output.id);
  }

  return {
    parser: "w3gjs",
    iterations,
    warmup,
    stats: summarizeSamples(samples),
    output: JSON.parse(JSON.stringify(output)),
  };
}

function parseAndTimeW3grs(replayPath, iterations, warmup) {
  const result = spawnSync(
    rustCompareBin,
    [replayPath, String(iterations), String(warmup)],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 256,
    },
  );
  if (result.status !== 0) {
    fail(
      [
        `w3grs failed for ${relative(repoRoot, replayPath)}`,
        result.stdout.trim(),
        result.stderr.trim(),
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }
  return JSON.parse(result.stdout);
}

function summarize(rows, iterations, warmup) {
  const exactMatches = rows.filter((row) => row.exactParity).length;
  const normalizedMatches = rows.filter((row) => row.normalizedParity).length;
  const speedups = rows.map((row) => row.speedup).filter(Number.isFinite);
  return {
    replayCount: rows.length,
    iterations,
    warmup,
    exactMatches,
    exactMismatches: rows.length - exactMatches,
    normalizedMatches,
    normalizedMismatches: rows.length - normalizedMatches,
    meanSpeedup:
      speedups.reduce((sum, speedup) => sum + speedup, 0) / speedups.length,
    minSpeedup: Math.min(...speedups),
    maxSpeedup: Math.max(...speedups),
  };
}

function printSummary(summary, rows) {
  console.log("");
  console.log(`Replays: ${summary.replayCount}`);
  console.log(`Iterations: ${summary.iterations} timed, ${summary.warmup} warmup`);
  console.log(
    `Exact byte parity: ${summary.exactMatches}/${summary.replayCount} ` +
      `(mismatches include expected parseTime differences)`,
  );
  console.log(
    `Normalized parity without parseTime: ${summary.normalizedMatches}/${summary.replayCount}`,
  );
  console.log(
    `Speedup mean/min/max: ${summary.meanSpeedup.toFixed(2)}x / ` +
      `${summary.minSpeedup.toFixed(2)}x / ${summary.maxSpeedup.toFixed(2)}x`,
  );

  const mismatches = rows.filter((row) => !row.normalizedParity);
  if (mismatches.length > 0) {
    console.log("");
    console.log("Normalized mismatches:");
    for (const row of mismatches.slice(0, 20)) {
      console.log(`- ${row.replay}: ${row.firstDiff?.path ?? "(unknown)"}`);
    }
    if (mismatches.length > 20) {
      console.log(`... ${mismatches.length - 20} more`);
    }
  }

  console.log("");
  console.log("Replay".padEnd(72), "w3gjs ms", "w3grs ms", "speedup", "parity");
  for (const row of rows) {
    console.log(
      row.replay.slice(0, 72).padEnd(72),
      fmt(row.w3gjsMeanMs),
      fmt(row.w3grsMeanMs),
      `${row.speedup.toFixed(2)}x`.padStart(7),
      row.normalizedParity ? "ok" : "DIFF",
    );
  }
}

function normalizeForParity(value) {
  if (Array.isArray(value)) {
    return value.map(normalizeForParity);
  }
  if (value && typeof value === "object") {
    const normalized = {};
    for (const [key, nested] of Object.entries(value)) {
      if (key === "parseTime") {
        continue;
      }
      normalized[key] = normalizeForParity(nested);
    }
    return normalized;
  }
  return value;
}

function canonicalStringify(value) {
  return JSON.stringify(sortKeys(value));
}

function sortKeys(value) {
  if (Array.isArray(value)) {
    return value.map(sortKeys);
  }
  if (value && typeof value === "object") {
    const sorted = {};
    for (const key of Object.keys(value).sort()) {
      sorted[key] = sortKeys(value[key]);
    }
    return sorted;
  }
  return value;
}

function firstDiff(left, right, path = "$") {
  if (Object.is(left, right)) {
    return null;
  }
  if (typeof left !== typeof right) {
    return { path, left, right };
  }
  if (Array.isArray(left) || Array.isArray(right)) {
    if (!Array.isArray(left) || !Array.isArray(right)) {
      return { path, left, right };
    }
    if (left.length !== right.length) {
      return { path: `${path}.length`, left: left.length, right: right.length };
    }
    for (let i = 0; i < left.length; i++) {
      const diff = firstDiff(left[i], right[i], `${path}[${i}]`);
      if (diff) return diff;
    }
    return null;
  }
  if (left && typeof left === "object") {
    const leftKeys = Object.keys(left).sort();
    const rightKeys = Object.keys(right).sort();
    if (leftKeys.join("\0") !== rightKeys.join("\0")) {
      return { path: `${path} keys`, left: leftKeys, right: rightKeys };
    }
    for (const key of leftKeys) {
      const diff = firstDiff(left[key], right[key], `${path}.${key}`);
      if (diff) return diff;
    }
    return null;
  }
  return { path, left, right };
}

function summarizeSamples(samples) {
  const totalMs = samples.reduce((sum, sample) => sum + sample, 0);
  return {
    totalMs,
    meanMs: totalMs / samples.length,
    minMs: Math.min(...samples),
    maxMs: Math.max(...samples),
  };
}

function listReplayFiles(root) {
  return readdirRecursive(root)
    .filter((path) => /\.(w3g|nwg)$/i.test(path))
    .sort();
}

function readdirRecursive(root) {
  const entries = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) {
      entries.push(...readdirRecursive(path));
    } else {
      entries.push(path);
    }
  }
  return entries;
}

function parseArgs(args) {
  const parsed = {};
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--prepare") {
      parsed.prepare = true;
    } else if (arg === "--json") {
      parsed.json = true;
    } else if (arg === "--fail-on-mismatch") {
      parsed.failOnMismatch = true;
    } else if (arg === "--write-mismatches") {
      parsed.writeMismatches = args[++i];
    } else if (arg === "--iterations" || arg === "-n") {
      parsed.iterations = parseNonNegativeInt(args[++i], arg, false);
    } else if (arg === "--warmup" || arg === "-w") {
      parsed.warmup = parseNonNegativeInt(args[++i], arg, true);
    } else if (arg === "--limit") {
      parsed.limit = parseNonNegativeInt(args[++i], arg, false);
    } else if (arg === "--help" || arg === "-h") {
      printHelp();
      process.exit(0);
    } else {
      fail(`Unknown argument: ${arg}`);
    }
  }
  return parsed;
}

function parseNonNegativeInt(value, flag, allowZero) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isInteger(parsed) || parsed < 0 || (!allowZero && parsed === 0)) {
    fail(`${flag} requires ${allowZero ? "a non-negative" : "a positive"} integer`);
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
  globalThis.__w3grsCompareSink = value;
}

function printHelp() {
  console.log(`Usage: node scripts/compare-all.mjs [options]

Checks every replay in upstream/w3gjs/test/replays.

Options:
  -n, --iterations N      Timed parses per parser per replay (default: 3)
  -w, --warmup N          Warmup parses per parser per replay (default: 1)
  --prepare               Build local w3gjs and release w3grs helper
  --json                  Print machine-readable JSON
  --limit N               Only process the first N replay fixtures
  --fail-on-mismatch      Exit 2 if normalized parity fails
  --write-mismatches DIR  Write normalized mismatch JSON pairs to DIR

Exact byte parity uses canonical JSON and includes parseTime.
Normalized parity uses canonical JSON with parseTime removed.`);
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
