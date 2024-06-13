//! Blockcache utility functionality.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

use crate::jsonrpc::connector::JsonRpcConnectorError;

/// Parser Error Type.
#[derive(Debug)]
pub enum ParseError {
    /// Io Error
    Io(std::io::Error),
    /// Invalid Data Error.
    InvalidData(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> ParseError {
        ParseError::Io(err)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(err) => write!(f, "IO Error: {}", err),
            ParseError::InvalidData(msg) => write!(f, "Invalid Data Error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Io(err) => Some(err),
            ParseError::InvalidData(_) => None,
        }
    }
}

impl From<JsonRpcConnectorError> for ParseError {
    fn from(err: JsonRpcConnectorError) -> ParseError {
        ParseError::InvalidData(err.to_string())
    }
}

/// Used for decoding zcash blocks from a bytestring.
pub trait ParseFromSlice {
    /// Reads data from a bytestring, consuming data read, and returns an instance of self along with the remaining data in the bytestring given.
    ///
    /// Txid is givin as an input as this is taken from a get_block verbose=1 call.
    fn parse_from_slice(
        data: &[u8],
        txid: Option<Vec<Vec<u8>>>,
    ) -> Result<(&[u8], Self), ParseError>
    where
        Self: Sized;
}

/// Skips the next n bytes in cursor, returns error message given if eof is reached.
pub fn skip_bytes(cursor: &mut Cursor<&[u8]>, n: usize, error_msg: &str) -> Result<(), ParseError> {
    if cursor.get_ref().len() < (cursor.position() + n as u64) as usize {
        return Err(ParseError::InvalidData(error_msg.to_string()));
    }
    cursor.set_position(cursor.position() + n as u64);
    Ok(())
}

/// Reads the next n bytes from cursor into a vec<u8>, returns error message given if eof is reached..
pub fn read_bytes(
    cursor: &mut Cursor<&[u8]>,
    n: usize,
    error_msg: &str,
) -> Result<Vec<u8>, ParseError> {
    let mut buf = vec![0; n];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| ParseError::InvalidData(error_msg.to_string()))?;
    Ok(buf)
}

/// Reads the next 8 bytes from cursor into a u64, returns error message given if eof is reached..
pub fn read_u64(cursor: &mut Cursor<&[u8]>, error_msg: &str) -> Result<u64, ParseError> {
    cursor
        .read_u64::<LittleEndian>()
        .map_err(ParseError::from)
        .map_err(|_| ParseError::InvalidData(error_msg.to_string()))
}

/// Reads the next 4 bytes from cursor into a u32, returns error message given if eof is reached..
pub fn read_u32(cursor: &mut Cursor<&[u8]>, error_msg: &str) -> Result<u32, ParseError> {
    cursor
        .read_u32::<LittleEndian>()
        .map_err(ParseError::from)
        .map_err(|_| ParseError::InvalidData(error_msg.to_string()))
}

/// Reads the next 4 bytes from cursor into an i32, returns error message given if eof is reached..
pub fn read_i32(cursor: &mut Cursor<&[u8]>, error_msg: &str) -> Result<i32, ParseError> {
    cursor
        .read_i32::<LittleEndian>()
        .map_err(ParseError::from)
        .map_err(|_| ParseError::InvalidData(error_msg.to_string()))
}

/// Reads the next byte from cursor into a bool, returns error message given if eof is reached..
pub fn read_bool(cursor: &mut Cursor<&[u8]>, error_msg: &str) -> Result<bool, ParseError> {
    let byte = cursor
        .read_u8()
        .map_err(ParseError::from)
        .map_err(|_| ParseError::InvalidData(error_msg.to_string()))?;
    match byte {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ParseError::InvalidData(error_msg.to_string())),
    }
}

/// read_zcash_script_int64 OP codes.
const OP_0: u8 = 0x00;
const OP_1_NEGATE: u8 = 0x4f;
const OP_1: u8 = 0x51;
const OP_16: u8 = 0x60;

/// Reads and interprets a Zcash (Bitcoin)-custom compact integer encoding used for int64 numbers in scripts.
pub fn read_zcash_script_i64(cursor: &mut Cursor<&[u8]>) -> Result<i64, ParseError> {
    let first_byte = read_bytes(cursor, 1, "Error reading first byte in i64 script hash")?[0];

    match first_byte {
        OP_1_NEGATE => Ok(-1),
        OP_0 => Ok(0),
        OP_1..=OP_16 => Ok((u64::from(first_byte) - u64::from(OP_1 - 1)) as i64),
        _ => {
            let num_bytes =
                read_bytes(cursor, first_byte as usize, "Error reading i64 script hash")?;
            let number = num_bytes
                .iter()
                .rev()
                .fold(0, |acc, &byte| (acc << 8) | u64::from(byte));
            Ok(number as i64)
        }
    }
}

/// Takes a vec of big endian hex encoded txids and returns them as a vec of little endian raw bytes.
pub fn display_txids_to_server(txids: Vec<String>) -> Vec<Vec<u8>> {
    txids
        .iter()
        .map(|txid| {
            txid.as_bytes()
                .chunks(2)
                .map(|chunk| {
                    let hex_pair = std::str::from_utf8(chunk).unwrap();
                    u8::from_str_radix(hex_pair, 16).unwrap()
                })
                .rev()
                .collect()
        })
        .collect()
}
