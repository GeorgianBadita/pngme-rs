use std::fmt::{Display, Formatter};

use anyhow::bail;
use crc::{Crc, CRC_32_ISO_HDLC};
use thiserror::Error;

use crate::chunk_type::ChunkType;

#[derive(Error, Debug)]
pub enum ChunkError {
    #[error("Expected 4 bytes for chunk length, less was provided")]
    InvalidLength,
    #[error("Expected 4 bytes for chunk type, less was provided")]
    InvalidChunkTypeLength,
    #[error("Length must not be greater than 2^31, {0} was provided")]
    LengthOverflow(u32),
    #[error("Data bytes do not match the specified length")]
    MismatchDataLength,
    #[error("Expected 4 bytes for crc length")]
    InvalidCrcLength,
    #[error("Mismatch between provided crc ({0}), and expected crc ({1})")]
    CrcMismatch(u32, u32),
}

pub(crate) struct Chunk {
    length: u32,
    data: Vec<u8>,
    chunk_type: ChunkType,
}

impl Chunk {
    pub(crate) fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        Chunk {
            chunk_type,
            length: data.len() as u32,
            data,
        }
    }

    pub(crate) fn length(&self) -> u32 {
        self.length
    }

    pub(crate) fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub(crate) fn data(&self) -> &[u8] {
        &self.data
    }

    pub(crate) fn crc(&self) -> u32 {
        let bytes: Vec<u8> = self.chunk_type
            .bytes()
            .iter()
            .chain(self.data.iter())
            .copied()
            .collect();
        let crc_alg = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        crc_alg.checksum(&bytes)
    }

    pub(crate) fn data_as_string(&self) -> anyhow::Result<String> {
        let s = std::str::from_utf8(&self.data)?;
        Ok(s.to_string())
    }

    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        self.length.
            to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.data.iter())
            .chain(self.crc().to_be_bytes().iter())
            .copied()
            .collect()
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Length: {}, Chunk type: {}, Data: {}, Crc: {}",
               self.length, self.chunk_type, self.data_as_string().unwrap(), self.crc())
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> anyhow::Result<Self> {
        let value = value.into_iter().map(|x| *x).collect::<Vec<u8>>();
        // First 4 bytes specifying the data length
        let length_bytes = value.get(0..4).ok_or(ChunkError::InvalidLength)?;
        // Next 4 bytes specifying chunk type
        let chunk_type_bytes = value.get(4..8).ok_or(ChunkError::InvalidChunkTypeLength)?;

        // Length as u32
        let length_u32: Vec<u32> = length_bytes.iter().map(|x| *x as u32).collect();
        let length = (length_u32[0] << 24) | (length_u32[1] << 16) | (length_u32[2] << 8) | length_u32[3];

        // Length > 2^31, error
        if length > (1 << 31) {
            bail!(ChunkError::LengthOverflow(length));
        }

        // Data bytes
        let data_bytes = if length > 0
        { value.get(8..8 + length as usize).ok_or(ChunkError::MismatchDataLength)? } else { &[] };

        // Crc bytes
        let crc_bytes = value.get(8 + length as usize..).ok_or(ChunkError::InvalidCrcLength)?;

        if crc_bytes.len() != 4 {
            bail!(ChunkError::InvalidCrcLength);
        }


        let crc_u32: Vec<u32> = crc_bytes.iter().map(|x| *x as u32).collect();
        let crc_num = (crc_u32[0] << 24) | (crc_u32[1] << 16) | (crc_u32[2] << 8) | crc_u32[3];


        let chunk_type = ChunkType::try_from(
            <[u8; 4]>::try_from(chunk_type_bytes).unwrap())?;

        if !chunk_type.is_valid() {
            bail!("Invalid chunk type {}", chunk_type);
        }

        let chunk = Chunk::new(chunk_type, data_bytes.to_vec());
        let chunk_crc = chunk.crc();
        if chunk_crc != crc_num {
            bail!(ChunkError::CrcMismatch(chunk_crc, crc_num));
        }

        Ok(chunk)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::chunk_type::ChunkType;

    use super::*;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}