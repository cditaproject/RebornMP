use std::net::TcpStream;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use crate::memory;

pub fn start_client() {
    thread::spawn(|| {
        loop {
            match TcpStream::connect("127.0.0.1:8080") {
                Ok(mut stream) => {
                    println!("✅ Connected to server!");
                    let _ = stream.write(b"{\"type\":\"hello\",\"data\":{\"client\":\"RebornMP\"}}\n");
                    
                    let mut buffer = [0; 4096];
                    loop {
                        match stream.read(&mut buffer) {
                            Ok(0) => break,
                            Ok(n) => {
                                let msg = String::from_utf8_lossy(&buffer[..n]);
                                println!("📩 Received: {}", msg);
                                
                                if msg.contains("\"freeMode\"") || msg.contains("\"disableStory\"") {
                                    memory::disable_story();
                                }
                                if msg.contains("\"spawn\"") {
                                    memory::teleport_to_spawn();
                                }
                            }
                            Err(e) => break,
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Connection failed: {}", e);
                    thread::sleep(Duration::from_secs(5));
                }
            }
        }
    });
}