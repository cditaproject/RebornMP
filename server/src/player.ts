interface Player {
    id: number;
    name: string;
    position: { x: number, y: number, z: number };
    health: number;
    armor: number;
    model: string;
    ping: number;
}

export class PlayerManager {
    private players: Map<number, Player> = new Map();
    private nextId: number = 1;
    
    generateId(): number {
        return this.nextId++;
    }
    
    create(id: number, data: Partial<Player>) {
        const player: Player = {
            id,
            name: `Player_${id}`,
            position: { x: 0, y: 0, z: 0 },
            health: 100,
            armor: 0,
            model: 'mp_m_freemode_01',
            ping: 0,
            ...data
        };
        
        this.players.set(id, player);
        console.log(`👤 Player ${player.name} (${id}) created at spawn point`);
        return player;
    }
    
    remove(id: number) {
        this.players.delete(id);
    }
    
    updatePosition(id: number, position: { x: number, y: number, z: number }) {
        const player = this.players.get(id);
        if (player) {
            player.position = position;
        }
    }
    
    get(id: number): Player | undefined {
        return this.players.get(id);
    }
    
    getName(id: number): string {
        return this.players.get(id)?.name || `Unknown_${id}`;
    }
    
    count(): number {
        return this.players.size;
    }
    
    list(): Player[] {
        return Array.from(this.players.values());
    }
    
    kick(id: number) {
        this.players.delete(id);
        // Здесь нужно будет отправить сообщение клиенту о кике
    }
}