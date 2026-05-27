use anyhow::Result;
use log::info;

pub fn init() -> Result<()> {
    info!("🪝 Hooks system initialized");
    Ok(())
}

pub fn on_frame() -> Result<()> {
    // Здесь будут хуки
    Ok(())
}

pub fn cleanup() -> Result<()> {
    info!("🪝 Hooks cleaned up");
    Ok(())
}