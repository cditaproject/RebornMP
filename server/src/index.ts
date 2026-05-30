import * as net from 'net';

// ============ КОНФИГУРАЦИЯ ============
const CONFIG = {
    host: '127.0.0.1',
    port: 8080,
    maxPlayers: 64,
    serverName: 'RebornMP Free Mode',
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
    socket: net.Socket;
    connectedAt: number;
}

// ============ СОСТОЯНИЕ МИРА ============
class World {
    public players: Map<number, Player> = new Map();
    public timeHour: number = 12;
    public timeMinute: number = 0;
    public weather: string = 'CLEAR';
    public freeMode: boolean = true;
    public storyDisabled: boolean = true;
    private nextPlayerId: number = 1;

    constructor() {
        console.log('🌍 World initialized: FREE MODE (No Story)');
    }

    addPlayer(socket: net.Socket, name?: string): Player {
        const id = this.nextPlayerId++;
        const player: Player = {
            id,
            name: name || `Player_${id}`,
            position: { x: -1038.5, y: -2745.0, z: 20.0 },
            health: 100,
            armor: 0,
            socket,
            connectedAt: Date.now()
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
            spawnPoint: { x: -1038.5, y: -2745.0, z: 20.0 }
        };
    }
}

// ============ ГЛОБАЛЬНЫЕ ПЕРЕМЕННЫЕ ============
const world = new World();

// ============ ФУНКЦИЯ РАССЫЛКИ ============
function broadcastToAll(packet: object, excludeSocket?: net.Socket): void {
    const message = JSON.stringify(packet) + '\n';
    
    for (const [id, player] of world.players) {
        if (player.socket !== excludeSocket && !player.socket.destroyed) {
            player.socket.write(message);
        }
    }
}

// ============ ОТПРАВКА ПАКЕТА ============
function sendPacket(socket: net.Socket, type: string, data?: any): void {
    const packet = { type, data, timestamp: Date.now() };
    socket.write(JSON.stringify(packet) + '\n');
}

// ============ ЗАПУСК TCP СЕРВЕРА ============
console.log(`
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║              REBORNMP TCP SERVER v1.0                     ║
║                                                           ║
║  🎮 Mode: FREE MODE (No Story)                            ║
║  🌍 World: Los Santos Open World                          ║
║  📡 Port: ${CONFIG.port}                                             ║
║  🔌 Protocol: TCP                                         ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
`);

const server = net.createServer((socket: net.Socket) => {
    const remoteAddr = `${socket.remoteAddress}:${socket.remotePort}`;
    console.log(`[+] Client connected from ${remoteAddr}`);
    
    // Создаём игрока
    const player = world.addPlayer(socket);
    
    // 1. Отправляем приветствие
    sendPacket(socket, 'welcome', {
        message: `Welcome to ${CONFIG.serverName}!`,
        clientId: player.id,
        serverVersion: '1.0.0'
    });
    
    // 2. Отправляем состояние мира (ПУСТОЙ МИР БЕЗ СЮЖЕТА)
    sendPacket(socket, 'worldState', world.getWorldState());
    
    // 3. Отправляем подтверждение свободного режима
    sendPacket(socket, 'gameMode', {
        mode: 'free',
        storyMissions: false,
        wantedLevelDisabled: true,
        spawnProtected: true,
        description: 'Free Mode - No story missions, explore Los Santos!'
    });
    
    // 4. Отправляем спавн позицию
    sendPacket(socket, 'spawn', {
        position: { x: -1038.5, y: -2745.0, z: 20.0 },
        heading: 90
    });
    
    // 5. Уведомляем всех о новом игроке
    broadcastToAll({
        type: 'playerJoin',
        data: {
            id: player.id,
            name: player.name,
            position: player.position
        }
    }, socket);
    
    // 6. Отправляем текущему игроку список всех игроков
    sendPacket(socket, 'playerList', {
        players: world.getPlayersList()
    });
    
    console.log(`   Player #${player.id} (${player.name}) joined free mode`);
    
    // Обработка входящих данных
    let buffer = '';
    
    socket.on('data', (data: Buffer) => {
        buffer += data.toString();
        
        // Разбираем по строкам (каждое сообщение заканчивается \n)
        let lines = buffer.split('\n');
        buffer = lines.pop() || '';
        
        for (const line of lines) {
            if (!line.trim()) continue;
            
            try {
                const packet = JSON.parse(line);
                console.log(`[${player.id}] Received: ${packet.type || 'unknown'}`);
                
                switch (packet.type) {
                    case 'position':
                        if (packet.data?.position) {
                            player.position = packet.data.position;
                            
                            // Рассылаем обновление позиции всем
                            broadcastToAll({
                                type: 'playerPosition',
                                data: {
                                    id: player.id,
                                    position: player.position
                                }
                            }, socket);
                        }
                        break;
                        
                    case 'chat':
                        if (packet.data?.message) {
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
                        sendPacket(socket, 'syncAck', { status: 'ok' });
                        break;
                        
                    case 'requestWorld':
                        sendPacket(socket, 'worldState', world.getWorldState());
                        break;
                        
                    default:
                        sendPacket(socket, 'response', { status: 'ok', echo: packet });
                }
            } catch (e) {
                console.log(`[${player.id}] Raw: ${line.trim()}`);
                sendPacket(socket, 'response', { status: 'ok', message: line.trim() });
            }
        }
    });
    
    // Отключение клиента
    socket.on('close', () => {
        console.log(`[-] Player #${player.id} (${player.name}) disconnected`);
        world.removePlayer(player.id);
        
        broadcastToAll({
            type: 'playerLeave',
            data: { id: player.id }
        });
    });
    
    socket.on('error', (err) => {
        console.log(`[!] Player #${player.id} error: ${