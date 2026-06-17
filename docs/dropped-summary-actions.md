# Dropped Summary Actions

Full path: `/code/w3grs/docs/dropped-summary-actions.md`

Generated on 2026-06-16 from the 50 upstream fixtures under
`upstream/w3gjs/test/replays`.

This document counts action records that the high-level summary parser scans
but does not emit to `ParserOutput` player summaries. This is about the
summary path used by `W3GReplay::parse_bytes`; it is not necessarily a loss
from the detailed API. When the low-level parser has a public `Action` variant
for a dropped id, that variant is listed below.

By default, the low-level parser also follows `w3gjs` for ids that upstream
drops. The optional Cargo feature `extended-actions` intentionally diverges from
that default for selected conformance-critical actions. At the time of writing,
it emits normalized action id `0x7b` as `Action::CommandCardSource` and exposes
known ids `0x50`, `0x60`, `0x62`, `0x69`, and `0x6a` as named variants in the
low-level/timed action stream. Other still-undecoded ids are exposed as
`Action::OpaqueDroppedAction`.

The table mixes two cases:

- Summary-dropped actions that are already exposed in the low-level API, such as
  `SelectSubgroup`, `SelectUnit`, `AllyPing`, `Mouse`, `BlzSync`, and
  `CommandFrame`.
- Summary-dropped actions that are not exposed by default. With
  `extended-actions`, ids `0x50`, `0x60`, `0x62`, `0x69`, `0x6a`, and `0x7b`
  are decoded as named variants; ids `0x02` and `0x7a` are emitted as opaque
  payloads.

Action ids are normalized with the same rule as the parser: for post-2.0.2
replays, raw action ids greater than `0x77` are shifted up by one.

Unknown-player command blocks are also skipped by the high-level parser before
decoding individual actions. Those bytes cannot honestly be attributed to
specific action ids without parsing data the production code deliberately
short-circuits, so they are reported separately.

## Corpus Summary

| Metric | Count |
|---|---:|
| Corpus replays | 50 |
| Known-player action records scanned | 962092 |
| Summary-emitted actions | 484617 |
| Summary-dropped actions | 477442 |
| Truncated/unclassified action starts | 33 |
| Skipped unknown-player command blocks | 6 |
| Skipped unknown-player action bytes | 78 |

## Dropped By Action Id

| Normalized id | Raw ids observed | Dropped count | Total observed | Dropped % | Summary behavior | Low-level Action variant |
|---:|---|---:|---:|---:|---|---|
| `0x02` | 0x02 | 13 | 13 | 100.00% | drop: no summary effect | `OpaqueDroppedAction` with `extended-actions` |
| `0x19` | 0x19 | 163064 | 163064 | 100.00% | drop: subgroup selection | SelectSubgroup |
| `0x1a` | 0x1a | 163532 | 163532 | 100.00% | drop: pre-subselection marker | PreSubselection |
| `0x1b` | 0x1b | 79978 | 79978 | 100.00% | drop: select unit | SelectUnit |
| `0x50` | 0x50 | 25 | 25 | 100.00% | drop: alliance flags | `ChangeAllyOptions` with `extended-actions` |
| `0x60` | 0x60 | 2880 | 2880 | 100.00% | drop: map trigger chat command | `MapTriggerChatCommand` with `extended-actions` |
| `0x62` | 0x62 | 3094 | 3094 | 100.00% | drop: scenario trigger | `ScenarioTrigger` with `extended-actions` |
| `0x68` | 0x68 | 580 | 580 | 100.00% | drop: ally ping | AllyPing |
| `0x69` | 0x69 | 6 | 6 | 100.00% | drop: continue game block B | `ContinueGameBlockB` with `extended-actions` |
| `0x6a` | 0x6a | 6 | 6 | 100.00% | drop: continue game block A | `ContinueGameBlockA` with `extended-actions` |
| `0x6b` | 0x6b | 548 | 548 | 100.00% | drop: game cache/sync storage | BlzCacheStoreInt |
| `0x75` | 0x75 | 232 | 232 | 100.00% | drop: arrow key | ArrowKey |
| `0x76` | 0x76 | 28872 | 28872 | 100.00% | drop: mouse event | Mouse |
| `0x77` | 0x77 | 2 | 14 | 14.29% | drop: W3 API command | W3Api |
| `0x78` | 0x78 | 100 | 100 | 100.00% | drop: BLZ sync | BlzSync |
| `0x79` | 0x79 | 4361 | 4361 | 100.00% | drop: command frame | CommandFrame |
| `0x7a` | 0x7a | 495 | 495 | 100.00% | drop: Reforged/post-202 action | `OpaqueDroppedAction` with `extended-actions` |
| `0x7b` | 0x7a, 0x7b | 29654 | 29654 | 100.00% | drop by default: Reforged/post-202 command-card source action; emitted with `extended-actions` | `CommandCardSource` with `extended-actions` |

## Truncated Or Unclassified Starts

The summary scanner counts an action id before decoding its payload. If payload
decoding reaches the end of the command block, production parsing breaks out
without classifying that action as emitted or dropped. These starts are listed
here so the corpus totals reconcile.

