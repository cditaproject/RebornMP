use anyhow::Result;
use log::info;

pub fn init() -> Result<()> {
    info!("🎨 UI module initialized");
    Ok(())
}

pub fn render() -> Result<()> {
    // Здесь будет рендер UI
    Ok(())
}

pub fn shutdown() -> Result<()> {
    info!("🎨 UI shutdown complete");
    Ok(())
}