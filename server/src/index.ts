// ============================================
// Freemode-MP Server
// WebSocket + TCP сервер для GTA V мода
// ============================================

import { serve } from "bun";
import * as net from "net";
import { join } from "path";

const PORT = parseInt(process.env.PORT || "8080", 10);
const HOST = process.env.HOST || "0.0.0.0";

// Типы для клиентов
interface Client {
  id: number;
  ws: any;
  name?: string;
  connectedAt: Date;
}

let clients: Map<number, Client> = new Map();
let nextClientId = 1;

console.log("========================================");
console.log("Freemode-MP Server starting...");
console.log("========================================");

// Загрузка Rust ядра (опционально)
let rustCore: any = null;
try {
  const corePath = join(process.cwd(), "core/target/release/server_core.dll");
  const { dlopen, FFIType } = await import("bun:ffi");
  
  rustCore = dlopen(corePath, {
    core_init: {
      args: [],
      returns: FFIType.void,
    },
    core_process_packet: {
      args: [FFIType.ptr, FFIType.u32],
      returns: FFIType.void,
    },
    core_shutdown: {
      args: [],
      returns: FFIType.void,
    },
  });
  
  rustCore.symbols.core_init();
  console.log("✅ Rust server core bound successfully!");
} catch (e) {
  console.log("⚠️  Rust core not found, running in pure TypeScript mode");
}

// Функция рассылки сообщения всем клиентам
function broadcastToAll(data: any, excludeWs?: any) {
  const msg = JSON.stringify(data);
  for (const client of clients.values()) {
    if (client.ws !== excludeWs && client.ws.readyState === 1) { // WebSocket.OPEN = 1
      client.ws.send(msg);
    }
  }
}

// ========== WebSocket сервер (основной) ==========
const wsServer = serve({
  port: PORT,
  hostname: HOST,
  fetch(req, server) {
    const url = new URL(req.url);
    
    // WebSocket upgrade
    if (url.pathname === "/ws") {
      if (server.upgrade(req)) {
        return;
      }
      return new Response("WebSocket upgrade failed", { status: 500 });
    }
    
    // API эндпоинты
    if (url.pathname === "/") {
      return new Response(
        JSON.stringify({
          name: "Freemode-MP Server",
          version: "1.0.0",
          status: "running",
          protocols: ["websocket", "tcp"],
          websocket_port: PORT,
          clients: clients.size,
        }),
        { headers: { "Content-Type": "application/json" } }
      );
    }
    
    if (url.pathname === "/status") {
      return new Response(
        JSON.stringify({
          clients: Array.from(clients.values()).map(c => ({
            id: c.id,
            name: c.name || "Unknown",
            connectedAt: c.connectedAt,
          })),
          total: clients.size,
        }),
        { headers: { "Content-Type": "application/json" } }
      );
    }
    
    return new Response("Not Found", { status: 404 });
  },
  websocket: {
    open(ws) {
      const clientId = nextClientId++;
      clients.set(clientId, {
        id: clientId,
        ws,
        connectedAt: new Date(),
      });
      
      console.log(`✅ Client ${clientId} connected (Total: ${clients.size})`);
      
      // Отправляем приветственное сообщение
      ws.send(JSON.stringify({
        type: "welcome",
        clientId,
        message: "Connected to Freemode-MP Server",
        timestamp: Date.now(),
      }));
      
      // Рассылаем всем о новом игроке
      broadcastToAll({
        type: "chat",
        sender: "Server",
        message: `Player ${clientId} joined the game!`,
        timestamp: Date.now(),
      }, ws);
    },
    
    message(ws, message) {
      const client = Array.from(clients.values()).find(c => c.ws === ws);
      if (!client) return;
      
      console.log(`📨 Message from client ${client.id}:`, message);
      
      // Обработка через Rust core (если есть)
      if (rustCore) {
        const data = typeof message === "string" ? Buffer.from(message) : message;
        rustCore.symbols.core_process_packet(data, data.length);
      }
      
      try {
        const parsed = JSON.parse(message as string);
        
        switch (parsed.type) {
          case "connect":
            client.name = parsed.name || `Player ${client.id}`;
            console.log(`   Client name: ${client.name}`);
            ws.send(JSON.stringify({
              type: "connected",
              message: `Welcome, ${client.name}!`,
            }));
            break;
          
          case "chat":
            console.log(`💬 ${client.name}: ${parsed.message}`);
            // Рассылаем сообщение всем клиентам
            broadcastToAll({
              type: "chat",
              sender: client.name,
              message: parsed.message,
              timestamp: Date.now(),
            });
            break;
          
          case "ping":
            ws.send(JSON.stringify({ type: "pong", timestamp: Date.now() }));
            break;
          
          case "position":
            // Синхронизация позиции игрока
            broadcastToAll({
              type: "position",
              playerId: client.id,
              name: client.name,
              x: parsed.x,
              y: parsed.y,
              z: parsed.z,
              timestamp: Date.now(),
            }, ws);
            break;
          
          default:
            console.log(`   Unknown message type: ${parsed.type}`);
        }
      } catch (e) {
        console.log(`   Raw message (not JSON): ${message}`);
      }
    },
    
    close(ws) {
      const client = Array.from(clients.values()).find(c => c.ws === ws);
      if (client) {
        clients.delete(client.id);
        console.log(`❌ Client ${client.id} (${client.name || "Unknown"}) disconnected (Total: ${clients.size})`);
        
        // Рассылаем всем о выходе игрока
        broadcastToAll({
          type: "chat",
          sender: "Server",
          message: `Player ${client.name || client.id} left the game`,
          timestamp: Date.now(),
        });
      }
    },
    
    drain(ws) {
      console.log("WebSocket backpressure drained");
    },
  },
});

