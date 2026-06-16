use std::path::{Path, PathBuf};

use serde_json::Value;
#[cfg(feature = "extended-actions")]
use w3grs::action::Action;
use w3grs::{ReplayParser, W3GReplay, game_data::GameDataBlock, replay::ObserverMode};

fn replay_fixtures() -> Vec<PathBuf> {
    let Some(root) = upstream_replay_root() else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for version_dir in std::fs::read_dir(root).unwrap() {
        let version_dir = version_dir.unwrap();
        if !version_dir.file_type().unwrap().is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(version_dir.path()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("w3g" | "nwg")
            ) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn upstream_replay_root() -> Option<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("upstream/w3gjs/test/replays");
    root.exists().then_some(root)
}

fn upstream_replay_path(relative: &str) -> Option<PathBuf> {
    upstream_replay_root().map(|root| root.join(relative))
}

#[test]
fn parses_all_upstream_replay_fixtures() {
    let fixtures = replay_fixtures();
    if fixtures.is_empty() {
        return;
    }

    let mut parser = W3GReplay::new();
    let mut parsed = 0;

    for fixture in fixtures {
        parser
            .parse_file(&fixture)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", fixture.display()));
        parsed += 1;
    }

    assert!(parsed > 40);
}

#[test]
fn matches_selected_upstream_high_level_expectations() {
    let Some(root) = upstream_replay_path("132") else {
        return;
    };
    let mut parser = W3GReplay::new();

    let release = parser
        .parse_file(root.join("reforged_release.w3g"))
        .unwrap();
    assert_eq!(release.version, "1.32");
    assert_eq!(release.build_number, 6105);
    assert_eq!(release.players.len(), 2);
    assert_eq!(release.players[0].name, "anXieTy#2932");
    assert_eq!(release.players[1].name, "IroNSoul#22724");
    assert_eq!(release.winning_team_id, 0);

    let random = parser
        .parse_file(root.join("replay_randomhero_randomraces.w3g"))
        .unwrap();
    assert!(random.settings.random_hero);
    assert!(random.settings.random_races);
    assert_eq!(random.winning_team_id, 0);

    let full_obs = parser.parse_file(root.join("replay_fullobs.w3g")).unwrap();
    assert_eq!(full_obs.settings.observer_mode, ObserverMode::Full);

    let referee = parser.parse_file(root.join("replay_referee.w3g")).unwrap();
    assert_eq!(referee.settings.observer_mode, ObserverMode::Referees);
}

#[test]
fn serializes_high_level_output_with_w3gjs_field_names() {
    let Some(fixture) = upstream_replay_path("132/replay_fullobs.w3g") else {
        return;
    };
    let mut parser = W3GReplay::new();
    let output = parser.parse_file(fixture).unwrap();
    let json = serde_json::to_value(&output).unwrap();

    assert!(json.get("gamename").is_some());
    assert!(json.get("randomseed").is_some());
    assert!(json.get("startSpots").is_some());
    assert!(json.get("buildNumber").is_some());
    assert!(json.get("parseTime").is_some());
    assert!(json.get("winningTeamId").is_some());
    assert!(json.get("game_name").is_none());
    assert!(json.get("random_seed").is_none());
    assert!(json.get("game_type").is_none());
    assert_eq!(json["type"], Value::String("1on1".to_string()));
    assert_eq!(json["settings"]["observerMode"], "FULL");
    assert!(json["settings"].get("fullSharedUnitControl").is_some());
    assert!(json["map"].get("checksumSha1").is_some());

    let first_player = &json["players"][0];
    assert!(first_player.get("raceDetected").is_some());
    assert!(first_player.get("groupHotkeys").is_some());
    assert!(first_player.get("resourceTransfers").is_some());
    assert!(first_player.get("current_time_played").is_none());
}

#[test]
fn serializes_low_level_output_with_w3gjs_event_shapes() {
    let Some(fixture) = upstream_replay_path("132/netease_132.nwg") else {
        return;
    };
    let bytes = std::fs::read(fixture).unwrap();
    let output = ReplayParser::new().parse(&bytes).unwrap();
    let json = serde_json::to_value(&output).unwrap();

    assert!(json["header"].get("compressedSize").is_some());
    assert!(json["subheader"].get("gameIdentifier").is_some());
    assert!(json["metadata"].get("playerRecords").is_some());
    assert!(json["metadata"]["map"].get("mapChecksumSha1").is_some());
    assert!(json.get("gameDataBlocks").is_some());

    let timeslot = output
        .game_data_blocks
        .iter()
        .find_map(|block| match block {
            GameDataBlock::Timeslot(timeslot) if !timeslot.command_blocks.is_empty() => {
                Some(timeslot)
            }
            _ => None,
        })
        .unwrap();
    let block_json = serde_json::to_value(GameDataBlock::Timeslot(timeslot.clone())).unwrap();
    assert_eq!(block_json["id"], 31);
    assert!(block_json.get("timeIncrement").is_some());
    assert!(block_json.get("commandBlocks").is_some());

    let action = timeslot
        .command_blocks
        .iter()
        .flat_map(|command| &command.actions)
        .next()
        .unwrap();
    let action_json = serde_json::to_value(action).unwrap();
    assert!(action_json.get("id").and_then(Value::as_u64).is_some());
    assert!(!action_json.as_object().unwrap().is_empty());
}

#[test]
#[cfg(feature = "extended-actions")]
fn extended_actions_surface_command_card_source_in_timed_stream() {
    let Some(fixture) = upstream_replay_path("132/reforged2010.w3g") else {
        return;
    };
    let bytes = std::fs::read(fixture).unwrap();
    let output = ReplayParser::new().parse(&bytes).unwrap();

    let timed = output
        .iter_timed_actions()
        .find(|timed| matches!(timed.action, Action::CommandCardSource { .. }))
        .expect("expected a command-card source action in timed stream");

    let Action::CommandCardSource {
        source_unit_tag,
        ability_id,
        order_id,
        raw_opcode,
        normalized_opcode,
    } = timed.action
    else {
        panic!("expected command-card source action");
    };

    assert_eq!(*normalized_opcode, 0x7b);
    assert!(matches!(*raw_opcode, 0x7a | 0x7b));
    assert_ne!(*source_unit_tag, [0, 0]);
    assert_ne!(*ability_id, [0, 0, 0, 0]);
    assert_ne!(*order_id, [0, 0, 0, 0]);
    assert_eq!(timed.block_id, 0x1f);
    assert!(timed.sequence > 0);
}

#[test]
#[cfg(feature = "extended-actions")]
fn extended_actions_surface_opaque_dropped_actions_in_timed_stream() {
    let Some(fixture) = upstream_replay_path("131/action0x7a.w3g") else {
        return;
    };
    let bytes = std::fs::read(fixture).unwrap();
    let output = ReplayParser::new().parse(&bytes).unwrap();

    let timed = output
        .iter_timed_actions()
        .find(|timed| {
            matches!(
                timed.action,
                Action::OpaqueDroppedAction {
                    normalized_opcode: 0x7a,
                    payload,
                    ..
                } if payload.len() == 20
            )
        })
        .expect("expected an opaque dropped 0x7a action in timed stream");

    let Action::OpaqueDroppedAction {
        raw_opcode,
        normalized_opcode,
        payload,
    } = timed.action
    else {
        panic!("expected opaque dropped action");
    };

    assert_eq!(*normalized_opcode, 0x7a);
    assert!(matches!(*raw_opcode, 0x79 | 0x7a));
    assert_eq!(payload.len(), 20);
    assert!(timed.sequence > 0);
}
