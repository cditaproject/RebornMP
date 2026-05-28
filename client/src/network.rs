// client/src/network.rs
// RebornMP Network Client - Full Version

use std::net::TcpStream;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

pub struct NetworkClient {
    stream: Option<TcpStream>,
    send_queue: Sender<String>,
    receive_queue: Receiver<String>,
    connected: bool,
}

impl NetworkClient {
    pub fn new() -> Self {
        let (send_tx, send_rx) = channel();
        let (recv_tx, recv_rx) = channel();
        
        NetworkClient {
            stream: None,
            send_queue: send_tx,
            receive_queue: recv_rx,
            connected: false,
        }
    }
    
    pub fn connect(&mut self, server_ip: &str) -> bool {
        println!("[Network] Connecting to {}...", server_ip);
        
        match TcpStream::connect(server_ip) {
            Ok(stream) => {
                println!("[Network] Connected successfully!");
                self.stream = Some(stream.try_clone().unwrap());
                self.connected = true;
                
                // Клонируем каналы для потоков
                let send_tx = self.send_queue.clone();
                let mut send_stream = stream.try_clone().unwrap();
                
                // Поток отправки
                thread::spawn(move || {
                    for msg in send_tx {
                        let _ = send_stream.write_all(msg.as_bytes());
                        let _ = send_stream.write_all(b"\n");
                        let _ = send_stream.flush();
                    }
                });
                
                // Поток получения
                let recv_tx = self.receive_queue.clone();
                let mut recv_stream = stream;
                thread::spawn(move || {
                    let mut buffer = String::new();
                    let mut temp = [0u8; 4096];
                    loop {
                        match recv_stream.read(&mut temp) {
                            Ok(n) if n > 0 => {
                                buffer.push_str(&String::from_utf8_lossy(&temp[..n]));
                                while let Some(pos) = buffer.find('\n') {
                                    let msg = buffer[..pos].to_string();
                                    let _ = recv_tx.send(msg);
                                    buffer = buffer[pos + 1..].to_string();
                                }
                            }
                            Ok(_) => continue,
                            Err(_) => break,
                        }
                    }
                });
                
                true
            }
            Err(e) => {
                println!("[Network] Connection failed: {}", e);
                false
            }
        }
    }
    
    pub fn send_position(&mut self, x: f32, y: f32, z: f32) {
        if self.connected {
            let msg = format!("POS|{}|{}|{}", x, y, z);
            let _ = self.send_queue.send(msg);
        }
    }
    
    pub fn send_chat(&mut self, text: &str) {
        if self.connected {
            let msg = format!("CHAT|{}", text);
            let _ = self.send_queue.send(msg);
        }
    }
    
    pub fn receive_messages(&mut self) -> Vec<String> {
        let mut messages = Vec::new();
        while let Ok(msg) = self.receive_queue.try_recv() {
            messages.push(msg);
        }
        messages
    }
    
    pub fn disconnect(&mut self) {
        self.connected = false;
        let _ = self.send_queue.send("DISCONNECT".to_string());
        println!("[Network] Disconnected");
    }
}
