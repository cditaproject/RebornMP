import { serve } from 'bun';
import type { ServerWebSocket } from 'bun';

// ============ КОНФИГУРАЦИЯ ============
const CONFIG = {
    host: '127.0.0.1',
    port: 8080,
    maxPlayers: 64,
    serverName: 'RebornMP Free Mode',
    serverVersion: '1.0.0'
};

// ============ ТИПЫ ============
interface Vector3 {
    x: number;
    y: number;
    z: number;
}

interface Player {
    id: number;
    name: string;
    position: Vector3;
    health: number;
    armor: number;
    model: string;
    ping: number;
    connectedAt: number;
    ws: ServerWebSocket<any>;
}

interface Vehicle {
    id: number;
    model: string;
    position: Vector3;
    health: number;
    owner: number | null;
}

// ============ СОСТОЯНИЕ МИРА ============
class World {
    public players: Map<number, Player> = new Map();
    public vehicles: Map<number, Vehicle> = new Map();
    public timeHour: number = 12;
    public timeMinute: number = 0;
    public weather: string = 'CLEAR';
    public freeMode: boolean = true;
    public storyDisabled: boolean = true;
    private nextPlayerId: number = 1;
    private nextVehicleId: number = 1;

    constructor() {
        this.initVehicles();
        console.log('🌍 World initialized: FREE MODE (No Story)');
    }

    private initVehicles(): void {
        const spawnPoints = [
            { x: -1038.5, y: -2745.0, z: 20.0 },
            { x: -1040.0, y: -2745.0, z: 20.0 },
            { x: -1041.5, y: -2745.0, z: 20.0 },
            { x: -1043.0, y: -2745.0, z: 20.0 },
            { x: -1044.5, y: -2745.0, z: 20.0 },
        ];
        
        const models = ['adder', 'zentorno', 't20', 'osiris', 'turismor'];
        
        for (let i = 0; i < spawnPoints.length; i++) {
            this.vehicles.set(this.nextVehicleId, {
                id: this.nextVehicleId,
                model: models[i % models.length],
                position: spawnPoints[i % spawnPoints.length],
                health: 100,
                owner: null
            });
            this.nextVehicleId++;
        }
        
        console.log(`🚗 Spawned ${this.vehicles.size} vehicles at LS Airport`);
    }

    addPlayer(ws: ServerWebSocket<any>, name?: string): Player {
        const id = this.nextPlayerId++;
        const player: Player = {
            id,
            name: name || `Player_${id}`,
            position: { x: -1038.5, y: -2745.0, z: 20.0 },
            health: 100,
            armor: 0,
            model: 'mp_m_freemode_01',
            ping: 0,
            connectedAt: Date.now(),
            ws
        };
        this.players.set(id, player);
        return player;
    }

    removePlayer(playerId: number): void {
        this.players.delete(playerId);
    }

    getPlayer(playerId: number): Player | undefined {
        return this.players.get(playerId);
    }

    updatePlayerPosition(playerId: number, position: Vector3): void {
        const player = this.players.get(playerId);
        if (player) {
            player.position = position;
        }
    }

    getPlayersList(): object[] {
        return Array.from(this.players.values()).map(p => ({
            id: p.id,
            name: p.name,
            position: p.position,
            health: p.health
        }));
    }

    getWorldState(): object {
        return {
            freeMode: this.freeMode,
            storyDisabled: this.storyDisabled,
            time: { hour: this.timeHour, minute: this.timeMinute },
            weather: this.weather,
            playersCount: this.players.size,
            maxPlayers: CONFIG.maxPlayers,
            spawnPoint: { x: -1038.5, y: -2745.0, z: 20.0 },
            vehicles: Array.from(this.vehicles.values()).map(v => ({
                id: v.id,
                model: v.model,
                position: v.position
            }))
        };
    }
}

// ============ ГЛОБАЛЬНЫЕ ПЕРЕМЕННЫЕ ============
const world = new World();
let clients: Map<number, ServerWebSocket<any>> = new Map();

// ============ ФУНКЦИЯ РАССЫЛКИ ============
function broadcastToAll(packet: object, excludeWs?: ServerWebSocket<any>): void {
    const message = JSON.stringify(packet);
    
    for (const [id, ws] of clients) {
        if (excludeWs !== ws && ws.readyState === 1) { // 1 = OPEN
            ws.send(message);
        }
    }
}

