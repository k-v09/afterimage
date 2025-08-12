mod cpu;
mod memory;
mod ppu;
mod gba;

use gba::Gba;

fn main() {
    let mut gba = Gba::new();
    
    let rom_paths = ["pokemon_emerald.gba", "test.gba", "game.gba"];
    let mut rom_loaded = false;
    
    for path in &rom_paths {
        match gba.load_rom(path) {
            Ok(_) => {
                println!("ROM loaded successfully: {}", path);
                rom_loaded = true;
                break;
            }
            Err(_) => continue,
        }
    }
    
    if !rom_loaded {
        println!("No ROM file found. Testing with empty ROM...");
        println!("Place a GBA ROM file in the current directory as 'pokemon_emerald.gba' to test with actual ROM data.");
    }
    
    println!("Starting emulator test...");
    println!("Initial CPU state:");
    println!("  PC: 0x{:08X}", gba.cpu.pc);
    println!("  SP: 0x{:08X}", gba.cpu.sp);
    
    for step in 0..5 {
        let old_pc = gba.cpu.pc;
        gba.step();
        println!("Step {}: PC 0x{:08X} -> 0x{:08X}, Cycles: {}", 
                step + 1, old_pc, gba.cpu.pc, gba.cycles);
    }
    
    println!("\nSTILL GOT IT BABYYYYYYY");
}
