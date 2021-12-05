//! # Respirator - cleanly inhale RESP stream.
//! Respirator is [nom](https://github.com/Geal/nom) based [Redis Serialization Protocol (resp)](https://redis.io/topics/protocol) parser. Currently only "complete" parsing (i.e. works only when all data to parse is available) works, yet an aim is to cover streaming parsing as well.
//!
//! ## Usage
//! ### Example
//! ```
//! use respirator::{Resp, resp};
//! use std::matches;
//!
//! let input = &b"*2\r\n$2\r\nOK\r\n$4\r\nResp\r\n"[..];
//! let (_, parsed) = resp(input).unwrap();
//! if let Resp::Array(Some(values)) = parsed {
//!   let simple_string = &values[0];
//!   let bulk_string = &values[1];
//!   assert!(matches!(Resp::SimpleString(b"OK".to_vec()), simple_string));
//!   assert!(matches!(Resp::BulkString(Some(b"Resp".to_vec())), bulk_string));
//! }
//! ```
pub mod parser;

pub use parser::{resp, Resp};