| Normalized id | Raw ids observed | Truncated/unclassified starts | Total observed |
|---:|---|---:|---:|
| `0x01` | 0x01 | 21 | 21 |
| `0x77` | 0x77 | 12 | 14 |

## Per-Replay Counts

| Replay | Dropped actions | Emitted actions | Truncated starts | Total scanned | Skipped unknown-player command blocks | Skipped unknown-player action bytes |
|---|---:|---:|---:|---:|---:|---:|
| `upstream/w3gjs/test/replays/126/999.w3g` | 3125 | 2462 | 0 | 5587 | 0 | 0 |
| `upstream/w3gjs/test/replays/126/standard_126.w3g` | 14116 | 18832 | 0 | 32948 | 0 | 0 |
| `upstream/w3gjs/test/replays/129/netease_129_obs.nwg` | 8670 | 11343 | 0 | 20013 | 0 | 0 |
| `upstream/w3gjs/test/replays/129/standard_129_3on3_leaver.w3g` | 45391 | 75225 | 3 | 120619 | 0 | 0 |
| `upstream/w3gjs/test/replays/129/standard_129_obs.w3g` | 7162 | 8279 | 0 | 15441 | 0 | 0 |
| `upstream/w3gjs/test/replays/130/standard_130.w3g` | 20999 | 23994 | 0 | 44993 | 0 | 0 |
| `upstream/w3gjs/test/replays/130/standard_1302.w3g` | 10522 | 12710 | 0 | 23232 | 0 | 0 |
| `upstream/w3gjs/test/replays/130/standard_1303.w3g` | 9287 | 13942 | 0 | 23229 | 0 | 0 |
| `upstream/w3gjs/test/replays/130/standard_1304.2on2.w3g` | 17920 | 21537 | 0 | 39457 | 0 | 0 |
| `upstream/w3gjs/test/replays/130/standard_1304.w3g` | 6474 | 9429 | 0 | 15903 | 0 | 0 |
| `upstream/w3gjs/test/replays/131/action0x7a.w3g` | 530 | 0 | 3 | 533 | 0 | 0 |
| `upstream/w3gjs/test/replays/131/roc-losttemple-mapname.w3g` | 4614 | 7933 | 0 | 12547 | 0 | 0 |
| `upstream/w3gjs/test/replays/131/standard_tomeofretraining_1.w3g` | 6984 | 8990 | 0 | 15974 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/1448202825.w3g` | 3628 | 4251 | 0 | 7879 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/1582070968.nwg` | 4039 | 4944 | 0 | 8983 | 2 | 26 |
| `upstream/w3gjs/test/replays/132/1582161008.nwg` | 4591 | 5524 | 0 | 10115 | 2 | 26 |
| `upstream/w3gjs/test/replays/132/1640262494.w3g` | 11320 | 12530 | 0 | 23850 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/706266088.w3g` | 11030 | 15140 | 0 | 26170 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/benjiii_vs_Scars_Concealed_Hill.w3g` | 5476 | 8244 | 1 | 13721 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/buildingwin_anxietyperspective.w3g` | 143 | 114 | 0 | 257 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/buildingwin_helpstoneperspective.w3g` | 143 | 114 | 0 | 257 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/ced_vs_lyn.w3g` | 8164 | 8069 | 0 | 16233 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/esl_cup_vs_changer_1.w3g` | 8457 | 11376 | 1 | 19834 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/moju_vs_fly.nwg` | 4311 | 5364 | 0 | 9675 | 2 | 26 |
| `upstream/w3gjs/test/replays/132/netease_132.nwg` | 7482 | 8835 | 0 | 16317 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/reforged1.w3g` | 1513 | 1473 | 0 | 2986 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/reforged2.w3g` | 1809 | 3298 | 0 | 5107 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/reforged2010.w3g` | 35855 | 46147 | 3 | 82005 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/reforged_hunter2_privatestring.w3g` | 3158 | 4625 | 0 | 7783 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/reforged_metadata_ghostplayer.w3g` | 7588 | 34232 | 3 | 41823 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/reforged_release.w3g` | 5829 | 7737 | 0 | 13566 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/reforged_truncated_playernames.w3g` | 10253 | 12155 | 0 | 22408 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/replay_fullobs.w3g` | 6 | 6 | 1 | 13 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/replay_obs_on_defeat.w3g` | 11 | 13 | 1 | 25 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/replay_randomhero_randomraces.w3g` | 17 | 22 | 1 | 40 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/replay_referee.w3g` | 6 | 7 | 1 | 14 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/replay_teamstogether.w3g` | 10 | 15 | 1 | 26 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/twistedmeadows.w3g` | 10 | 13 | 1 | 24 | 0 | 0 |
| `upstream/w3gjs/test/replays/132/wan_vs_trunks.w3g` | 2335 | 3441 | 0 | 5776 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/148249993_Edo_Leon_Tidehunters 12.w3g` | 7849 | 3926 | 2 | 11777 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/2.0.2-FloTVSavedByWc3.w3g` | 5827 | 7178 | 2 | 13007 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/2.0.2-LAN-bots.w3g` | 10 | 14 | 0 | 24 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/2.0.2-Melee.w3g` | 6 | 6 | 0 | 12 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/2661392198_PhoeNix_Changer_Concealed Hill.w3g` | 24429 | 10315 | 2 | 34746 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/3320738873_Changer_PhoeNix_Springtime 13.w3g` | 40519 | 11808 | 2 | 52329 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/455872485_PhoeNix_Changer_Hammerfall.w3g` | 29250 | 10727 | 2 | 39979 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/791786117_Edo_Leon_Springtime 13.w3g` | 22478 | 9036 | 2 | 31516 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/TempReplay.w3g` | 39863 | 2684 | 0 | 42547 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/goldmine test.w3g` | 7 | 7 | 1 | 15 | 0 | 0 |
| `upstream/w3gjs/test/replays/200/retrainingissues.w3g` | 14226 | 16551 | 0 | 30777 | 0 | 0 |