// ============ ЗАПУСК СЕРВЕРА ============
console.log(`
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║              REBORNMP SERVER v1.0                         ║
║                                                           ║
║  🎮 Mode: FREE MODE (No Story)                            ║
║  🌍 World: Los Santos Open World                          ║
║  📡 Port: ${CONFIG.port}                                             ║
║  🚗 Vehicles pre-spawned: ${world.vehicles.size}                     ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
`);

const server = serve({
    hostname: CONFIG.host,
    port: CONFIG.port,
    fetch(req, server) {
        const url = new URL(req.url);
        
        // Обновляем WebSocket соединение
        if (url.pathname === '/ws' || req.headers.get('upgrade') === 'websocket') {
            const success = server.upgrade(req);
            if (success) return;
        }
        
        return new Response('RebornMP Server - Free Mode', { 
            status: 200,
            headers: { 'Content-Type': 'text/plain' }
        });
    },
    websocket: {
        open(ws: ServerWebSocket<any>) {
            console.log(`[+] Client connected from ${ws.remoteAddress}`);
            
            // Создаём игрока
            const player = world.addPlayer(ws);
            ws.data = { playerId: player.id };
            clients.set(player.id, ws);
            
            // 1. Отправляем приветствие
            ws.send(JSON.stringify({
                type: 'welcome',
                data: {
                    message: `Welcome to ${CONFIG.serverName}!`,
                    clientId: player.id,
                    serverVersion: CONFIG.serverVersion
                }
            }));
            
            // 2. Отправляем состояние мира (ПУСТОЙ МИР БЕЗ СЮЖЕТА)
            ws.send(JSON.stringify({
                type: 'worldState',
                data: world.getWorldState()
            }));
            
            // 3. Отправляем подтверждение свободного режима
            ws.send(JSON.stringify({
                type: 'gameMode',
                data: {
                    mode: 'free',
                    storyMissions: false,
                    copsDisabled: false,
                    wantedLevelDisabled: true,
                    spawnProtected: true,
                    description: 'Free Mode - No story missions, explore the world!'
                }
            }));
            
            // 4. Отправляем спавн позицию
            ws.send(JSON.stringify({
                type: 'spawn',
                data: {
                    position: { x: -1038.5, y: -2745.0, z: 20.0 },
                    heading: 90,
                    dimension: 0
                }
            }));
            
            // 5. Уведомляем всех остальных игроков о новом игроке
            broadcastToAll({
                type: 'playerJoin',
                data: {
                    id: player.id,
                    name: player.name,
                    position: player.position
                }
            }, ws);
            
            // 6. Отправляем текущему игроку список всех игроков
            ws.send(JSON.stringify({
                type: 'playerList',
                data: {
                    players: world.getPlayersList()
                }
            }));
            
            console.log(`   Player #${player.id} (${player.name}) joined the free mode`);
        },
        
        message(ws: ServerWebSocket<any>, message: string | Buffer) {
            const playerId = ws.data?.playerId;
            const msgStr = typeof message === 'string' ? message : message.toString();
            
            try {
                const packet = JSON.parse(msgStr);
                console.log(`[${playerId}] Received: ${packet.type}`);
                
                switch (packet.type) {
                    case 'position':
                        if (packet.data && packet.data.position && playerId) {
                            world.updatePlayerPosition(playerId, packet.data.position);
                            
                            broadcastToAll({
                                type: 'playerPosition',
                                data: {
                                    id: playerId,
                                    position: packet.data.position
                                }
                            }, ws);
                        }
                        break;
                        
                    case 'chat':
                        const player = world.getPlayer(playerId);
                        if (player && packet.data?.message) {
                            console.log(`💬 [${player.name}]: ${packet.data.message}`);
                            
                            broadcastToAll({
                                type: 'chat',
                                data: {
                                    playerId: player.id,
                                    playerName: player.name,
                                    message: packet.data.message
                                }
                            });
                        }
                        break;
                        
                    case 'sync':
                        ws.send(JSON.stringify({
                            type: 'syncAck',
                            data: { status: 'ok', timestamp: Date.now() }
                        }));
                        break;
                        
                    case 'requestWorld':
                        ws.send(JSON.stringify({
                            type: 'worldState',
                            data: world.getWorldState()
                        }));
                        break;
                        
                    case 'requestVehicles':
                        ws.send(JSON.stringify({
                            type: 'vehicleList',
                            data: {
                                vehicles: Array.from(world.vehicles.values()).map(v => ({
                                    id: v.id,
                                    model: v.model,
                                    position: v.position
                                }))
                            }
                        }));
                        break;
                        
                    default:
                        ws.send(JSON.stringify({
                            type: 'response',
                            data: { status: 'ok', echo: packet }
                        }));
                }
            } catch (e) {
                console.log(`[${playerId}] Raw: ${msgStr}`);
                ws.send(JSON.stringify({
                    type: 'response',
                    data: { status: 'ok', message: msgStr }
                }));
            }
        },
        
        close(ws: ServerWebSocket<any>) {
            const playerId = ws.data?.playerId;
            if (playerId) {
                const player = world.getPlayer(playerId);
                console.log(`[-] Player #${playerId} (${player?.name}) disconnected`);
                world.removePlayer(playerId);
                clients.delete(playerId);
                
                broadcastToAll({
                    type: 'playerLeave',
                    data: { id: playerId }
                });
            }
        }
    }
});

