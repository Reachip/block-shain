use std::collections::HashMap;
use std::hash::Hash;

use serde::Deserialize;
use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde::de::*;

use crate::block::Block;
use crate::client::Client;

use super::message::*;
use std::io;

#[derive(Debug, Deserialize)]
pub struct Signal {
    pub from_socket_path: String,
    pub key: Key,
    pub value: Value,
}

impl<'a> Serialize for Signal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut s = serializer.serialize_struct("Signal", 3)?;
        s.serialize_field("from_socket_path", &self.from_socket_path)?;
        s.serialize_field("key", &self.key)?;
        s.serialize_field("value", &self.value)?;
        s.end()
    }
}

impl Signal {
    fn new(from: &Client, key: Key, value: Value) -> Self {
        Signal {
            from_socket_path: from.client_socket_path.clone(),
            key,
            value,
        }
    }

    pub fn is_okay(from: &Client, is_okay: bool) -> Signal {
        Signal::new(from, Key::IsOkay, Value::BoolMessage(is_okay))
    }

    pub fn add_a_block(from: &Client, block: Block) -> Signal {
        Signal::new(from, Key::AddBlock, Value::Block(block))
    }

    pub fn is_this_block_conform(from: &Client, is_conform: bool) -> Signal {
        Signal::new(from, Key::IsThisBlockIsConform, Value::BoolMessage(is_conform))
    }

    pub fn finished_mining(from: &Client, block: Block) -> Signal {
        Signal::new(from, Key::FinishedMining, Value::Block(block))
    }

    pub fn new_miner(from: &Client) -> Signal {
        Signal::new(from, Key::NewMiner, Value::Nothing)
    }

    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}
