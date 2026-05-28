// client/src/network.rs

use std::net::TcpStream;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::Duration;

pub struct NetworkClient {
    stream: Option<TcpStream>,
    send_queue: Sender<String>,
    receive_queue: Receiver<String>,
    connected: bool,
}

impl NetworkClient {
    pub fn new() -> Self {
        let (send_tx, _send_rx) = channel();
        let (_recv_tx, recv_rx) = channel();
        
        NetworkClient {
            stream: None,
            send_queue: send_tx,
            receive_queue: recv_rx,
            connected: false,
        }
    }
    
    pub fn connect(&mut self, server_ip: &str) -> bool {
        match TcpStream::connect(server_ip) {
            Ok(stream) => {
                self.stream = Some(stream.try_clone().unwrap());
                self.connected = true;
                true
            }
            Err(e) => {
                println!("[Network] Error: {}", e);
                false
            }
        }
    }
    
    pub fn send_chat(&mut self, text: &str) {
        if self.connected {
            if let Some(stream) = &mut self.stream {
                let _ = stream.write_all(format!("CHAT|{}\n", text).as_bytes());
            }
        }
    }
    
    pub fn receive_messages(&mut self) -> Vec<String> {
        Vec::new()
    }
    
    pub fn disconnect(&mut self) {
        self.connected = false;
    }
}
