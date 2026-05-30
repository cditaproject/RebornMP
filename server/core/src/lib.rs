use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Типы пакетов
#[repr(u8)]
enum PacketType {
    Handshake = 0x01,
    Position = 0x02,
    Chat = 0x03,
    Disconnect = 0xFF,
}

struct Player {
    id: u32,
    stream: TcpStream,
    position: (f32, f32, f32),
}

pub struct Server {
    players: Arc<Mutex<HashMap<u32, TcpStream>>>,
    next_id: Arc<Mutex<u32>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            players: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
    
    pub fn start(&self, port: u16) -> std::io::Result<()> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr)?;
        println!("[Server] TCP server listening on {}", addr);
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let players = Arc::clone(&self.players);
                    let next_id = Arc::clone(&self.next_id);
                    
                    thread::spawn(move || {
                        handle_client(stream, players, next_id);
                    });
                }
                Err(e) => {
                    eprintln!("[Server] Connection failed: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

fn handle_client(mut stream: TcpStream, players: Arc<Mutex<HashMap<u32, TcpStream>>>, next_id: Arc<Mutex<u32>>) {
    // Выдаем ID игроку
    let player_id = {
        let mut id = next_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    };
    
    println!("[Server] Player {} connected from {:?}", player_id, stream.peer_addr());
    
    // Отправляем Handshake с ID
    let mut handshake = vec![PacketType::Handshake as u8];
    handshake.extend_from_slice(&player_id.to_le_bytes());
    let _ = stream.write_all(&handshake);
    
    // Сохраняем поток игрока
    {
        let mut players_guard = players.lock().unwrap();
        players_guard.insert(player_id, stream.try_clone().unwrap());
    }
    
    // Обрабатываем сообщения от клиента
    let mut buffer = [0u8; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("[Server] Player {} disconnected", player_id);
                break;
            }
            Ok(n) => {
                if n < 1 {
                    continue;
                }
                
                match buffer[0] {
                    0x01 => { // Handshake
                        println!("[Server] Player {} sent handshake", player_id);
                    }
                    0x02 => { // Position
                        if n >= 13 {
                            let x = f32::from_le_bytes([buffer[5], buffer[6], buffer[7], buffer[8]]);
                            let y = f32::from_le_bytes([buffer[9], buffer[10], buffer[11], buffer[12]]);
                            println!("[Server] Player {} position: ({}, {})", player_id, x, y);
                        }
                    }
                    0x03 => { // Chat
                        if let Ok(msg) = String::from_utf8(buffer[1..n].to_vec()) {
                            println!("[Server] Player {} says: {}", player_id, msg);
                        }
                    }
                    _ => {
                        println!("[Server] Unknown packet from {}: 0x{:02X}", player_id, buffer[0]);
                    }
                }
            }
            Err(e) => {
                eprintln!("[Server] Error reading from player {}: {}", player_id, e);
                break;
            }
        }
    }
    
    // Удаляем игрока
    {
        let mut players_guard = players.lock().unwrap();
        players_guard.remove(&player_id);
    }
}

// FFI экспорт для Bun/Node
#[no_mangle]
pub extern "C" fn start_server(port: u16) {
    let server = Server::new();
    if let Err(e) = server.start(port) {
        eprintln!("[Server] Failed to start: {}", e);
    }
}