## Empirical Validation (2026-06-17): payload lengths confirmed from the corpus

The previously opaque/undecoded action ids above were carried with **guessed**
fixed payload lengths. Those guesses have now been validated against the full
50-replay corpus (`upstream/w3gjs/test/replays`, 478,581 command blocks) by two
independent methods, with the WC3 Reforged binary as a third anchor. The classic
[WarCraft III Replay Action Format Description](https://www.gamedevs.org/uploads/w3g_actions.txt)
corroborates the names and layouts for `0x50`, `0x60`, `0x62`, `0x69`, and
`0x6a`.

**Method.** A parameterized length-walker (`src/bin/w3grs-action-probe.rs`) replays
the exact command-block framing from `game_data.rs`, then for each block checks that
it consumes to its declared boundary with all-known action ids. Because a wrong
payload length desyncs the actions *within* a block (the framing force-resets at the
block boundary), two signals isolate the truth:

1. **Residue-when-last (model-independent):** when one of these candidate actions
   is the *final* action of a cleanly-parsed block, the bytes from its payload
   start to the block boundary *are* its payload — no length assumption involved.
2. **Whole-corpus length sweep:** vary one opaque length over 0..40, hold the rest at
   their guesses, and count clean blocks corpus-wide. The length that maximizes clean
   parsing is the empirical truth.

**Baseline (current lengths):** 478,548 / 478,581 blocks (**99.9931%**) parse to the
exact declared boundary with **zero unknown action ids**. The 33 remaining are
genuine end-of-stream truncations, not length errors — the parser never desyncs.

**Confirmed payloads:**

| Norm. id | Payload | Method-1 (residue-when-last) | Method-2 (sweep peak / runner-up) | Confidence |
|---:|---|---|---|---|
| `0x02` | 0 bytes (opcode only) | 13 samples, all len 0 | len0=478548 / len1=478535 | solid |
| `0x50` | 5 bytes (`ChangeAllyOptions`) | 15 samples, all len 5 | len5=478548 / len11=478541 | solid + spec-backed |
| `0x60` | 8 fixed + null-terminated string (`MapTriggerChatCommand`) | 242 samples, spread 10..73 | n/a (variable) | solid + spec-backed |
| `0x62` | 12 bytes (`ScenarioTrigger`) | 1,346 samples, all len 12 | len12=478548 / len25=476879 | strong + spec-backed |
| `0x69` | 16 bytes (`ContinueGameBlockB`) | 4 samples, all len 16 | len16=478548 / len17=478544 | good + spec-backed |
| `0x6a` | 16 bytes (`ContinueGameBlockA`) | none (never last) | len16=478548 (ties len33) | spec-backed |
| `0x7a` | 20 bytes | 480 samples, all len 20 | len20=478548 / len37=478068 | strong |
| `0x7b` | 16 bytes (`CommandCardSource`) | 18,795 samples, all len 16 | len16=478548 / len31=459585 | overwhelming |

**Scope of the confirmation.** This pins the **byte boundaries / sizes** decisively.
The classic spec gives names and layouts for `0x50`, `0x60`, `0x62`, `0x69`, and
`0x6a`, though several field meanings remain intentionally neutral in the public API.
The Reforged-era `0x7a` (20) and `0x7b` (16) are not documented there; `0x7a`
therefore stays opaque, while `0x7b` is decoded from corpus evidence as
`CommandCardSource` (`net_tag(8) + ability FourCC(4) + order FourCC(4)`).

**Binary anchor.** Static RE of the (Arxan-protected) Reforged client confirmed
`0x77` independently: `BlzSendSyncData` (RVA `0xc908f0`) serializes
`[0x77][prefix string\0][data string\0]`, matching the parser's `W3Api` handler.
Deeper binary-side decoding is a hard grind (indirect calls defeat call-graph
walking; no clean dispatcher jump table), so the empirical/corpus route above is the
better ROI for the remaining ids.

**Reproduce:**
```
cargo run --release --bin w3grs-action-probe --features extended-actions -- upstream/w3gjs/test/replays
```

**Sources:**

- [WarCraft III Replay Action Format Description](https://www.gamedevs.org/uploads/w3g_actions.txt)
- [WarCraft III Replay file format description](http://w3g.deepnode.de/files/w3g_format.txt)
