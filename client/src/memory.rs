// client/src/memory.rs
// RebornMP Memory Manager - GTA V Offsets (to be updated)

const OFFSET_POS_X: usize = 0x40;
const OFFSET_POS_Y: usize = 0x44;
const OFFSET_POS_Z: usize = 0x48;

fn get_player_address() -> Option<usize> {
    Some(0x0) // TODO: Найти реальный адрес
}

pub fn get_player_position() -> Option<(f32, f32, f32)> {
    if let Some(addr) = get_player_address() {
        if addr != 0 {
            unsafe {
                let pos_x = *(addr as *const f32);
                let pos_y = *((addr + OFFSET_POS_Y) as *const f32);
                let pos_z = *((addr + OFFSET_POS_Z) as *const f32);
                return Some((pos_x, pos_y, pos_z));
            }
        }
    }
    Some((0.0, 0.0, 0.0))
}

pub fn set_player_position(x: f32, y: f32, z: f32) -> bool {
    if let Some(addr) = get_player_address() {
        if addr != 0 {
            unsafe {
                *(addr as *mut f32) = x;
                *((addr + OFFSET_POS_Y) as *mut f32) = y;
                *((addr + OFFSET_POS_Z) as *mut f32) = z;
                return true;
            }
        }
    }
    false
}
