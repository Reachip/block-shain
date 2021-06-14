extern crate sha2;
extern crate uuid;

use std::{
    cell::RefCell,
    error::Error,
    fs::{File, read_dir, remove_file},
    io::prelude::*,
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    rc::Rc,
};
use std::{env, io};

use serde::{Deserialize, Serialize};
use sha2::digest::generic_array::typenum::UInt;
use uuid::Uuid;

use crate::{block::Block};
use crate::hash::BlockHash;
use crate::network::{node::Node, signal::Signal};
use std::sync::{Mutex};

/// Represent a simple client on the blockchain network.
/// It can be a miner or a data provider.
pub struct Client {
    /// A unique ID for a client.
    pub client_socket_id: Uuid,
    /// TODO
    pub client_socket_path: String,
    /// The socket assigned for a client.
    pub client_socket: Mutex<UnixListener>,
    /// A vector which store all of the blocks provided by the network.
    blockchain: Mutex<Vec<Block>>,
}

impl Drop for Client {
    fn drop(&mut self) {
        remove_file(&self.client_socket_path).unwrap();
    }
}

impl Client {
    /// Create a new client.
    /// It create also a new socket assigned for the client.
    pub fn new(clients_sockets_location: String) -> io::Result<Client> {
        let client_socket_id = Uuid::new_v4();
        let client_socket_path = format!("{}{}.sock", clients_sockets_location, client_socket_id);
        let client = Client {
            client_socket_id,
            client_socket_path: client_socket_path.clone(),
            client_socket: Mutex::new(UnixListener::bind(client_socket_path)?),
            blockchain: Mutex::new(vec![]),
        };

        Ok(client)
    }

    /// Update "blockchain" field by given a new block.
    pub fn update_local_chain(&self, new_block: Block) {
        &self.blockchain.lock().unwrap().push(new_block);
    }

    /// Mine a block by changing the prof of work to find
    /// two zero in the current hash (BlockHash) of  the Block
    pub fn mine_block(&self, block: Block) {

    }

    /// Send a block on the network by given a recipient and some data.
    pub fn send_block(&self, to: &Client, data: &str) -> io::Result<()> {
        let block = Block::new(self.client_socket_id, data, BlockHash::default());
        let message = Signal::add_a_block(self, block.clone());
        &self._send_to_node(to.client_socket_path.clone(), message)?;
        Ok(())
    }

    /// Send a message to a specific node
    fn _send_to_node(&self, to: String, message: Signal) -> io::Result<()> {
        let mut streamer = UnixStream::connect(to)?;
        streamer.set_nonblocking(true).expect("Couldn't set nonblocking");
        let message_as_string = message.to_string().unwrap();
        streamer.write_all(message_as_string.as_bytes())?;
        Ok(())
    }

    /// Handle communication with other nodes
    pub fn respond_to_node(&self, signal: Signal) -> io::Result<()> {
        let listener = &self.client_socket.lock().unwrap();
        listener.set_nonblocking(true).expect("Cannot set non-blocking");

        for mut stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                stream.set_nonblocking(true).expect("Couldn't set nonblocking");
                let mut buffer = String::new();
                
                if let Ok(_) = stream.read_to_string(&mut buffer) {
                    let received_signal: Signal = serde_json::from_str(buffer.as_str()).unwrap();
                    self._send_to_node(received_signal.from_socket_path, Signal::is_okay(&self, true)).unwrap();
                }
            }

            break;
        }

        Ok(())
    }

    /// Fetch blocks provided by the network.
    pub fn fetch_blocks(&self, clients_sockets_location: String) -> Result<(), io::Error> {
        let mut socket_connections: usize = 0;
        let nodes = Node::get_nodes(clients_sockets_location, &self).unwrap();

        for node in nodes {
            if let Ok(stream) = UnixStream::connect(&node) {
                println!("comm with {:?}", node);
                socket_connections += 1;
                let mut streamer: UnixStream = stream;
                streamer.set_nonblocking(true).expect("Couldn't set nonblocking");

                let signal = Signal::new_miner(self).to_string()?;

                if let Ok(writer) = streamer.write_all(signal.as_bytes()) {
                    let mut response = String::new();
                    
                    if let Ok(_) = streamer.read_to_string(&mut response) {
                        println!("Receive : {}", response);
                    }
                } else {
                    println!("Receive anything");
                }
            }
        }

        if socket_connections < 0 {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "Connnection refused for all of the nodes in the network",
            ));
            
        }

        return Ok(());
        
    }

    /// Return the local blockchain on a vector
    pub fn send_local_blockchain(&self) -> Vec<Block> {
        let blockchain = &self.blockchain.lock().unwrap();
        blockchain.to_vec()
    }

    /// Check if a given block is valid
    pub fn is_a_valid_block(&self, block: Block) -> bool {
        true
    }
}
