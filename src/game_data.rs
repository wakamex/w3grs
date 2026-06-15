//! Game data block parser port.

use crate::{
    action::{Action, ActionParser},
    buffer::StatefulBufferParser,
    error::{Error, Result},
};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum GameDataBlock {
    LeaveGame(LeaveGameBlock),
    Timeslot(TimeslotBlock),
    PlayerChatMessage(PlayerChatMessageBlock),
}

impl GameDataBlock {
    pub fn id(&self) -> u8 {
        match self {
            GameDataBlock::LeaveGame(_) => 0x17,
            GameDataBlock::Timeslot(block) => block.id,
            GameDataBlock::PlayerChatMessage(_) => 0x20,
        }
    }
}

impl Serialize for GameDataBlock {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            GameDataBlock::LeaveGame(block) => {
                let mut state = serializer.serialize_struct("LeaveGameBlock", 4)?;
                state.serialize_field("id", &0x17u8)?;
                state.serialize_field("playerId", &block.player_id)?;
                state.serialize_field("reason", &block.reason)?;
                state.serialize_field("result", &block.result)?;
                state.end()
            }
            GameDataBlock::Timeslot(block) => {
                let mut state = serializer.serialize_struct("TimeslotBlock", 3)?;
                state.serialize_field("id", &block.id)?;
                state.serialize_field("timeIncrement", &block.time_increment)?;
                state.serialize_field("commandBlocks", &block.command_blocks)?;
                state.end()
            }
            GameDataBlock::PlayerChatMessage(block) => {
                let mut state = serializer.serialize_struct("PlayerChatMessageBlock", 4)?;
                state.serialize_field("id", &0x20u8)?;
                state.serialize_field("playerId", &block.player_id)?;
                state.serialize_field("mode", &block.mode)?;
                state.serialize_field("message", &block.message)?;
                state.end()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveGameBlock {
    pub player_id: u8,
    pub reason: String,
    pub result: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeslotBlock {
    pub id: u8,
    pub time_increment: u16,
    pub command_blocks: Vec<CommandBlock>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandBlock {
    pub player_id: u8,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerChatMessageBlock {
    pub player_id: u8,
    pub mode: u32,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct GameDataParser;

impl GameDataParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(
        &self,
        data: &[u8],
        is_post_202_replay_format: bool,
    ) -> Result<Vec<GameDataBlock>> {
        let mut blocks = Vec::new();
        self.parse_with(data, is_post_202_replay_format, |block| {
            blocks.push(block);
            Ok(())
        })?;

        Ok(blocks)
    }

    pub fn parse_with<F>(
        &self,
        data: &[u8],
        is_post_202_replay_format: bool,
        mut visitor: F,
    ) -> Result<()>
    where
        F: FnMut(GameDataBlock) -> Result<()>,
    {
        let mut parser = StatefulBufferParser::new(data);

        while !parser.is_done() {
            match parse_block(&mut parser, is_post_202_replay_format) {
                Ok(Some(block)) => visitor(block)?,
                Ok(None) => {}
                Err(error) => return Err(error),
            }
        }

        Ok(())
    }
}

fn parse_block(
    parser: &mut StatefulBufferParser<'_>,
    is_post_202_replay_format: bool,
) -> Result<Option<GameDataBlock>> {
    let id = parser.read_u8()?;
    let block = match id {
        0x17 => Some(GameDataBlock::LeaveGame(parse_leave_game_block(parser)?)),
        0x1a..=0x1c => {
            parser.skip(4)?;
            None
        }
        0x1f | 0x1e => Some(GameDataBlock::Timeslot(parse_timeslot_block(
            parser,
            is_post_202_replay_format,
        )?)),
        0x20 => Some(GameDataBlock::PlayerChatMessage(parse_chat_message(
            parser,
        )?)),
        0x22 => {
            parse_unknown_0x22(parser)?;
            None
        }
        0x23 => {
            parser.skip(10)?;
            None
        }
        0x2f => {
            parser.skip(8)?;
            None
        }
        _ => None,
    };
    Ok(block)
}

fn parse_unknown_0x22(parser: &mut StatefulBufferParser<'_>) -> Result<()> {
    let length = parser.read_u8()?;
    parser.skip(length as isize)
}

fn parse_chat_message(parser: &mut StatefulBufferParser<'_>) -> Result<PlayerChatMessageBlock> {
    let player_id = parser.read_u8()?;
    let _byte_count = parser.read_u16_le()?;
    let flags = parser.read_u8()?;
    let mode = if flags == 0x20 {
        parser.read_u32_le()?
    } else {
        0
    };
    let message = parser.read_zero_term_string()?;
    Ok(PlayerChatMessageBlock {
        player_id,
        mode,
        message,
    })
}

fn parse_leave_game_block(parser: &mut StatefulBufferParser<'_>) -> Result<LeaveGameBlock> {
    let reason = parser.read_hex_string(4)?;
    let player_id = parser.read_u8()?;
    let result = parser.read_hex_string(4)?;
    parser.skip(4)?;
    Ok(LeaveGameBlock {
        player_id,
        reason,
        result,
    })
}

fn parse_timeslot_block(
    parser: &mut StatefulBufferParser<'_>,
    is_post_202_replay_format: bool,
) -> Result<TimeslotBlock> {
    let byte_count = parser.read_u16_le()? as usize;
    let time_increment = parser.read_u16_le()?;
    let action_block_last_offset = parser
        .offset()
        .checked_add(
            byte_count
                .checked_sub(2)
                .ok_or_else(|| Error::Message("timeslot block byte count underflow".to_string()))?,
        )
        .ok_or_else(|| Error::Message("timeslot block offset overflow".to_string()))?;
    let mut command_blocks = Vec::new();
    let mut action_parser = ActionParser::new();

    while parser.offset() < action_block_last_offset {
        let player_id = parser.read_u8()?;
        let action_block_length = parser.read_u16_le()? as usize;
        let action_start = parser.offset();
        let action_end = action_start
            .saturating_add(action_block_length)
            .min(parser.buffer().len());
        let actions = &parser.buffer()[action_start..action_end];
        let actions = action_parser.parse(actions, is_post_202_replay_format)?;
        parser.set_offset(action_start.saturating_add(action_block_length));
        command_blocks.push(CommandBlock { player_id, actions });
    }

    Ok(TimeslotBlock {
        id: 0x1f,
        time_increment,
        command_blocks,
    })
}

#[cfg(test)]
mod tests {
    use crate::{metadata::MetadataParser, raw::RawParser};

    use super::*;

    #[test]
    fn parses_game_data_blocks_from_fixture() {
        let bytes = include_bytes!("../fixtures/replays/132/netease_132.nwg");
        let raw = RawParser::new().parse(bytes).unwrap();
        let metadata = MetadataParser::new().parse(&raw.blocks).unwrap();
        let blocks = GameDataParser::new()
            .parse(&metadata.game_data, metadata.is_post_202_replay_format)
            .unwrap();

        let timeslots = blocks
            .iter()
            .filter(|block| matches!(block, GameDataBlock::Timeslot(_)))
            .count();
        assert!(timeslots > 50);
    }

    #[test]
    fn rejects_truncated_game_data_block() {
        let truncated_timeslot_header = [0x1f, 0x03];

        assert!(matches!(
            GameDataParser::new().parse(&truncated_timeslot_header, false),
            Err(Error::UnexpectedEof { .. })
        ));
    }
}
