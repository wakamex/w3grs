//! Action parser port.

use crate::{
    buffer::StatefulBufferParser,
    error::{Error, Result},
};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};

pub type NetTag = [u32; 2];
pub type Vec2 = [f32; 2];
pub type FourCC = [u8; 4];

pub fn format_fourcc_or_hex(value: FourCC) -> String {
    if value[3].is_ascii_alphabetic() {
        value.iter().rev().map(|byte| *byte as char).collect()
    } else {
        format!(
            "0x{}",
            value
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        )
    }
}

pub fn format_net_tag(value: NetTag) -> String {
    format!("n:{}:{}", value[0], value[1])
}

pub fn format_optional_net_tag(value: NetTag) -> Option<String> {
    (value != [u32::MAX, u32::MAX]).then(|| format_net_tag(value))
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum Action {
    SetGameSpeed {
        game_speed: u8,
    },
    UnitBuildingAbilityNoParams {
        ability_flags: u16,
        order_id: FourCC,
    },
    UnitBuildingAbilityTargetPosition {
        ability_flags: u16,
        order_id: FourCC,
        target: Vec2,
    },
    UnitBuildingAbilityTargetPositionObject {
        ability_flags: u16,
        order_id: FourCC,
        target: Vec2,
        object: NetTag,
    },
    GiveItemToUnit {
        ability_flags: u16,
        order_id: FourCC,
        target: Vec2,
        unit: NetTag,
        item: NetTag,
    },
    UnitBuildingAbilityTwoTargetPositions {
        ability_flags: u16,
        order_id1: FourCC,
        target_a: Vec2,
        order_id2: FourCC,
        flags: u32,
        category: u32,
        owner: u8,
        target_b: Vec2,
    },
    UnitBuildingAbilityTargetPositionObjectItem {
        ability_flags: u16,
        order_id1: FourCC,
        target_a: Vec2,
        order_id2: FourCC,
        flags: u32,
        category: u32,
        owner: u8,
        target_b: Vec2,
        object: NetTag,
    },
    ChangeSelection {
        select_mode: u8,
        number_units: u16,
        units: Vec<NetTag>,
    },
    AssignGroupHotkey {
        group_number: u8,
        number_units: u16,
        units: Vec<NetTag>,
    },
    SelectGroupHotkey {
        group_number: u8,
    },
    SelectSubgroup {
        item_id: FourCC,
        object: NetTag,
    },
    PreSubselection,
    SelectUnit {
        object: NetTag,
    },
    SelectGroundItem {
        item: NetTag,
    },
    CancelHeroRevival {
        hero: NetTag,
    },
    RemoveUnitFromBuildingQueue {
        action_id: u8,
        slot_number: u8,
        item_id: FourCC,
    },
    TransferResources {
        slot: u8,
        gold: u32,
        lumber: u32,
    },
    EscPressed,
    TrackableHit {
        object: NetTag,
    },
    TrackableTrack {
        object: NetTag,
    },
    ChooseHeroSkillSubmenu,
    EnterBuildingSubmenu,
    AllyPing {
        pos: Vec2,
        duration: f32,
    },
    BlzCacheStoreInt {
        cache: Cache,
        value: u32,
    },
    BlzCacheStoreReal {
        cache: Cache,
        value: f32,
    },
    BlzCacheStoreBoolean {
        cache: Cache,
        value: u8,
    },
    BlzCacheStoreUnit {
        cache: Cache,
        value: Unit,
    },
    BlzCacheClearInt {
        cache: Cache,
    },
    BlzCacheClearReal {
        cache: Cache,
    },
    BlzCacheClearBoolean {
        cache: Cache,
    },
    BlzCacheClearUnit {
        cache: Cache,
    },
    ArrowKey {
        arrow_key: u8,
    },
    Mouse {
        event_id: u8,
        pos: Vec2,
        button: u8,
    },
    W3Api {
        command_id: u32,
        data: u32,
        buffer: String,
    },
    BlzSync {
        identifier: String,
        value: String,
    },
    CommandFrame {
        event_id: u32,
        val: f32,
        text: String,
    },
}

impl Action {
    pub fn id(&self) -> u8 {
        match self {
            Action::SetGameSpeed { .. } => 0x03,
            Action::UnitBuildingAbilityNoParams { .. } => 0x10,
            Action::UnitBuildingAbilityTargetPosition { .. } => 0x11,
            Action::UnitBuildingAbilityTargetPositionObject { .. } => 0x12,
            Action::GiveItemToUnit { .. } => 0x13,
            Action::UnitBuildingAbilityTwoTargetPositions { .. } => 0x14,
            Action::UnitBuildingAbilityTargetPositionObjectItem { .. } => 0x15,
            Action::ChangeSelection { .. } => 0x16,
            Action::AssignGroupHotkey { .. } => 0x17,
            Action::SelectGroupHotkey { .. } => 0x18,
            Action::SelectSubgroup { .. } => 0x19,
            Action::PreSubselection => 0x1a,
            Action::SelectUnit { .. } => 0x1b,
            Action::SelectGroundItem { .. } => 0x1c,
            Action::CancelHeroRevival { .. } => 0x1d,
            Action::RemoveUnitFromBuildingQueue { action_id, .. } => *action_id,
            Action::TransferResources { .. } => 0x51,
            Action::EscPressed => 0x61,
            Action::TrackableHit { .. } => 0x64,
            Action::TrackableTrack { .. } => 0x65,
            Action::ChooseHeroSkillSubmenu => 0x66,
            Action::EnterBuildingSubmenu => 0x67,
            Action::AllyPing { .. } => 0x68,
            Action::BlzCacheStoreInt { .. } => 0x6b,
            Action::BlzCacheStoreReal { .. } => 0x6c,
            Action::BlzCacheStoreBoolean { .. } => 0x6d,
            Action::BlzCacheStoreUnit { .. } => 0x6e,
            Action::BlzCacheClearInt { .. } => 0x70,
            Action::BlzCacheClearReal { .. } => 0x71,
            Action::BlzCacheClearBoolean { .. } => 0x72,
            Action::BlzCacheClearUnit { .. } => 0x73,
            Action::ArrowKey { .. } => 0x75,
            Action::Mouse { .. } => 0x76,
            Action::W3Api { .. } => 0x77,
            Action::BlzSync { .. } => 0x78,
            Action::CommandFrame { .. } => 0x79,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SummaryAction {
    UnitBuildingAbilityNoParams { order_id: FourCC },
    UnitBuildingAbilityTargetPosition { order_id: FourCC },
    UnitBuildingAbilityTargetPositionObject { order_id: FourCC },
    GiveItemToUnit,
    UnitBuildingAbilityTwoTargetPositions { order_id1: FourCC },
    ChangeSelection { select_mode: u8 },
    AssignGroupHotkey { group_number: u8 },
    SelectGroupHotkey { group_number: u8 },
    SelectGroundItem,
    CancelHeroRevival,
    RemoveUnitFromBuildingQueue,
    TransferResources { slot: u8, gold: u32, lumber: u32 },
    EscPressed,
    ChooseHeroSkillSubmenu,
    EnterBuildingSubmenu,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SummaryActionStats {
    pub actions: u64,
    pub emitted_actions: u64,
    pub ignored_actions: u64,
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("id", &self.id())?;

        match self {
            Action::SetGameSpeed { game_speed } => {
                map.serialize_entry("gameSpeed", game_speed)?;
            }
            Action::UnitBuildingAbilityNoParams {
                ability_flags,
                order_id,
            } => {
                map.serialize_entry("abilityFlags", ability_flags)?;
                map.serialize_entry("orderId", order_id)?;
            }
            Action::UnitBuildingAbilityTargetPosition {
                ability_flags,
                order_id,
                target,
            } => {
                map.serialize_entry("abilityFlags", ability_flags)?;
                map.serialize_entry("orderId", order_id)?;
                map.serialize_entry("target", target)?;
            }
            Action::UnitBuildingAbilityTargetPositionObject {
                ability_flags,
                order_id,
                target,
                object,
            } => {
                map.serialize_entry("abilityFlags", ability_flags)?;
                map.serialize_entry("orderId", order_id)?;
                map.serialize_entry("target", target)?;
                map.serialize_entry("object", object)?;
            }
            Action::GiveItemToUnit {
                ability_flags,
                order_id,
                target,
                unit,
                item,
            } => {
                map.serialize_entry("abilityFlags", ability_flags)?;
                map.serialize_entry("orderId", order_id)?;
                map.serialize_entry("target", target)?;
                map.serialize_entry("unit", unit)?;
                map.serialize_entry("item", item)?;
            }
            Action::UnitBuildingAbilityTwoTargetPositions {
                ability_flags,
                order_id1,
                target_a,
                order_id2,
                flags,
                category,
                owner,
                target_b,
            } => serialize_two_target_action(
                &mut map,
                TwoTargetActionFields {
                    ability_flags,
                    order_id1,
                    target_a,
                    order_id2,
                    flags,
                    category,
                    owner,
                    target_b,
                },
            )?,
            Action::UnitBuildingAbilityTargetPositionObjectItem {
                ability_flags,
                order_id1,
                target_a,
                order_id2,
                flags,
                category,
                owner,
                target_b,
                object,
            } => {
                serialize_two_target_action(
                    &mut map,
                    TwoTargetActionFields {
                        ability_flags,
                        order_id1,
                        target_a,
                        order_id2,
                        flags,
                        category,
                        owner,
                        target_b,
                    },
                )?;
                map.serialize_entry("object", object)?;
            }
            Action::ChangeSelection {
                select_mode,
                number_units,
                units,
            } => {
                map.serialize_entry("selectMode", select_mode)?;
                map.serialize_entry("numberUnits", number_units)?;
                map.serialize_entry("units", units)?;
            }
            Action::AssignGroupHotkey {
                group_number,
                number_units,
                units,
            } => {
                map.serialize_entry("groupNumber", group_number)?;
                map.serialize_entry("numberUnits", number_units)?;
                map.serialize_entry("units", units)?;
            }
            Action::SelectGroupHotkey { group_number } => {
                map.serialize_entry("groupNumber", group_number)?;
            }
            Action::SelectSubgroup { item_id, object } => {
                map.serialize_entry("itemId", item_id)?;
                map.serialize_entry("object", object)?;
            }
            Action::PreSubselection => {}
            Action::SelectUnit { object } => {
                map.serialize_entry("object", object)?;
            }
            Action::SelectGroundItem { item } => {
                map.serialize_entry("item", item)?;
            }
            Action::CancelHeroRevival { hero } => {
                map.serialize_entry("hero", hero)?;
            }
            Action::RemoveUnitFromBuildingQueue {
                slot_number,
                item_id,
                ..
            } => {
                map.serialize_entry("slotNumber", slot_number)?;
                map.serialize_entry("itemId", item_id)?;
            }
            Action::TransferResources { slot, gold, lumber } => {
                map.serialize_entry("slot", slot)?;
                map.serialize_entry("gold", gold)?;
                map.serialize_entry("lumber", lumber)?;
            }
            Action::EscPressed | Action::ChooseHeroSkillSubmenu | Action::EnterBuildingSubmenu => {}
            Action::TrackableHit { object } | Action::TrackableTrack { object } => {
                map.serialize_entry("object", object)?;
            }
            Action::AllyPing { pos, duration } => {
                map.serialize_entry("pos", pos)?;
                map.serialize_entry("duration", duration)?;
            }
            Action::BlzCacheStoreInt { cache, value } => {
                map.serialize_entry("cache", cache)?;
                map.serialize_entry("value", value)?;
            }
            Action::BlzCacheStoreReal { cache, value } => {
                map.serialize_entry("cache", cache)?;
                map.serialize_entry("value", value)?;
            }
            Action::BlzCacheStoreBoolean { cache, value } => {
                map.serialize_entry("cache", cache)?;
                map.serialize_entry("value", value)?;
            }
            Action::BlzCacheStoreUnit { cache, value } => {
                map.serialize_entry("cache", cache)?;
                map.serialize_entry("value", value)?;
            }
            Action::BlzCacheClearInt { cache }
            | Action::BlzCacheClearReal { cache }
            | Action::BlzCacheClearBoolean { cache }
            | Action::BlzCacheClearUnit { cache } => {
                map.serialize_entry("cache", cache)?;
            }
            Action::ArrowKey { arrow_key } => {
                map.serialize_entry("arrowKey", arrow_key)?;
            }
            Action::Mouse {
                event_id,
                pos,
                button,
            } => {
                map.serialize_entry("eventId", event_id)?;
                map.serialize_entry("pos", pos)?;
                map.serialize_entry("button", button)?;
            }
            Action::W3Api {
                command_id,
                data,
                buffer,
            } => {
                map.serialize_entry("commandId", command_id)?;
                map.serialize_entry("data", data)?;
                map.serialize_entry("buffer", buffer)?;
            }
            Action::BlzSync { identifier, value } => {
                map.serialize_entry("identifier", identifier)?;
                map.serialize_entry("value", value)?;
            }
            Action::CommandFrame {
                event_id,
                val,
                text,
            } => {
                map.serialize_entry("eventId", event_id)?;
                map.serialize_entry("val", val)?;
                map.serialize_entry("text", text)?;
            }
        }

        map.end()
    }
}

fn serialize_two_target_action<S>(
    map: &mut S,
    fields: TwoTargetActionFields<'_>,
) -> std::result::Result<(), S::Error>
where
    S: SerializeMap,
{
    map.serialize_entry("abilityFlags", fields.ability_flags)?;
    map.serialize_entry("orderId1", fields.order_id1)?;
    map.serialize_entry("targetA", fields.target_a)?;
    map.serialize_entry("orderId2", fields.order_id2)?;
    map.serialize_entry("flags", fields.flags)?;
    map.serialize_entry("category", fields.category)?;
    map.serialize_entry("owner", fields.owner)?;
    map.serialize_entry("targetB", fields.target_b)?;
    Ok(())
}

struct TwoTargetActionFields<'a> {
    ability_flags: &'a u16,
    order_id1: &'a FourCC,
    target_a: &'a Vec2,
    order_id2: &'a FourCC,
    flags: &'a u32,
    category: &'a u32,
    owner: &'a u8,
    target_b: &'a Vec2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cache {
    pub filename: String,
    pub mission_key: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub item_id: FourCC,
    pub charges: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ability {
    pub id: FourCC,
    pub level: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeroData {
    pub xp: u32,
    pub level: u32,
    pub skill_points: u32,
    pub proper_name_id: u32,
    pub str: u32,
    pub str_bonus: f32,
    pub agi: u32,
    pub speed_mod: f32,
    pub cooldown_mod: f32,
    pub agi_bonus: f32,
    pub intel: u32,
    pub int_bonus: f32,
    pub hero_abils: Vec<Ability>,
    pub max_life: f32,
    pub max_mana: f32,
    pub sight: f32,
    pub damage: Vec<u32>,
    pub defense: f32,
    pub control_groups: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub unit_id: FourCC,
    pub items: Vec<Item>,
    pub hero_data: HeroData,
}

#[derive(Debug, Default)]
pub struct ActionParser {
    old_action_id: u8,
}

impl ActionParser {
    pub fn new() -> Self {
        Self {
            old_action_id: u8::MAX,
        }
    }

    pub fn parse(&mut self, input: &[u8], is_post_202_replay_format: bool) -> Result<Vec<Action>> {
        let mut parser = StatefulBufferParser::new(input);
        let mut actions = Vec::new();

        while !parser.is_done() {
            let action_id = parser.read_u8()?;
            match self.parse_action(&mut parser, action_id, is_post_202_replay_format) {
                Ok(Some(action)) => actions.push(action),
                Ok(None) => {}
                Err(Error::UnexpectedEof { .. }) => break,
                Err(error) => return Err(error),
            }
        }

        Ok(actions)
    }

    pub(crate) fn parse_summary_with<F>(
        &mut self,
        input: &[u8],
        is_post_202_replay_format: bool,
        mut visitor: F,
    ) -> Result<()>
    where
        F: FnMut(SummaryAction) -> Result<()>,
    {
        let mut parser = StatefulBufferParser::new(input);

        while !parser.is_done() {
            let action_id = parser.read_u8()?;
            match self.parse_summary_action(&mut parser, action_id, is_post_202_replay_format) {
                Ok(Some(action)) => visitor(action)?,
                Ok(None) => {}
                Err(Error::UnexpectedEof { .. }) => break,
                Err(error) => return Err(error),
            }
        }

        Ok(())
    }

    pub(crate) fn parse_summary_with_stats<F>(
        &mut self,
        input: &[u8],
        is_post_202_replay_format: bool,
        stats: &mut SummaryActionStats,
        mut visitor: F,
    ) -> Result<()>
    where
        F: FnMut(SummaryAction) -> Result<()>,
    {
        let mut parser = StatefulBufferParser::new(input);

        while !parser.is_done() {
            let action_id = parser.read_u8()?;
            stats.actions += 1;
            match self.parse_summary_action(&mut parser, action_id, is_post_202_replay_format) {
                Ok(Some(action)) => {
                    stats.emitted_actions += 1;
                    visitor(action)?;
                }
                Ok(None) => stats.ignored_actions += 1,
                Err(Error::UnexpectedEof { .. }) => break,
                Err(error) => return Err(error),
            }
        }

        Ok(())
    }

    fn parse_action(
        &mut self,
        parser: &mut StatefulBufferParser<'_>,
        action_id: u8,
        is_post_202_replay_format: bool,
    ) -> Result<Option<Action>> {
        let action_id = normalize_action_id(action_id, is_post_202_replay_format);
        let result = self.parse_action_fields(parser, action_id)?;
        self.old_action_id = action_id;
        Ok(result)
    }

    fn parse_summary_action(
        &mut self,
        parser: &mut StatefulBufferParser<'_>,
        action_id: u8,
        is_post_202_replay_format: bool,
    ) -> Result<Option<SummaryAction>> {
        let action_id = normalize_action_id(action_id, is_post_202_replay_format);
        let result = match action_id {
            0x01 => {
                parser.skip(1)?;
                None
            }
            0x02 => None,
            0x03 => {
                parser.skip(1)?;
                None
            }
            0x04 | 0x05 => None,
            0x06 => {
                skip_zero_term_string(parser)?;
                skip_zero_term_string(parser)?;
                parser.skip(1)?;
                None
            }
            0x07 => {
                parser.skip(4)?;
                None
            }
            0x10 => {
                parser.skip(2)?;
                let order_id = read_fourcc(parser)?;
                parser.skip(8)?;
                Some(SummaryAction::UnitBuildingAbilityNoParams { order_id })
            }
            0x11 => {
                parser.skip(2)?;
                let order_id = read_fourcc(parser)?;
                parser.skip(16)?;
                Some(SummaryAction::UnitBuildingAbilityTargetPosition { order_id })
            }
            0x12 => {
                parser.skip(2)?;
                let order_id = read_fourcc(parser)?;
                parser.skip(24)?;
                Some(SummaryAction::UnitBuildingAbilityTargetPositionObject { order_id })
            }
            0x13 => {
                parser.skip(38)?;
                Some(SummaryAction::GiveItemToUnit)
            }
            0x14 => {
                parser.skip(2)?;
                let order_id1 = read_fourcc(parser)?;
                parser.skip(37)?;
                Some(SummaryAction::UnitBuildingAbilityTwoTargetPositions { order_id1 })
            }
            0x15 => {
                parser.skip(51)?;
                None
            }
            0x19 => {
                parser.skip(12)?;
                None
            }
            0x1a => None,
            0x1b => {
                parser.skip(9)?;
                None
            }
            0x16 => {
                let select_mode = parser.read_u8()?;
                let number_units = parser.read_u16_le()?;
                skip_selection_units(parser, number_units)?;
                Some(SummaryAction::ChangeSelection { select_mode })
            }
            0x17 => {
                let group_number = parser.read_u8()?;
                let number_units = parser.read_u16_le()?;
                skip_selection_units(parser, number_units)?;
                Some(SummaryAction::AssignGroupHotkey { group_number })
            }
            0x18 => {
                let group_number = parser.read_u8()?;
                parser.skip(1)?;
                Some(SummaryAction::SelectGroupHotkey { group_number })
            }
            0x1c => {
                parser.skip(9)?;
                Some(SummaryAction::SelectGroundItem)
            }
            0x1d => {
                parser.skip(8)?;
                Some(SummaryAction::CancelHeroRevival)
            }
            0x1e | 0x1f => {
                parser.skip(5)?;
                Some(SummaryAction::RemoveUnitFromBuildingQueue)
            }
            0x20 => None,
            0x21 => {
                parser.skip(8)?;
                None
            }
            0x22..=0x26 => None,
            0x27 | 0x28 => {
                parser.skip(5)?;
                None
            }
            0x29..=0x2c => None,
            0x2d => {
                parser.skip(5)?;
                None
            }
            0x2e => {
                parser.skip(4)?;
                None
            }
            0x2f => None,
            0x50 => {
                parser.skip(5)?;
                None
            }
            0x51 => {
                let slot = parser.read_u8()?;
                let gold = parser.read_u32_le()?;
                let lumber = parser.read_u32_le()?;
                Some(SummaryAction::TransferResources { slot, gold, lumber })
            }
            0x60 => {
                parser.skip(8)?;
                skip_zero_term_string(parser)?;
                None
            }
            0x61 => Some(SummaryAction::EscPressed),
            0x62 => {
                parser.skip(12)?;
                None
            }
            0x63 | 0x64 => {
                parser.skip(8)?;
                None
            }
            0x65 => {
                parser.skip(8)?;
                None
            }
            0x66 => Some(SummaryAction::ChooseHeroSkillSubmenu),
            0x67 => Some(SummaryAction::EnterBuildingSubmenu),
            0x68 => {
                parser.skip(12)?;
                None
            }
            0x69 | 0x6a => {
                parser.skip(16)?;
                None
            }
            0x6b | 0x6c => {
                skip_cache_desc(parser)?;
                parser.skip(4)?;
                None
            }
            0x6d => {
                skip_cache_desc(parser)?;
                parser.skip(1)?;
                None
            }
            0x6e => {
                skip_cache_desc(parser)?;
                skip_cache_unit(parser)?;
                None
            }
            0x70..=0x73 => {
                skip_cache_desc(parser)?;
                None
            }
            0x75 => {
                parser.skip(1)?;
                None
            }
            0x76 => {
                parser.skip(10)?;
                None
            }
            0x77 => {
                parser.skip(8)?;
                let buff_len = parser.read_u32_le()? as usize;
                skip_usize(parser, buff_len)?;
                None
            }
            0x78 => {
                skip_zero_term_string(parser)?;
                skip_zero_term_string(parser)?;
                parser.skip(4)?;
                None
            }
            0x79 => {
                parser.skip(16)?;
                skip_zero_term_string(parser)?;
                None
            }
            0x7a => {
                parser.skip(20)?;
                None
            }
            0x7b => {
                parser.skip(16)?;
                None
            }
            0xa0 => {
                parser.skip(14)?;
                None
            }
            0xa1 => {
                parser.skip(9)?;
                None
            }
            _ => None,
        };

        self.old_action_id = action_id;
        Ok(result)
    }

    fn parse_action_fields(
        &mut self,
        parser: &mut StatefulBufferParser<'_>,
        action_id: u8,
    ) -> Result<Option<Action>> {
        let result = match action_id {
            0x01 => {
                parser.skip(1)?;
                None
            }
            0x02 => None,
            0x03 => Some(Action::SetGameSpeed {
                game_speed: parser.read_u8()?,
            }),
            0x04 | 0x05 => None,
            0x06 => {
                let _ = parser.read_zero_term_string()?;
                let _ = parser.read_zero_term_string()?;
                let _ = parser.read_u8()?;
                None
            }
            0x07 => {
                parser.skip(4)?;
                None
            }
            0x10 => {
                let ability_flags = parser.read_u16_le()?;
                let order_id = read_fourcc(parser)?;
                parser.skip(8)?;
                Some(Action::UnitBuildingAbilityNoParams {
                    ability_flags,
                    order_id,
                })
            }
            0x11 => {
                let ability_flags = parser.read_u16_le()?;
                let order_id = read_fourcc(parser)?;
                parser.skip(8)?;
                let target = read_vec2(parser)?;
                Some(Action::UnitBuildingAbilityTargetPosition {
                    ability_flags,
                    order_id,
                    target,
                })
            }
            0x12 => {
                let ability_flags = parser.read_u16_le()?;
                let order_id = read_fourcc(parser)?;
                parser.skip(8)?;
                let target = read_vec2(parser)?;
                let object = read_net_tag(parser)?;
                Some(Action::UnitBuildingAbilityTargetPositionObject {
                    ability_flags,
                    order_id,
                    target,
                    object,
                })
            }
            0x13 => {
                let ability_flags = parser.read_u16_le()?;
                let order_id = read_fourcc(parser)?;
                parser.skip(8)?;
                let target = read_vec2(parser)?;
                let unit = read_net_tag(parser)?;
                let item = read_net_tag(parser)?;
                Some(Action::GiveItemToUnit {
                    ability_flags,
                    order_id,
                    target,
                    unit,
                    item,
                })
            }
            0x14 => {
                let ability_flags = parser.read_u16_le()?;
                let order_id1 = read_fourcc(parser)?;
                parser.skip(8)?;
                let target_a = read_vec2(parser)?;
                let order_id2 = read_fourcc(parser)?;
                let flags = parser.read_u32_le()?;
                let category = parser.read_u32_le()?;
                let owner = parser.read_u8()?;
                let target_b = read_vec2(parser)?;
                Some(Action::UnitBuildingAbilityTwoTargetPositions {
                    ability_flags,
                    order_id1,
                    target_a,
                    order_id2,
                    flags,
                    category,
                    owner,
                    target_b,
                })
            }
            0x15 => {
                let ability_flags = parser.read_u16_le()?;
                let order_id1 = read_fourcc(parser)?;
                parser.skip(8)?;
                let target_a = read_vec2(parser)?;
                let order_id2 = read_fourcc(parser)?;
                let flags = parser.read_u32_le()?;
                let category = parser.read_u32_le()?;
                let owner = parser.read_u8()?;
                let target_b = read_vec2(parser)?;
                let object = read_net_tag(parser)?;
                Some(Action::UnitBuildingAbilityTargetPositionObjectItem {
                    ability_flags,
                    order_id1,
                    target_a,
                    order_id2,
                    flags,
                    category,
                    owner,
                    target_b,
                    object,
                })
            }
            0x16 => {
                let select_mode = parser.read_u8()?;
                let number_units = parser.read_u16_le()?;
                let units = read_selection_units(parser, number_units)?;
                Some(Action::ChangeSelection {
                    select_mode,
                    number_units,
                    units,
                })
            }
            0x17 => {
                let group_number = parser.read_u8()?;
                let number_units = parser.read_u16_le()?;
                let units = read_selection_units(parser, number_units)?;
                Some(Action::AssignGroupHotkey {
                    group_number,
                    number_units,
                    units,
                })
            }
            0x18 => {
                let group_number = parser.read_u8()?;
                parser.skip(1)?;
                Some(Action::SelectGroupHotkey { group_number })
            }
            0x19 => {
                let item_id = read_fourcc(parser)?;
                let object = read_net_tag(parser)?;
                Some(Action::SelectSubgroup { item_id, object })
            }
            0x1a => Some(Action::PreSubselection),
            0x1b => {
                parser.skip(1)?;
                let object = read_net_tag(parser)?;
                Some(Action::SelectUnit { object })
            }
            0x1c => {
                parser.skip(1)?;
                let item = read_net_tag(parser)?;
                Some(Action::SelectGroundItem { item })
            }
            0x1d => {
                let hero = read_net_tag(parser)?;
                Some(Action::CancelHeroRevival { hero })
            }
            0x1e | 0x1f => {
                let slot_number = parser.read_u8()?;
                let item_id = read_fourcc(parser)?;
                Some(Action::RemoveUnitFromBuildingQueue {
                    action_id,
                    slot_number,
                    item_id,
                })
            }
            0x20 => None,
            0x21 => {
                parser.skip(8)?;
                None
            }
            0x22..=0x26 => None,
            0x27 | 0x28 => {
                parser.skip(5)?;
                None
            }
            0x29..=0x2c => None,
            0x2d => {
                parser.skip(5)?;
                None
            }
            0x2e => {
                parser.skip(4)?;
                None
            }
            0x2f => None,
            0x50 => {
                let _slot_number = parser.read_u8()?;
                let _flags = parser.read_u32_le()?;
                None
            }
            0x51 => {
                let slot = parser.read_u8()?;
                let gold = parser.read_u32_le()?;
                let lumber = parser.read_u32_le()?;
                Some(Action::TransferResources { slot, gold, lumber })
            }
            0x60 => {
                parser.skip(8)?;
                let _ = parser.read_zero_term_string()?;
                None
            }
            0x61 => Some(Action::EscPressed),
            0x62 => {
                parser.skip(12)?;
                None
            }
            0x63 => {
                parser.skip(8)?;
                None
            }
            0x64 => Some(Action::TrackableHit {
                object: read_net_tag(parser)?,
            }),
            0x65 => Some(Action::TrackableTrack {
                object: read_net_tag(parser)?,
            }),
            0x66 => Some(Action::ChooseHeroSkillSubmenu),
            0x67 => Some(Action::EnterBuildingSubmenu),
            0x68 => {
                let pos = read_vec2(parser)?;
                let duration = parser.read_f32_le()?;
                Some(Action::AllyPing { pos, duration })
            }
            0x69 | 0x6a => {
                parser.skip(16)?;
                None
            }
            0x6b => {
                let cache = read_cache_desc(parser)?;
                let value = parser.read_u32_le()?;
                Some(Action::BlzCacheStoreInt { cache, value })
            }
            0x6c => {
                let cache = read_cache_desc(parser)?;
                let value = parser.read_f32_le()?;
                Some(Action::BlzCacheStoreReal { cache, value })
            }
            0x6d => {
                let cache = read_cache_desc(parser)?;
                let value = parser.read_u8()?;
                Some(Action::BlzCacheStoreBoolean { cache, value })
            }
            0x6e => {
                let cache = read_cache_desc(parser)?;
                let value = read_cache_unit(parser)?;
                Some(Action::BlzCacheStoreUnit { cache, value })
            }
            0x70 => Some(Action::BlzCacheClearInt {
                cache: read_cache_desc(parser)?,
            }),
            0x71 => Some(Action::BlzCacheClearReal {
                cache: read_cache_desc(parser)?,
            }),
            0x72 => Some(Action::BlzCacheClearBoolean {
                cache: read_cache_desc(parser)?,
            }),
            0x73 => Some(Action::BlzCacheClearUnit {
                cache: read_cache_desc(parser)?,
            }),
            0x75 => Some(Action::ArrowKey {
                arrow_key: parser.read_u8()?,
            }),
            0x76 => {
                let event_id = parser.read_u8()?;
                let pos = read_vec2(parser)?;
                let button = parser.read_u8()?;
                Some(Action::Mouse {
                    event_id,
                    pos,
                    button,
                })
            }
            0x77 => {
                let command_id = parser.read_u32_le()?;
                let data = parser.read_u32_le()?;
                let buff_len = parser.read_u32_le()? as usize;
                let buffer = parser.read_string(buff_len)?;
                Some(Action::W3Api {
                    command_id,
                    data,
                    buffer,
                })
            }
            0x78 => {
                let identifier = parser.read_zero_term_string()?;
                let value = parser.read_zero_term_string()?;
                parser.skip(4)?;
                Some(Action::BlzSync { identifier, value })
            }
            0x79 => {
                parser.skip(8)?;
                let event_id = parser.read_u32_le()?;
                let val = parser.read_f32_le()?;
                let text = parser.read_zero_term_string()?;
                Some(Action::CommandFrame {
                    event_id,
                    val,
                    text,
                })
            }
            0x7a => {
                parser.skip(20)?;
                None
            }
            0x7b => {
                parser.skip(16)?;
                None
            }
            0xa0 => {
                parser.skip(14)?;
                None
            }
            0xa1 => {
                parser.skip(9)?;
                None
            }
            _ => None,
        };

        Ok(result)
    }
}

fn normalize_action_id(action_id: u8, is_post_202_replay_format: bool) -> u8 {
    if is_post_202_replay_format && action_id > 0x77 {
        action_id.saturating_add(1)
    } else {
        action_id
    }
}

fn read_selection_units(parser: &mut StatefulBufferParser<'_>, length: u16) -> Result<Vec<NetTag>> {
    let mut units = Vec::with_capacity(length as usize);
    for _ in 0..length {
        units.push(read_net_tag(parser)?);
    }
    Ok(units)
}

fn skip_selection_units(parser: &mut StatefulBufferParser<'_>, length: u16) -> Result<()> {
    parser.skip(isize::try_from(usize::from(length) * 8).expect("u16 * 8 fits in isize"))
}

fn skip_usize(parser: &mut StatefulBufferParser<'_>, byte_count: usize) -> Result<()> {
    let byte_count = isize::try_from(byte_count)
        .map_err(|_| Error::Message("skip length overflow".to_string()))?;
    parser.skip(byte_count)
}

fn skip_zero_term_string(parser: &mut StatefulBufferParser<'_>) -> Result<()> {
    let start = parser.offset();
    let remaining = &parser.buffer()[start..];
    let length = remaining
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(Error::UnexpectedEof {
            offset: start,
            needed: 1,
        })?;
    skip_usize(parser, length + 1)
}

fn read_fourcc(parser: &mut StatefulBufferParser<'_>) -> Result<FourCC> {
    Ok([
        parser.read_u8()?,
        parser.read_u8()?,
        parser.read_u8()?,
        parser.read_u8()?,
    ])
}

fn read_cache_desc(parser: &mut StatefulBufferParser<'_>) -> Result<Cache> {
    let filename = parser.read_zero_term_string()?;
    let mission_key = parser.read_zero_term_string()?;
    let key = parser.read_zero_term_string()?;
    Ok(Cache {
        filename,
        mission_key,
        key,
    })
}

fn skip_cache_desc(parser: &mut StatefulBufferParser<'_>) -> Result<()> {
    skip_zero_term_string(parser)?;
    skip_zero_term_string(parser)?;
    skip_zero_term_string(parser)
}

fn read_cache_item(parser: &mut StatefulBufferParser<'_>) -> Result<Item> {
    let item_id = read_fourcc(parser)?;
    let charges = parser.read_u32_le()?;
    let flags = parser.read_u32_le()?;
    Ok(Item {
        item_id,
        charges,
        flags,
    })
}

fn read_ability(parser: &mut StatefulBufferParser<'_>) -> Result<Ability> {
    let id = read_fourcc(parser)?;
    let level = parser.read_u32_le()?;
    Ok(Ability { id, level })
}

fn skip_cache_item(parser: &mut StatefulBufferParser<'_>) -> Result<()> {
    parser.skip(12)
}

fn skip_cache_ability(parser: &mut StatefulBufferParser<'_>) -> Result<()> {
    parser.skip(8)
}

fn read_cache_hero_data(parser: &mut StatefulBufferParser<'_>) -> Result<HeroData> {
    let xp = parser.read_u32_le()?;
    let level = parser.read_u32_le()?;
    let skill_points = parser.read_u32_le()?;
    let proper_name_id = parser.read_u32_le()?;
    let str = parser.read_u32_le()?;
    let str_bonus = parser.read_f32_le()?;
    let agi = parser.read_u32_le()?;
    let speed_mod = parser.read_f32_le()?;
    let cooldown_mod = parser.read_f32_le()?;
    let agi_bonus = parser.read_f32_le()?;
    let intel = parser.read_u32_le()?;
    let int_bonus = parser.read_f32_le()?;
    let hero_abil_count = parser.read_u32_le()?;
    let mut hero_abils = Vec::with_capacity(hero_abil_count as usize);
    for _ in 0..hero_abil_count {
        hero_abils.push(read_ability(parser)?);
    }
    let max_life = parser.read_f32_le()?;
    let max_mana = parser.read_f32_le()?;
    let sight = parser.read_f32_le()?;
    let damage_count = parser.read_u32_le()?;
    let mut damage = Vec::with_capacity(damage_count as usize);
    for _ in 0..damage_count {
        damage.push(parser.read_u32_le()?);
    }
    let defense = parser.read_f32_le()?;
    let control_groups = parser.read_u16_le()?;

    Ok(HeroData {
        xp,
        level,
        skill_points,
        proper_name_id,
        str,
        str_bonus,
        agi,
        speed_mod,
        cooldown_mod,
        agi_bonus,
        intel,
        int_bonus,
        hero_abils,
        max_life,
        max_mana,
        sight,
        damage,
        defense,
        control_groups,
    })
}

fn skip_cache_hero_data(parser: &mut StatefulBufferParser<'_>) -> Result<()> {
    parser.skip(48)?;
    let hero_abil_count = parser.read_u32_le()?;
    let hero_abil_count = usize::try_from(hero_abil_count)
        .map_err(|_| Error::Message("hero ability count overflow".to_string()))?;
    for _ in 0..hero_abil_count {
        skip_cache_ability(parser)?;
    }
    parser.skip(12)?;
    let damage_count = parser.read_u32_le()?;
    let damage_byte_count = usize::try_from(damage_count)
        .ok()
        .and_then(|count| count.checked_mul(4))
        .ok_or_else(|| Error::Message("damage count overflow".to_string()))?;
    skip_usize(parser, damage_byte_count)?;
    parser.skip(6)
}

fn read_cache_unit(parser: &mut StatefulBufferParser<'_>) -> Result<Unit> {
    let unit_id = read_fourcc(parser)?;
    let items_count = parser.read_u32_le()?;
    let mut items = Vec::with_capacity(items_count as usize);
    for _ in 0..items_count {
        items.push(read_cache_item(parser)?);
    }
    let hero_data = read_cache_hero_data(parser)?;
    Ok(Unit {
        unit_id,
        items,
        hero_data,
    })
}

fn skip_cache_unit(parser: &mut StatefulBufferParser<'_>) -> Result<()> {
    parser.skip(4)?;
    let items_count = parser.read_u32_le()?;
    let items_count = usize::try_from(items_count)
        .map_err(|_| Error::Message("cache item count overflow".to_string()))?;
    for _ in 0..items_count {
        skip_cache_item(parser)?;
    }
    skip_cache_hero_data(parser)
}

fn read_net_tag(parser: &mut StatefulBufferParser<'_>) -> Result<NetTag> {
    Ok([parser.read_u32_le()?, parser.read_u32_le()?])
}

fn read_vec2(parser: &mut StatefulBufferParser<'_>) -> Result<Vec2> {
    Ok([parser.read_f32_le()?, parser.read_f32_le()?])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{game_data::GameDataParser, metadata::MetadataParser, raw::RawParser};

    #[test]
    fn parses_actions_from_fixture_timeslots() {
        let bytes = include_bytes!("../fixtures/replays/132/netease_132.nwg");
        let raw = RawParser::new().parse(bytes).unwrap();
        let metadata = MetadataParser::new().parse(&raw.blocks).unwrap();
        let blocks = GameDataParser::new()
            .parse(&metadata.game_data, metadata.is_post_202_replay_format)
            .unwrap();

        let action_count = blocks
            .iter()
            .flat_map(|block| match block {
                crate::game_data::GameDataBlock::Timeslot(timeslot) => {
                    timeslot.command_blocks.iter().collect::<Vec<_>>()
                }
                _ => Vec::new(),
            })
            .map(|command| command.actions.len())
            .sum::<usize>();

        assert!(action_count > 0);
    }

    #[test]
    fn formats_fourcc_and_net_tags_for_downstream_adapters() {
        assert_eq!(format_fourcc_or_hex(*b"trah"), "hart");
        assert_eq!(format_fourcc_or_hex([0x03, 0, 0, 0]), "0x03000000");
        assert_eq!(format_net_tag([1, 2]), "n:1:2");
        assert_eq!(format_optional_net_tag([1, 2]), Some("n:1:2".to_string()));
        assert_eq!(format_optional_net_tag([u32::MAX, u32::MAX]), None);
    }
}
