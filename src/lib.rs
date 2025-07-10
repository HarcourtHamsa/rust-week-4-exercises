use std::{
    io::{Cursor, Read},
    str::FromStr,
};
use thiserror::Error;

// Custom errors for Bitcoin operations
#[derive(Error, Debug)]
pub enum BitcoinError {
    #[error("Invalid transaction format")]
    InvalidTransaction,
    #[error("Invalid script format")]
    InvalidScript,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Generic Point struct for Bitcoin addresses or coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        // TODO: Implement constructor for Point
        Point { x, y }
    }
}

// Custom serialization for Bitcoin transaction
pub trait BitcoinSerialize {
    fn serialize(&self) -> Vec<u8> {
        // TODO: Implement serialization to bytes
        todo!()
    }
}

// Legacy Bitcoin transaction
#[derive(Debug, Clone)]
pub struct LegacyTransaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl LegacyTransaction {
    pub fn builder() -> LegacyTransactionBuilder {
        // TODO: Return a new builder for constructing a transaction
        LegacyTransactionBuilder::default()
    }
}

// Transaction builder
pub struct LegacyTransactionBuilder {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl Default for LegacyTransactionBuilder {
    fn default() -> Self {
        // TODO: Implement default values
        LegacyTransactionBuilder {
            version: 1,
            inputs: vec![],
            outputs: vec![],
            lock_time: 0,
        }
    }
}

impl LegacyTransactionBuilder {
    pub fn new() -> Self {
        // TODO: Initialize new builder by calling default
        LegacyTransactionBuilder::default()
    }

    pub fn version(mut self, version: i32) -> Self {
        // TODO: Set the transaction version
        self.version = version;
        self
    }

    pub fn add_input(mut self, input: TxInput) -> Self {
        // TODO: Add input to the transaction
        self.inputs.push(input);
        self
    }

    pub fn add_output(mut self, output: TxOutput) -> Self {
        // TODO: Add output to the transaction
        self.outputs.push(output);
        self
    }

    pub fn lock_time(mut self, lock_time: u32) -> Self {
        // TODO: Set lock_time for transaction
        self.lock_time = lock_time;
        self
    }

    pub fn build(self) -> LegacyTransaction {
        // TODO: Build and return the final LegacyTransaction
        let v = self;

        LegacyTransaction {
            outputs: v.outputs,
            version: v.version,
            inputs: v.inputs,
            lock_time: v.lock_time,
        }
    }
}

// Transaction components
#[derive(Debug, Clone)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value: u64, // in satoshis
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub vout: u32,
}

// Simple CLI argument parser
pub fn parse_cli_args(args: &[String]) -> Result<CliCommand, BitcoinError> {
    // TODO: Match args to "send" or "balance" commands and parse required arguments

    let [command, amount, address] = &args[..] else {
        return Err(BitcoinError::ParseError("Too few args".to_string()));
    };

    match command.as_str() {
        "send" => {
            let parsed_amount = u64::from_str(amount.as_str())
                .map_err(|_| BitcoinError::ParseError("Failed to parse amount".to_string()))?;

            Ok(CliCommand::Send {
                amount: parsed_amount,
                address: address.to_owned(),
            })
        }
        "balance" => Ok(CliCommand::Balance),
        _ => Err(BitcoinError::ParseError(
            "Failed to parse amount".to_string(),
        )),
    }
}

pub enum CliCommand {
    Send { amount: u64, address: String },
    Balance,
}

pub fn read_compact_size(cursor: &mut Cursor<&[u8]>) -> u64 {
    let mut buffer = [0u8; 1]; // marker
    let _ = cursor
        .read_exact(&mut buffer)
        .map_err(|_| BitcoinError::InvalidScript);

    let first_byte = buffer[0];

    match first_byte {
        0..=252 => first_byte as u64,
        253 => {
            let mut buffer = [0u8; 2];
            let _ = cursor
                .read_exact(&mut buffer)
                .map_err(|_| BitcoinError::InvalidScript);

            u16::from_le_bytes(buffer) as u64
        }
        254 => {
            let mut buffer = [0u8; 4];
            let _ = cursor
                .read_exact(&mut buffer)
                .map_err(|_| BitcoinError::InvalidScript);

            u32::from_le_bytes(buffer) as u64
        }
        255 => {
            let mut buffer = [0u8; 8];
            let _ = cursor
                .read_exact(&mut buffer)
                .map_err(|_| BitcoinError::InvalidScript);

            u64::from_le_bytes(buffer)
        }
    }
}

