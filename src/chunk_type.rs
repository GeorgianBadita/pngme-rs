use std::fmt::{Display, Formatter};
use std::str;
use std::str::FromStr;

use anyhow::bail;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChunkTypeError {
    #[error("Wrong length for constructing chunk from string, expected 4, got {0}")]
    WrongStringByteLength(usize),
    #[error("Only lowercase [65-90] and uppercase [97-122] bytes are allowed, got: {0}")]
    InvalidChunkByte(u8),
}

#[derive(PartialEq, Debug, Eq, Clone)]
pub struct ChunkType {
    num: u32,
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", str::from_utf8(&self.bytes()).unwrap())
    }
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        [0, 1, 2, 3].iter()
            .map(|x| self.nth_byte(*x as usize).unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap()
    }

    fn is_critical(&self) -> bool {
        return self.nth_byte(0).unwrap() & (1 << 5) == 0;
    }

    fn is_public(&self) -> bool {
        return self.nth_byte(1).unwrap() & (1 << 5) == 0;
    }

    fn is_reserved_bit_valid(&self) -> bool {
        return self.nth_byte(2).unwrap() & (1 << 5) == 0;
    }

    fn is_safe_to_copy(&self) -> bool {
        return self.nth_byte(3).unwrap() & (1 << 5) != 0;
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }

    fn nth_byte(&self, idx: usize) -> anyhow::Result<u8> {
        if idx > 4 {
            bail!("Missing attribute: {}", 1);
        }
        return Ok((self.num >> (24 - idx * 8)) as u8);
    }

    fn verify_and_map_byte(byte: u8) -> anyhow::Result<u32> {
        let ch = byte as char;
        if ('a' <= ch && ch <= 'z') || ('A' <= ch && ch <= 'Z') {
            return Ok(byte as u32);
        }
        bail!(ChunkTypeError::InvalidChunkByte(byte));
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = anyhow::Error;

    fn try_from(value: [u8; 4]) -> anyhow::Result<Self> {
        let mut mapped_to_u32 = Vec::new();
        for b in value {
            mapped_to_u32.push(ChunkType::verify_and_map_byte(b)?)
        }

        Ok(ChunkType {
            num: ((mapped_to_u32[0] << 24) | (mapped_to_u32[1] << 16) | (mapped_to_u32[2] << 8) | mapped_to_u32[3])
        })
    }
}

impl FromStr for ChunkType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<ChunkType> {
        if s.len() != 4 {
            bail!(ChunkTypeError::WrongStringByteLength(s.len()));
        }
        let bytes = <[u8; 4]>::try_from(s.as_bytes()).unwrap();
        ChunkType::try_from(bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::str::FromStr;

    use super::*;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}