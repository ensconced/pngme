use std::{
    fmt::{Display, Formatter},
    string::FromUtf8Error,
};

use crate::chunk_type::ChunkType;
use crc::crc32::checksum_ieee;

pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

pub struct TakenFrom {
    pub chunk: Chunk,
    pub bytes_remaining: u32,
}

fn four_bytes_from_slice(slice: &[u8]) -> Result<[u8; 4], ()> {
    if let Ok(result) = slice.try_into() {
        Ok(result)
    } else {
        Err(())
    }
}

impl Chunk {
    fn length(&self) -> u32 {
        self.length
    }
    fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }
    fn data_as_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }
    fn crc(&self) -> u32 {
        self.crc
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.length.to_be_bytes());
        bytes.extend(self.chunk_type.bytes());
        bytes.extend(&self.data);
        bytes.extend(self.crc.to_be_bytes());
        bytes
    }

    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let mut data_for_crc = Vec::new();
        data_for_crc.extend_from_slice(&chunk_type.bytes());
        data_for_crc.extend(data.iter());
        let crc = checksum_ieee(&data_for_crc);

        Self {
            length: data.len() as u32,
            chunk_type,
            data,
            crc,
        }
    }

    pub fn take_from(bytes: &[u8]) -> Result<TakenFrom, ()> {
        let first_four_bytes = four_bytes_from_slice(&bytes[0..4])?;
        let length = u32::from_be_bytes(first_four_bytes);
        let second_four_bytes = four_bytes_from_slice(&bytes[4..8])?;
        let chunk_type = ChunkType::try_from(second_four_bytes)?;
        let mut data = Vec::new();
        let crc_start = 8 + length as usize;
        data.extend_from_slice(&bytes[8..crc_start]);
        let provided_crc =
            u32::from_be_bytes(four_bytes_from_slice(&bytes[crc_start..bytes.len()])?);
        let computed_crc = checksum_ieee(&bytes[4..crc_start]);
        if provided_crc != computed_crc {
            return Err(());
        }
        Ok(TakenFrom {
            chunk: Self {
                length,
                chunk_type,
                data,
                crc: computed_crc,
            },
            bytes_remaining: bytes.len() as u32 - 4 - 4 - length as u32 - 4,
        })
    }
}

impl TryFrom<&Vec<u8>> for Chunk {
    type Error = ();
    fn try_from(bytes: &Vec<u8>) -> Result<Self, ()> {
        if let Ok(TakenFrom {
            chunk,
            bytes_remaining,
        }) = Self::take_from(&bytes[..])
        {
            if bytes_remaining == 0 {
                Ok(chunk)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

impl Display for Chunk {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        if let Ok(string) = self.data_as_string() {
            write!(fmt, "{}", string)
        } else {
            Err(std::fmt::Error)
        }
    }
}

#[cfg(test)]
mod tests {
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