pub fn read_scriptsig(length: u64, cursor: &mut Cursor<&[u8]>) -> Vec<u8> {
    let mut buffer = vec![0u8; length as usize];
    let _ = cursor.read_exact(&mut buffer);

    buffer
}

// Decoding legacy transaction
impl TryFrom<&[u8]> for LegacyTransaction {
    type Error = BitcoinError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // TODO: Parse binary data into a LegacyTransaction
        // Minimum length is 10 bytes (4 version + 4 inputs count + 4 lock_time)
        if data.len() < 10 {
            Err(BitcoinError::InvalidTransaction)
        } else {
            let mut cursor = Cursor::new(data);

            let mut version_buffer = [0u8; 4];
            let _ = cursor
                .read_exact(&mut version_buffer)
                .map_err(|_| BitcoinError::InvalidScript);

            let version = i32::from_le_bytes(version_buffer);

            let num_inputs = read_compact_size(&mut cursor);

            let mut tx_inputs: Vec<TxInput> = Vec::with_capacity(num_inputs as usize);

            for _ in 0..num_inputs {
                let mut txid_buffer = [0u8; 32];
                let _ = cursor
                    .read_exact(&mut txid_buffer)
                    .map_err(|_| BitcoinError::InvalidScript);

                txid_buffer.reverse(); // natural order

                let mut vout_buffer = [0u8; 4];
                let _ = cursor
                    .read_exact(&mut vout_buffer)
                    .map_err(|_| BitcoinError::InvalidScript);

                let vout = u32::from_le_bytes(vout_buffer);

                let outpoint = OutPoint {
                    txid: txid_buffer,
                    vout,
                };

                let scriptsig_size = read_compact_size(&mut cursor);
                let script_sig = read_scriptsig(scriptsig_size, &mut cursor);

                let mut sequence_buffer = [0u8; 4];
                let _ = cursor
                    .read_exact(&mut sequence_buffer)
                    .map_err(|_| BitcoinError::InvalidScript);

                let sequence = u32::from_le_bytes(sequence_buffer);

                tx_inputs.push(TxInput {
                    previous_output: outpoint,
                    script_sig,
                    sequence,
                });
            }

            let num_ouputs = read_compact_size(&mut cursor);

            let mut tx_outputs: Vec<TxOutput> = vec![];

            for _ in 0..num_ouputs {
                let mut amount_buffer = [0u8; 8];
                let _ = cursor
                    .read_exact(&mut amount_buffer)
                    .map_err(|_| BitcoinError::InvalidScript);

                let amount = u64::from_le_bytes(amount_buffer);

                let script_pubkey_size = read_compact_size(&mut cursor);
                let script_pubkey = read_scriptsig(script_pubkey_size, &mut cursor);

                tx_outputs.push(TxOutput {
                    value: amount,
                    script_pubkey,
                });
            }

            let mut lock_time_buffer = [0u8; 4];

            let _ = cursor
                .read_exact(&mut lock_time_buffer)
                .map_err(|_| BitcoinError::InvalidScript);

            let lock_time = u32::from_le_bytes(lock_time_buffer);

            Ok(LegacyTransaction {
                lock_time,
                inputs: tx_inputs,
                outputs: tx_outputs,
                version,
            })
        }
    }
}

// Custom serialization for transaction
impl BitcoinSerialize for LegacyTransaction {
    fn serialize(&self) -> Vec<u8> {
        // TODO: Serialize only version and lock_time (simplified)
        let mut buffer = Vec::new();
        let version_bytes = &self.version.to_le_bytes();
        buffer.extend_from_slice(version_bytes);

        let lock_time_bytes = &self.lock_time.to_le_bytes();
        buffer.extend_from_slice(lock_time_bytes);

        buffer
    }
}
