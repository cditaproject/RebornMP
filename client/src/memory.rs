// client/src/memory.rs
// RebornMP Memory Manager

const OFFSET_POS_X: usize = 0x40;
const OFFSET_POS_Y: usize = 0x44;
const OFFSET_POS_Z: usize = 0x48;
const OFFSET_HEALTH: usize = 0x280;
const OFFSET_ARMOR: usize = 0x284;
const OFFSET_WANTED: usize = 0x2A0;

fn get_player_address() -> Option<usize> {
    Some(0x0)
}

pub fn get_player_position() -> Option<(f32, f32, f32)> {
    if let Some(player_addr) = get_player_address() {
        if player_addr != 0 {
            unsafe {
                let pos_x = *(player_addr as *const f32);
                let pos_y = *((player_addr + OFFSET_POS_Y) as *const f32);
                let pos_z = *((player_addr + OFFSET_POS_Z) as *const f32);
                return Some((pos_x, pos_y, pos_z));
            }
        }
    }
    Some((0.0, 0.0, 0.0))
}

pub fn get_player_health() -> u32 {
    if let Some(player_addr) = get_player_address() {
        if player_addr != 0 {
            unsafe {
                return *((player_addr + OFFSET_HEALTH) as *const u32);
            }
        }
    }
    100
}

pub fn get_player_armor() -> u32 {
    if let Some(player_addr) = get_player_address() {
        if player_addr != 0 {
            unsafe {
                return *((player_addr + OFFSET_ARMOR) as *const u32);
            }
        }
    }
    0
}

pub fn get_wanted_level() -> u32 {
    if let Some(player_addr) = get_player_address() {
        if player_addr != 0 {
            unsafe {
                return *((player_addr + OFFSET_WANTED) as *const u32);
            }
        }
    }
    0
}

pub fn set_player_position(x: f32, y: f32, z: f32) -> bool {
    if let Some(player_addr) = get_player_address() {
        if player_addr != 0 {
            unsafe {
                *(player_addr as *mut f32) = x;
                *((player_addr + OFFSET_POS_Y) as *mut f32) = y;
                *((player_addr + OFFSET_POS_Z) as *mut f32) = z;
                return true;
            }
        }
    }
    false
}

pub fn set_player_health(health: u32) -> bool {
    if let Some(player_addr) = get_player_address() {
        if player_addr != 0 {
            unsafe {
                *((player_addr + OFFSET_HEALTH) as *mut u32) = health;
                return true;
            }
        }
    }
    false
}

pub fn set_player_armor(armor: u32) -> bool {
    if let Some(player_addr) = get_player_address() {
        if player_addr != 0 {
            unsafe {
                *((player_addr + OFFSET_ARMOR) as *mut u32) = armor;
                return true;
            }
        }
    }
    false
}
