import * as net from 'net';

const PORT = 8080;
const SPAWN_POINT = { x: -1038.5, y: -2745.0, z: 20.0 };

const server = net.createServer((socket) => {
    console.log(`[Server] Client connected from ${socket.remoteAddress}`);
    
    // 1. Отправляем режим свободной игры
    socket.write(JSON.stringify({
        type: "freeMode",
        data: true
    }) + '\n');
    
    // 2. Отправляем команду телепортации на спавн
    setTimeout(() => {
        socket.write(JSON.stringify({
            type: "spawn",
            data: SPAWN_POINT
        }) + '\n');
        console.log(`[Server] Sent spawn command to client`);
    }, 1000);
    
    // 3. Отключаем сюжет
    socket.write(JSON.stringify({
        type: "disableStory",
        data: true
    }) + '\n');
    
    // Обработка сообщений от клиента
    socket.on('data', (data) => {
        const msg = data.toString();
        console.log(`[Server] Received: ${msg}`);
        
        // Отправляем подтверждение
        socket.write(JSON.stringify({
            type: "ack",
            data: "received"
        }) + '\n');
    });
    
    socket.on('error', (err) => {
        console.log(`[Server] Error: ${err.message}`);
    });
    
    socket.on('close', () => {
        console.log(`[Server] Client disconnected`);
    });
});

server.listen(PORT, '127.0.0.1', () => {
    console.log(`========================================`);
    console.log(`   RebornMP Server running on port ${PORT}`);
    console.log(`   Free Mode Active`);
    console.log(`   Spawn point: ${SPAWN_POINT.x}, ${SPAWN_POINT.y}`);
    console.log(`========================================`);
});