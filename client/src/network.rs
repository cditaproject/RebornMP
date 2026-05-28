// client/src/network.rs
// RebornMP Network Client - Final Working Version

use std::net::TcpStream;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};

pub struct NetworkClient {
    stream: Option<TcpStream>,
    message_receiver: Receiver<String>,
    message_sender: Sender<String>,
    connected: bool,
}

impl NetworkClient {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        NetworkClient {
            stream: None,
            message_receiver: rx,
            message_sender: tx,
            connected: false,
        }
    }
    
    pub fn connect(&mut self, server_ip: &str) -> bool {
        match TcpStream::connect(server_ip) {
            Ok(stream) => {
                println!("[Network] Connected to {}", server_ip);
                
                // Клонируем stream для потока
                let mut read_stream = stream.try_clone().unwrap();
                let tx = self.message_sender.clone();
                
                // Поток чтения
                thread::spawn(move || {
                    let mut buf = [0u8; 4096];
                    loop {
                        match read_stream.read(&mut buf) {
                            Ok(n) if n > 0 => {
                                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                                let _ = tx.send(msg);
                            }
                            Ok(_) => thread::sleep(Duration::from_millis(10)),
                            Err(_) => break,
                        }
                    }
                });
                
                // Сохраняем оригинальный stream для отправки
                self.stream = Some(stream);
                self.connected = true;
                true
            }
            Err(e) => {
                println!("[Network] Connect error: {}", e);
                false
            }
        }
    }
    
    pub fn send_position(&mut self, x: f32, y: f32, z: f32) {
        self.send(&format!("POS|{}|{}|{}", x, y, z));
    }
    
    pub fn send_chat(&mut self, text: &str) {
        self.send(&format!("CHAT|{}", text));
    }
    
    pub fn send_command(&mut self, cmd: &str, args: &str) {
        self.send(&format!("CMD|{}|{}", cmd, args));
    }
    
    fn send(&mut self, msg: &str) {
        if let Some(stream) = &mut self.stream {
            let _ = stream.write_all(msg.as_bytes());
            let _ = stream.write_all(b"\n");
            let _ = stream.flush();
        }
    }
    
    pub fn receive_messages(&mut self) -> Vec<String> {
        let mut msgs = Vec::new();
        while let Ok(msg) = self.message_receiver.try_recv() {
            msgs.push(msg);
        }
        msgs
    }
    
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.stream = None;
        println!("[Network] Disconnected");
    }
    
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}