console.log("");
console.log("========================================");
console.log("🚀 Freemode-MP Server Started!");
console.log("========================================");
console.log(`📡 WebSocket:    ws://${HOST}:${PORT}/ws`);
console.log(`📊 Status API:   http://${HOST}:${PORT}/status`);
console.log("========================================");
console.log("");
console.log("Waiting for clients to connect...");

// ========== TCP сервер (для обратной совместимости) ==========
const TCP_PORT = PORT + 1;
const tcpServer = net.createServer((socket) => {
  const addr = `${socket.remoteAddress}:${socket.remotePort}`;
  console.log(`🔌 TCP client connected: ${addr}`);
  
  socket.write("Welcome to Freemode-MP Server (TCP mode)!\n");
  socket.write("Commands: /help, /status, /ping\n");
  
  socket.on("data", (data) => {
    const msg = data.toString().trim();
    console.log(`📨 TCP from ${addr}: ${msg}`);
    
    if (msg === "/ping") {
      socket.write("pong\n");
    } else if (msg === "/status") {
      socket.write(`Server uptime: ${process.uptime()}s\n`);
      socket.write(`Connected WebSocket clients: ${clients.size}\n`);
    } else if (msg === "/help") {
      socket.write("Available commands: /ping, /status, /help\n");
    } else {
      socket.write(`Echo: ${msg}\n`);
    }
  });
  
  socket.on("end", () => {
    console.log(`🔌 TCP client disconnected: ${addr}`);
  });
  
  socket.on("error", (err) => {
    console.error(`TCP socket error: ${err.message}`);
  });
});

tcpServer.listen(TCP_PORT, HOST, () => {
  console.log(`🔌 TCP server:    ${HOST}:${TCP_PORT}`);
  console.log("");
});

// Обработка завершения работы
process.on("SIGINT", () => {
  console.log("\n🛑 Shutting down server...");
  
  if (rustCore) {
    rustCore.symbols.core_shutdown();
  }
  
  // Закрываем все соединения
  for (const client of clients.values()) {
    try {
      client.ws.close();
    } catch (e) {}
  }
  
  tcpServer.close();
  process.exit(0);
});