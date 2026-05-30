use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub position: Vector3,
    pub health: u32,
    pub armor: u32,
    pub model: String,
    pub ping: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Vehicle {
    pub id: u32,
    pub model: String,
    pub position: Vector3,
    pub health: u32,
    pub color: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct World {
    pub players: HashMap<u32, Player>,
    pub vehicles: HashMap<u32, Vehicle>,
    pub time_hour: u32,
    pub time_minute: u32,
    pub weather: String,
    pub story_mode_enabled: bool,
    pub free_mode: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Packet {
    Connect { client_id: u32, version: String },
    Disconnect { client_id: u32, reason: String },
    PositionUpdate { client_id: u32, position: Vector3 },
    VehicleUpdate { vehicle_id: u32, position: Vector3, health: u32 },
    Sync { entities: Vec<EntityData> },
    Chat { client_id: u32, message: String, target_id: Option<u32> },
    Command { client_id: u32, command: String, args: Vec<String> },
    WorldState { world: World },
    Welcome { client_id: u32, spawn: Vector3, world_name: String },
    PlayerJoin { client_id: u32, player: Player },
    PlayerLeave { client_id: u32 },
    HealthUpdate { client_id: u32, health: u32, armor: u32 },
    WeaponGive { client_id: u32, weapon: String, ammo: u32 },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntityData {
    pub id: u32,
    pub entity_type: String,
    pub position: Vector3,
    pub rotation: Vector3,
    pub data: Vec<u8>,
}

pub struct RebornServer {
    listener: TcpListener,
    clients: Arc<Mutex<HashMap<u32, TcpStream>>>,
    world: Arc<Mutex<World>>,
    next_client_id: Arc<Mutex<u32>>,
    running: Arc<Mutex<bool>>,
}

impl RebornServer {
    pub fn new(addr: &str) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(addr)?;
        println!("==========================================");
        println!("   REBORNMP RUST CORE v1.0");
        println!("==========================================");
        println!("✅ Listening on: {}", addr);
        
        let mut world = World {
            players: HashMap::new(),
            vehicles: HashMap::new(),
            time_hour: 12,
            time_minute: 0,
            weather: "CLEAR".to_string(),
            story_mode_enabled: false,
            free_mode: true,
        };
        
        // Добавляем тестовые машины в свободный мир
        for i in 1..10 {
            world.vehicles.insert(i, Vehicle {
                id: i,
                model: "adder".to_string(),
                position: Vector3 { x: -1035.0 + (i as f32), y: -2745.0, z: 20.0 },
                health: 100,
                color: "BLACK".to_string(),
            });
        }
        
        Ok(RebornServer {
            listener,
            clients: Arc::new(Mutex::new(HashMap::new())),
            world: Arc::new(Mutex::new(world)),
            next_client_id: Arc::new(Mutex::new(1)),
            running: Arc::new(Mutex::new(true)),
        })
    }
    
    pub fn start(&self) {
        let clients = Arc::clone(&self.clients);
        let world = Arc::clone(&self.world);
        let next_id = Arc::clone(&self.next_client_id);
        let running = Arc::clone(&self.running);
        
        println!("🚀 Server core started, waiting for connections...\n");
        
        for stream in self.listener.incoming() {
            if !*running.lock().unwrap() {
                break;
            }
            
            match stream {
                Ok(stream) => {
                    let clients = Arc::clone(&clients);
                    let world = Arc::clone(&world);
                    let next_id = Arc::clone(&next_id);
                    
                    thread::spawn(move || {
                        Self::handle_client(stream, clients, world, next_id);
                    });
                }
                Err(e) => {
                    eprintln!("❌ Connection failed: {}", e);
                }
            }
        }
    }
    
    fn handle_client(
        mut stream: TcpStream,
        clients: Arc<Mutex<HashMap<u32, TcpStream>>>,
        world: Arc<Mutex<World>>,
        next_id: Arc<Mutex<u32>>,
    ) {
        let addr = stream.peer_addr().unwrap();
        let client_id = {
            let mut id = next_id.lock().unwrap();
            let current = *id;
            *id += 1;
            current
        };
        
        println!("[+] Client #{} connected from {}", client_id, addr);
        
        let spawn_point = Vector3 { x: -1038.5, y: -2745.0, z: 20.0 };
        let player = Player {
            id: client_id,
            name: format!("Player_{}", client_id),
            position: spawn_point.clone(),
            health: 100,
            armor: 0,
            model: "mp_m_freemode_01".to_string(),
            ping: 0,
        };
        
        // Добавляем игрока в мир
        {
            let mut world_guard = world.lock().unwrap();
            world_guard.players.insert(client_id, player.clone());
        }
        
        // Отправляем приветствие
        let welcome = Packet::Welcome {
            client_id,
            spawn: spawn_point,
            world_name: "Los Santos - Free Mode".to_string(),
        };
        Self::send_packet(&mut stream, &welcome);
        
        // Отправляем текущее состояние мира
        let world_state = Packet::WorldState {
            world: world.lock().unwrap().clone(),
        };
        Self::send_packet(&mut stream, &world_state);
        
        // Уведомляем всех о новом игроке
        {
            let clients_guard = clients.lock().unwrap();
            let join_packet = Packet::PlayerJoin {
                client_id,
                player: player.clone(),
            };
            for (_, client_stream) in clients_guard.iter() {
                if let Ok(mut s) = client_stream.try_clone() {
                    Self::send_packet(&mut s, &join_packet);
                }
            }
        }
        
        // Добавляем в список клиентов
        {
            let mut clients_guard = clients.lock().unwrap();
            if let Ok(clone) = stream.try_clone() {
                clients_guard.insert(client_id, clone);
            }
        }
        
        // Основной цикл обработки сообщений
        let mut buffer = [0; 8192];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]);
                    for line in data.lines() {
                        if let Ok(packet) = serde_json::from_str::<Packet>(line) {
                            Self::process_packet(client_id, packet, &clients, &world);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[-] Client #{} error: {}", client_id, e);
                    break;
                }
            }
        }
        
        // Очистка при отключении
        {
            let mut clients_guard = clients.lock().unwrap();
            clients_guard.remove(&client_id);
        }
        {
            let mut world_guard = world.lock().unwrap();
            world_guard.players.remove(&client_id);
        }
        
        // Уведомляем всех об отключении
        let leave_packet = Packet::PlayerLeave { client_id };
        let clients_guard = clients.lock().unwrap();
        for (_, client_stream) in clients_guard.iter() {
            if let Ok(mut s) = client_stream.try_clone() {
                Self::send_packet(&mut s, &leave_packet);
            }
        }
        
        println!("[-] Client #{} disconnected", client_id);
    }
    
    fn process_packet(
        client_id: u32,
        packet: Packet,
        clients: &Arc<Mutex<HashMap<u32, TcpStream>>>,
        world: &Arc<Mutex<World>>,
    ) {
        match packet {
            Packet::PositionUpdate { client_id: id, position } if id == client_id => {
                let mut world_guard = world.lock().unwrap();
                if let Some(player) = world_guard.players.get_mut(&client_id) {
                    player.position = position.clone();
                    
                    // Рассылаем обновление всем игрокам
                    let update = Packet::PositionUpdate {
                        client_id,
                        position,
                    };
                    let clients_guard = clients.lock().unwrap();
                    for (_, stream) in clients_guard.iter() {
                        if let Ok(mut s) = stream.try_clone() {
                            Self::send_packet(&mut s, &update);
                        }
                    }
                }
            }
            
            Packet::Chat { client_id: id, message, target_id } if id == client_id => {
                println!("💬 [Player {}]: {}", client_id, message);
                
                let chat_packet = Packet::Chat {
                    client_id,
                    message,
                    target_id: None,
                };
                
                let clients_guard = clients.lock().unwrap();
                if let Some(target) = target_id {
                    if let Some(stream) = clients_guard.get(&target) {
                        if let Ok(mut s) = stream.try_clone() {
                            Self::send_packet(&mut s, &chat_packet);
                        }
                    }
                } else {
                    for (_, stream) in clients_guard.iter() {
                        if let Ok(mut s) = stream.try_clone() {
                            Self::send_packet(&mut s, &chat_packet);
                        }
                    }
                }
            }
            
            Packet::Command { client_id: id, command, args } if id == client_id => {
                Self::execute_command(client_id, &command, &args, clients, world);
            }
            
            _ => {}
        }
    }
    
    fn execute_command(
        client_id: u32,
        command: &str,
        args: &[String],
        clients: &Arc<Mutex<HashMap<u32, TcpStream>>>,
        world: &Arc<Mutex<World>>,
    ) {
        match command {
            "heal" => {
                let mut world_guard = world.lock().unwrap();
                if let Some(player) = world_guard.players.get_mut(&client_id) {
                    player.health = 100;
                    println!("💊 Player {} healed", client_id);
                }
            }
            "vehicle" => {
                if let Some(model) = args.first() {
                    println!("🚗 Spawning vehicle {} for player {}", model, client_id);
                }
            }
            _ => {
                println!("📝 Unknown command from {}: {}", client_id, command);
            }
        }
    }
    
    fn send_packet(stream: &mut TcpStream, packet: &Packet) {
        if let Ok(json) = serde_json::to_string(packet) {
            let _ = stream.write(json.as_bytes());
            let _ = stream.write(b"\n");
        }
    }
    
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
        println!("🛑 Server core stopping...");
    }
}

// FFI экспорты для Node.js/Bun
#[no_mangle]
pub extern "C" fn server_create(addr_ptr: *const u8, addr_len: usize) -> *mut RebornServer {
    let addr = unsafe {
        String::from_utf8_unchecked(std::slice::from_raw_parts(addr_ptr, addr_len).to_vec())
    };
    
    match RebornServer::new(&addr) {
        Ok(server) => Box::into_raw(Box::new(server)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn server_start(server_ptr: *mut RebornServer) {
    if !server_ptr.is_null() {
        unsafe {
            (*server_ptr).start();
        }
    }
}

#[no_mangle]
pub extern "C" fn server_stop(server_ptr: *mut RebornServer) {
    if !server_ptr.is_null() {
        unsafe {
            (*server_ptr).stop();
        }
    }
}

#[no_mangle]
pub extern "C" fn server_destroy(server_ptr: *mut RebornServer) {
    if !server_ptr.is_null() {
        unsafe {
            drop(Box::from_raw(server_ptr));
        }
    }
}