console.log(`✅ Server running on ${CONFIG.host}:${CONFIG.port}`);
console.log(`🌍 Free Mode: Story missions DISABLED`);
console.log(`🎯 Spawn point: Los Santos International Airport (-1038.5, -2745.0, 20.0)`);
console.log(`🚗 Available vehicles: ${world.vehicles.size}`);
console.log(`\n💡 Console commands: status, players, weather, time, help\n`);

// ============ КОНСОЛЬНЫЕ КОМАНДЫ ============
const handleCommand = (input: string) => {
    const parts = input.trim().toLowerCase().split(' ');
    const cmd = parts[0];
    
    switch (cmd) {
        case 'status':
            console.log(`\n=== Server Status ===`);
            console.log(`Players online: ${world.players.size}/${CONFIG.maxPlayers}`);
            console.log(`Mode: FREE MODE (No Story)`);
            console.log(`Weather: ${world.weather}`);
            console.log(`Time: ${world.timeHour}:${world.timeMinute.toString().padStart(2, '0')}`);
            console.log(`Vehicles: ${world.vehicles.size}\n`);
            break;
            
        case 'players':
            console.log(`\n=== Online Players ===`);
            world.players.forEach(p => {
                console.log(`  #${p.id} ${p.name} - health: ${p.health}`);
            });
            console.log(``);
            break;
            
        case 'weather':
            const weathers = ['CLEAR', 'CLOUDS', 'RAIN', 'FOG', 'THUNDER', 'CLEARING'];
            let newWeather = weathers[(weathers.indexOf(world.weather) + 1) % weathers.length];
            world.weather = newWeather;
            console.log(`🌤️ Weather changed to: ${world.weather}`);
            
            broadcastToAll({
                type: 'weather',
                data: { weather: world.weather }
            });
            break;
            
        case 'time':
            if (parts.length > 1) {
                const timeParts = parts[1].split(':');
                const hour = parseInt(timeParts[0]);
                const minute = timeParts[1] ? parseInt(timeParts[1]) : 0;
                if (!isNaN(hour) && hour >= 0 && hour <= 23) {
                    world.timeHour = hour;
                    world.timeMinute = minute;
                    console.log(`⏰ Time set to: ${world.timeHour}:${world.timeMinute.toString().padStart(2, '0')}`);
                    
                    broadcastToAll({
                        type: 'time',
                        data: { hour: world.timeHour, minute: world.timeMinute }
                    });
                }
            }
            break;
            
        case 'help':
            console.log(`
Available commands:
  status   - Show server status
  players  - List connected players
  weather  - Change weather
  time HH:MM - Set time
  help     - Show this help
`);
            break;
            
        default:
            if (cmd && cmd !== '') {
                console.log(`Unknown command: ${cmd}. Type 'help' for commands.`);
            }
    }
};

// Обработка ввода в консоли
process.stdin.on('data', (data) => {
    handleCommand(data.toString());
});