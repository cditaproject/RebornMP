import { serve } from "bun";

const server = serve({
    port: 3000,
    fetch(req, server) {
        const url = new URL(req.url);
        
        if (url.pathname === "/ws") {
            const success = server.upgrade(req);
            return success ? undefined : new Response("WebSocket failed", { status: 400 });
        }
        
        return new Response("RebornMP Server", { status: 200 });
    },
    
    websocket: {
        open(ws) {
            console.log(`✅ Client connected`);
            ws.send(JSON.stringify({ type: "init", name: "Server", money: 1000 }));
        },
        
        message(ws, data) {
            const msg = data.toString();
            console.log(`📨 ${msg}`);
            
            if (msg.startsWith("CHAT|")) {
                const text = msg.substring(5);
                ws.send(JSON.stringify({ type: "chat", message: `Server: ${text}` }));
            }
        },
        
        close(ws) {
            console.log(`❌ Client disconnected`);
        }
    }
});

console.log(`🚀 RebornMP Server on port ${server.port}`);