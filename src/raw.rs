//! Raw replay parser port.

use std::{io::Read, thread};

use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};

use crate::{
    buffer::StatefulBufferParser,
    error::{Error, Result},
};

const REPLAY_MAGIC: &[u8] = b"Warcraft III recorded game";
const FULL_DECOMPRESSED_BLOCK_SIZE: u16 = 8192;
const PARALLEL_DECOMPRESS_MIN_BLOCKS: usize = 192;
const PARALLEL_DECOMPRESS_MIN_BLOCKS_PER_WORKER: usize = 64;
const PARALLEL_DECOMPRESS_MIN_BYTES: usize = 1 << 20;
const PARALLEL_DECOMPRESS_MAX_WORKERS: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub compressed_size: u32,
    pub header_version: String,
    pub decompressed_size: u32,
    pub compressed_data_block_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubHeader {
    pub game_identifier: String,
    pub version: u32,
    pub build_no: u16,
    pub replay_length_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataBlock {
    pub block_size: u16,
    pub block_decompressed_size: u16,
    pub block_content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawReplayData {
    pub header: Header,
    pub subheader: SubHeader,
    pub blocks: Vec<DataBlock>,
}

#[derive(Debug, Default)]
pub struct RawParser;

impl RawParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &[u8]) -> Result<RawReplayData> {
        let mut parser = StatefulBufferParser::new(input);
        let header = parse_header(&mut parser)?;
        let subheader = parse_subheader(&mut parser)?;
        let blocks = parse_blocks(&mut parser, subheader.build_no)?;

        Ok(RawReplayData {
            header,
            subheader,
            blocks,
        })
    }
}

pub fn get_uncompressed_data(blocks: &[DataBlock]) -> Result<Vec<u8>> {
    let capacity = blocks
        .iter()
        .map(|block| block.block_decompressed_size as usize)
        .sum();
    let worker_count = parallel_decompress_worker_count(blocks.len(), capacity);
    if worker_count > 1 {
        return get_uncompressed_data_parallel(blocks, capacity, worker_count);
    }

    get_uncompressed_data_sequential(blocks, capacity)
}

fn get_uncompressed_data_sequential(blocks: &[DataBlock], capacity: usize) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(capacity);

    for block in blocks {
        decode_block_to_vec(block, &mut out)?;
    }

    Ok(out)
}

fn get_uncompressed_data_parallel(
    blocks: &[DataBlock],
    capacity: usize,
    worker_count: usize,
) -> Result<Vec<u8>> {
    let chunk_size = blocks.len().div_ceil(worker_count);
    let mut chunks = Vec::with_capacity(worker_count);

    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(worker_count);
        for chunk in blocks.chunks(chunk_size) {
            handles.push(scope.spawn(move || {
                let capacity = chunk
                    .iter()
                    .map(|block| block.block_decompressed_size as usize)
                    .sum();
                get_uncompressed_data_sequential(chunk, capacity)
            }));
        }

        for handle in handles {
            chunks.push(
                handle
                    .join()
                    .map_err(|_| Error::Message("decompression worker panicked".to_string()))??,
            );
        }

        Ok::<_, Error>(())
    })?;

    let mut out = Vec::with_capacity(capacity);
    for chunk in chunks {
        out.extend(chunk);
    }

    Ok(out)
}

fn decode_block_to_vec(block: &DataBlock, out: &mut Vec<u8>) -> Result<()> {
    if block.block_content.is_empty() {
        return Ok(());
    }

    let mut decoder = ZlibDecoder::new(block.block_content.as_slice());
    decoder.read_to_end(out)?;

    Ok(())
}

fn parallel_decompress_worker_count(blocks: usize, capacity: usize) -> usize {
    if blocks < PARALLEL_DECOMPRESS_MIN_BLOCKS || capacity < PARALLEL_DECOMPRESS_MIN_BYTES {
        return 1;
    }

    let available = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(PARALLEL_DECOMPRESS_MAX_WORKERS);
    available
        .min(blocks / PARALLEL_DECOMPRESS_MIN_BLOCKS_PER_WORKER)
        .max(1)
}

