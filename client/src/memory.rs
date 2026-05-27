use anyhow::Result;
use log::info;
use std::sync::atomic::{AtomicBool, Ordering};

static GAME_ALIVE: AtomicBool = AtomicBool::new(true);

pub fn init() -> Result<()> {
    info!("📦 Memory module initialized");
    Ok(())
}

pub fn is_game_alive() -> bool {
    GAME_ALIVE.load(Ordering::Relaxed)
}

pub fn mark_game_dead() {
    GAME_ALIVE.store(false, Ordering::Relaxed);
}