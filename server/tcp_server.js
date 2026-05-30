const net = require('net');

const CONFIG = {
    host: '127.0.0.1',
    port: 8080
};

// Состояние мира (пустой мир без сюжета)
const worldState = {
    players: new Map(),
    time: { hour: 12, minute: 0 },
    weather: 'CLEAR',
    storyDisabled: true,
    spawnPoint: { x: -1038.5, y: -2745.0, z: 20.0 }
};

console.log(`
╔════════════════════════════════════════╗
║     REBORNMP TCP SERVER v1.0           ║
╠════════════════════════════════════════╣
║  Server running on: ${CONFIG.host}:${CONFIG.port}
║  Mode: Free Mode (Story disabled)
║  Spawn: Los Santos International
╚════════════════════════════════════════╝
`);

const server = net.createServer((socket) => {
    const playerId = Date.now();
    console.log(`[+] Player ${playerId} connected from ${socket.remoteAddress}:${socket.remotePort}`);
    
    // Отправляем приветствие и информацию о мире
    const welcomeMessage = JSON.stringify({
        type: 'welcome',
        data: {
            id: playerId,
            message: 'Welcome to RebornMP Free Mode!',
            world: {
                name: 'RebornMP - No Story',
                spawnPoint: worldState.spawnPoint,
                weather: worldState.weather,
                time: worldState.time,
                storyDisabled: true
            }
        }
    });
    socket.write(welcomeMessage + '\n');
    
    // Отправляем информацию о пустом мире
    const worldInfo = JSON.stringify({
        type: 'worldState',
        data: {
            vehicles: [
                { model: 'adder', x: -1038.5, y: -2745.0, z: 20.0 },
                { model: 'zentorno', x: -1040.0, y: -2745.0, z: 20.0 }
            ],
            pickups: [
                { type: 'weapon_pistol', x: -1035.0, y: -2745.0, z: 20.0 }
            ]
        }
    });
    socket.write(worldInfo + '\n');
    
    // Обработка данных от клиента
    socket.on('data', (data) => {
        const message = data.toString().trim();
        console.log(`[${playerId}] Received: ${message}`);
        
        // Обрабатываем команды от клиента
        if (message.startsWith('position:')) {
            // Обновляем позицию игрока
            const pos = message.replace('position:', '');
            console.log(`📍 Player ${playerId} moved to ${pos}`);
            
            // Подтверждаем получение
            socket.write(JSON.stringify({ type: 'sync', status: 'ok' }) + '\n');
        } 
        else if (message === 'requestWorld') {
            // Отправляем полное состояние мира
            socket.write(JSON.stringify({ type: 'world', data: worldState }) + '\n');
        }
        else if (message.startsWith('chat:')) {
            // Обработка чата
            const chatMsg = message.replace('chat:', '');
            console.log(`💬 [Player ${playerId}]: ${chatMsg}`);
            socket.write(JSON.stringify({ type: 'chat', message: 'Message received' }) + '\n');
        }
        else {
            // Просто отвечаем на любые сообщения
            socket.write(JSON.stringify({ 
                type: 'serverResponse', 
                data: 'Connected to RebornMP Free Mode!',
                world: worldState
            }) + '\n');
        }
    });
    
    // Обработка отключения
    socket.on('end', () => {
        console.log(`[-] Player ${playerId} disconnected`);
    });
    
    socket.on('error', (err) => {
        console.log(`[!] Error with player ${playerId}: ${err.message}`);
    });
});

// Запуск сервера
server.listen(CONFIG.port, CONFIG.host, () => {
    console.log(`✅ Server is running on ${CONFIG.host}:${CONFIG.port}`);
    console.log(`🌍 Free mode active - No story missions`);
    console.log(`🎯 Spawn at: ${worldState.spawnPoint.x}, ${worldState.spawnPoint.y}\n`);
});

// Консольные команды
process.stdin.on('data', (data) => {
    const command = data.toString().trim();
    
    switch(command) {
        case 'status':
            console.log(`\n=== Server Status ===`);
            console.log(`Players online: ${worldState.players.size}`);
            console.log(`Weather: ${worldState.weather}`);
            console.log(`Time: ${worldState.time.hour}:${worldState.time.minute}\n`);
            break;
        case 'weather':
            const weathers = ['CLEAR', 'CLOUDS', 'RAIN', 'FOG'];
            worldState.weather = weathers[(weathers.indexOf(worldState.weather) + 1) % weathers.length];
            console.log(`🌤️ Weather changed to ${worldState.weather}`);
            break;
        case 'help':
            console.log(`Commands: status, weather, help`);
            break;
        default:
            console.log(`Unknown command. Type 'help' for commands.`);
    }
});