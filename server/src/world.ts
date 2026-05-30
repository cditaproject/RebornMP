export class WorldManager {
    private time: { hour: number, minute: number } = { hour: 12, minute: 0 };
    private weather: string = 'CLEAR';
    private entities: Map<number, any> = new Map();
    private storyDisabled: boolean = true;
    
    // Очищаем сюжетные миссии
    clearStory() {
        this.storyDisabled = true;
        console.log('🗺️ Story mode disabled - Free mode active');
        
        // Отключаем сюжетные триггеры
        this.disableStoryTriggers();
    }
    
    private disableStoryTriggers() {
        // Список координат сюжетных триггеров для отключения
        const storyTriggers = [
            { x: -215.7, y: -2785.7, z: 6.0 },  // Franklin's house
            { x: 128.5, y: -1298.5, z: 29.3 },  // Michael's house
            { x: -45.5, y: -1120.3, z: 26.5 }   // Trevor's trailer
        ];
        
        // В реальном клиенте эти координаты будут отправлены для отключения
        console.log(`📍 Disabled ${storyTriggers.length} story triggers`);
    }
    
    setTime(hour: number, minute: number = 0) {
        this.time = { hour, minute };
        console.log(`⏰ Time set to ${hour}:${minute.toString().padStart(2, '0')}`);
    }
    
    setWeather(weather: string) {
        const validWeather = ['CLEAR', 'CLOUDS', 'SMOG', 'FOG', 'RAIN'];
        if (validWeather.includes(weather)) {
            this.weather = weather;
            console.log(`🌤️ Weather changed to ${weather}`);
        }
    }
    
    getState() {
        return {
            time: this.time,
            weather: this.weather,
            storyDisabled: this.storyDisabled,
            entities: Array.from(this.entities.values())
        };
    }
    
    getFullState() {
        return {
            ...this.getState(),
            name: 'RebornMP Free World',
            vehicles: this.getVehicles(),
            pickups: this.getPickups()
        };
    }
    
    syncEntity(entity: any) {
        this.entities.set(entity.id, entity);
    }
    
    private getVehicles() {
        // Базовые машины для свободного режима
        return [
            { model: 'adder', position: { x: -1038.5, y: -2745.0, z: 20.0 } },
            { model: 'zentorno', position: { x: -1040.0, y: -2745.0, z: 20.0 } }
        ];
    }
    
    private getPickups() {
        // Оружие и аптечки для свободного режима
        return [
            { type: 'weapon_pistol', position: { x: -1035.0, y: -2745.0, z: 20.0 } }
        ];
    }
    
    getWeather() {
        return this.weather;
    }
}