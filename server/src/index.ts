import { dlopen, suffix } from 'bun:ffi';
import { join } from 'path';
import { existsSync } from 'fs';

console.log('EntityMP Server starting...');

// Правильный путь к release сборке
const libPath = join('C:/GitHub/Test/RebornMP/target/release', `server_core.${suffix}`);

console.log(`Loading Rust server core from: ${libPath}`);

// Проверяем существование файла
if (!existsSync(libPath)) {
    console.error(`❌ Library not found at: ${libPath}`);
    console.error('\nPlease build first with:');
    console.error('  cd C:\\GitHub\\Test\\RebornMP');
    console.error('  cargo build --release --package server_core');
    process.exit(1);
}

try {
    const serverLib = dlopen(libPath, {
        start_server: {
            args: ['u16'],
            returns: 'void'
        }
    });
    
    console.log('✅ Rust server core bound successfully!');
    console.log('[TS] Starting TCP server on port 7777...');
    serverLib.symbols.start_server(7777);
    
    console.log('[TS] Server is running on 127.0.0.1:7777');
    console.log('[TS] Press Ctrl+C to stop.');
    
    // Держим процесс активным
    setInterval(() => {}, 1000);
    
} catch (error) {
    console.error('❌ Failed to load Rust library:', error);
    console.error('\nPossible issues:');
    console.error('1. Missing Visual C++ Redistributable');
    console.error('2. Library architecture mismatch (x64 vs x86)');
    console.error('3. Missing dependencies');
    process.exit(1);
}