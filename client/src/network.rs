// client/src/network.rs
// RebornMP Network Client - Simplified Working Version

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
                println!("[Network] Connected to {}", server_ip);
                self.stream = Some(stream.try_clone().unwrap());
                self.connected = true;
                
                let send_queue = self.send_queue.clone();
                let mut send_stream = stream.try_clone().unwrap();
                
                thread::spawn(move || {
                    for msg in send_queue {
                        let _ = send_stream.write_all(msg.as_bytes());
                        let _ = send_stream.write_all(b"\n");
                        let _ = send_stream.flush();
                    }
                });
                
                let recv_queue = self.receive_queue.clone();
                let mut recv_stream = stream;
                
                thread::spawn(move || {
                    let mut buf = [0u8; 4096];
                    loop {
                        match recv_stream.read(&mut buf) {
                            Ok(n) if n > 0 => {
                                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                                let _ = recv_queue.send(msg);
                            }
                            Ok(_) => thread::sleep(Duration::from_millis(10)),
                            Err(_) => break,
                        }
                    }
                });
                
                true
            }
            Err(e) => {
                println!("[Network] Connect error: {}", e);
                false
            }
        }
    }
    
    pub fn send_position(&mut self, x: f32, y: f32, z: f32) {
        if self.connected {
            let _ = self.send_queue.send(format!("POS|{}|{}|{}", x, y, z));
        }
    }
    
    pub fn send_chat(&mut self, text: &str) {
        if self.connected && !text.is_empty() {
            let _ = self.send_queue.send(format!("CHAT|{}", text));
        }
    }
    
    pub fn send_command(&mut self, cmd: &str, args: &str) {
        if self.connected {
            let _ = self.send_queue.send(format!("CMD|{}|{}", cmd, args));
        }
    }
    
    pub fn receive_messages(&mut self) -> Vec<String> {
        let mut msgs = Vec::new();
        while let Ok(msg) = self.receive_queue.try_recv() {
            msgs.push(msg);
        }
        msgs
    }
    
    pub fn disconnect(&mut self) {
        self.connected = false;
        println!("[Network] Disconnected");
    }
    
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}
