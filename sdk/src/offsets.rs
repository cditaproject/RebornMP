// sdk/src/offsets.rs
// Оффсеты для GTA V Legacy (v1.0.3258.0)

pub mod offsets {
    // ========== ГЛОБАЛЬНЫЕ УКАЗАТЕЛИ ==========
    pub const WORLD_PTR: usize = 0x267B0F8;          
    pub const LOCAL_PED: usize = 0x8;                 
    
    // ========== ЗДОРОВЬЕ И БРОНЯ ==========
    pub const HEALTH: usize = 0x280;                  
    pub const MAX_HEALTH: usize = 0x284;              
    pub const ARMOR: usize = 0x288;                  
    
    // ========== ПОЗИЦИЯ ==========
    pub const POSITION_X: usize = 0x40;               
    pub const POSITION_Y: usize = 0x44;               
    pub const POSITION_Z: usize = 0x48;               
    pub const ROTATION: usize = 0x70;                 
    
    // ========== ДЕНЬГИ И РОЗЫСК ==========
    pub const MONEY: usize = 0x11F8;                  
    pub const WANTED: usize = 0x10A8;                 
    
    // ========== ТРАНСПОРТ ==========
    pub const VEHICLE: usize = 0x18D8;                
    pub const VEHICLE_SPEED: usize = 0x48;            
    
    // ========== ОРУЖИЕ ==========
    pub const WEAPON: usize = 0x1078;                 
    pub const AMMO: usize = 0x1080;                  
}

pub fn get_ped_address() -> usize {
    unsafe {
        let world_ptr = *(offsets::WORLD_PTR as *const usize);
        if world_ptr != 0 {
            return *(world_ptr + offsets::LOCAL_PED) as *const usize;
        }
        0
    }
}