fn parse_header(parser: &mut StatefulBufferParser<'_>) -> Result<Header> {
    let offset = find_parse_start_offset(parser.buffer()).ok_or(Error::HeaderNotFound)?;
    parser.set_offset(offset);
    let _magic = parser.read_zero_term_string()?;
    parser.skip(4)?;
    let compressed_size = parser.read_u32_le()?;
    let header_version = parser.read_hex_string(4)?;
    let decompressed_size = parser.read_u32_le()?;
    let compressed_data_block_count = parser.read_u32_le()?;

    Ok(Header {
        compressed_size,
        header_version,
        decompressed_size,
        compressed_data_block_count,
    })
}

fn parse_subheader(parser: &mut StatefulBufferParser<'_>) -> Result<SubHeader> {
    let game_identifier = parser.read_string(4)?;
    let version = parser.read_u32_le()?;
    let build_no = parser.read_u16_le()?;
    parser.skip(2)?;
    let replay_length_ms = parser.read_u32_le()?;
    parser.skip(4)?;

    Ok(SubHeader {
        game_identifier,
        version,
        build_no,
        replay_length_ms,
    })
}

fn parse_blocks(parser: &mut StatefulBufferParser<'_>, build_no: u16) -> Result<Vec<DataBlock>> {
    let mut blocks = Vec::new();

    while !parser.is_done() {
        let block = parse_block(parser, build_no)?;
        if block.block_decompressed_size == FULL_DECOMPRESSED_BLOCK_SIZE {
            blocks.push(block);
        }
    }

    Ok(blocks)
}

fn parse_block(parser: &mut StatefulBufferParser<'_>, build_no: u16) -> Result<DataBlock> {
    let is_reforged = build_no >= 6089;
    let block_size = parser.read_u16_le()?;

    if is_reforged {
        parser.skip(2)?;
    }

    let block_decompressed_size = parser.read_u16_le()?;
    parser.skip(if is_reforged { 6 } else { 4 })?;
    let start = parser.offset();
    let end = start
        .saturating_add(block_size as usize)
        .min(parser.buffer().len());
    let block_content = parser.buffer()[start..end].to_vec();
    parser.set_offset(start.saturating_add(block_size as usize));

    Ok(DataBlock {
        block_size,
        block_decompressed_size,
        block_content,
    })
}

fn find_parse_start_offset(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(REPLAY_MAGIC.len())
        .position(|window| window == REPLAY_MAGIC)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reforged_raw_replay() {
        let bytes = include_bytes!("../fixtures/replays/132/reforged1.w3g");
        let raw = RawParser::new().parse(bytes).unwrap();

        assert_eq!(raw.subheader.game_identifier, "PX3W");
        assert_eq!(raw.subheader.version, 10032);
        assert_eq!(raw.subheader.build_no, 6091);
        assert!(!raw.blocks.is_empty());
        assert_eq!(
            raw.header.compressed_data_block_count as usize,
            raw.blocks.len()
        );

        let data = get_uncompressed_data(&raw.blocks).unwrap();
        assert!(!data.is_empty());
    }

    #[test]
    fn parses_classic_raw_replay() {
        let bytes = include_bytes!("../fixtures/replays/126/standard_126.w3g");
        let raw = RawParser::new().parse(bytes).unwrap();

        assert_eq!(raw.subheader.game_identifier, "PX3W");
        assert_eq!(raw.subheader.version, 26);
        assert_eq!(raw.subheader.build_no, 6059);
        assert!(!raw.blocks.is_empty());
    }

    #[test]
    fn rejects_truncated_raw_block_header() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"Warcraft III recorded game\0");
        bytes.extend_from_slice(&[0; 4]);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&[0; 4]);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(b"PX3W");
        bytes.extend_from_slice(&10032u32.to_le_bytes());
        bytes.extend_from_slice(&6091u16.to_le_bytes());
        bytes.extend_from_slice(&[0; 2]);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&[0; 4]);
        bytes.push(0x01);

        assert!(matches!(
            RawParser::new().parse(&bytes),
            Err(Error::UnexpectedEof { .. })
        ));
    }
}
