use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct Cpu {
    pub registers: [u32; 13],
    pub sp: u32,
    pub lr: u32,
    pub pc: u32,
    pub cpsr: u32,
    pub spsr: [u32; 5],
    pub mode: CpuMode,
    pub thumb_mode: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum CpuMode {
    User = 0x10,
    Fiq = 0x11,
    Irq = 0x12,
    Supervisor = 0x13,
    Abort = 0x17,
    Undefined = 0x1B,
    System = 0x1F,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: [0; 13],
            sp: 0x03007F00,
            lr: 0,
            pc: 0x08000000,
            cpsr: 0x1F,
            spsr: [0; 5],
            mode: CpuMode::System,
            thumb_mode: false,
        }
    }

    pub fn step(&mut self, memory: &mut Memory) {
        let instruction = if self.thumb_mode {
            memory.read_u16(self.pc) as u32
        } else {
            memory.read_u32(self.pc)
        };

        self.pc += if self.thumb_mode { 2 } else { 4 };
        
        if self.thumb_mode {
            self.execute_thumb(instruction as u16, memory);
        } else {
            self.execute_arm(instruction, memory);
        }
    }

    fn execute_arm(&mut self, instruction: u32, _memory: &mut Memory) {
        if !self.check_condition((instruction >> 28) & 0xF) {
            return;
        }

        if (instruction >> 25) & 0x7 == 0x5 {
            self.execute_branch(instruction);
            return;
        }

        let opcode = (instruction >> 21) & 0xF;
        
        match opcode {
            0xD => {
                let rd = ((instruction >> 12) & 0xF) as usize;
                let operand = instruction & 0xFFF;
                if rd < 13 {
                    self.registers[rd] = operand;
                } else if rd == 15 {
                    self.pc = operand;
                }
            }
            // will add more processing instructions
            _ => {
                // this is just for debugging
                // println!("Unimplemented ARM instruction: 0x{:08X} at PC: 0x{:08X}", instruction, self.pc - 4);
            }
        }
    }

    fn execute_branch(&mut self, instruction: u32) {
        let link = (instruction >> 24) & 1 == 1;
        
        let mut offset = instruction & 0xFFFFFF;
        if offset & 0x800000 != 0 {
            offset |= 0xFF000000;
        }
        
        let offset = ((offset as i32) << 2) as u32;
        
        if link {
            self.lr = self.pc;
        }
        
        self.pc = ((self.pc as i32) + (offset as i32) + 4) as u32;
    }

    fn check_condition(&self, condition: u32) -> bool { // flags
        let n = (self.cpsr >> 31) & 1 == 1; // neg
        let z = (self.cpsr >> 30) & 1 == 1; // zero  
        let c = (self.cpsr >> 29) & 1 == 1; // carry
        let v = (self.cpsr >> 28) & 1 == 1; // ovf
        
        match condition {
            0x0 => z,                    // EQ - Equal (Z set)
            0x1 => !z,                   // NE - Not Equal (Z clear)
            0x2 => c,                    // CS/HS - Carry Set/Unsigned Higher or Same
            0x3 => !c,                   // CC/LO - Carry Clear/Unsigned Lower
            0x4 => n,                    // MI - Minus/Negative
            0x5 => !n,                   // PL - Plus/Positive or Zero
            0x6 => v,                    // VS - Overflow Set
            0x7 => !v,                   // VC - Overflow Clear
            0x8 => c && !z,              // HI - Unsigned Higher
            0x9 => !c || z,              // LS - Unsigned Lower or Same
            0xA => n == v,               // GE - Signed Greater or Equal
            0xB => n != v,               // LT - Signed Less Than
            0xC => !z && (n == v),       // GT - Signed Greater Than
            0xD => z || (n != v),        // LE - Signed Less or Equal
            0xE => true,                 // AL - Always (unconditional)
            0xF => false,                // Reserved (should not occur)
            _ => false,
        }
    }

    fn execute_thumb(&mut self, instruction: u16, _memory: &mut Memory) {
        let opcode = (instruction >> 11) & 0x1F;
        
        match opcode {
            0x1C => {
                let mut offset = instruction & 0x7FF;
                if offset & 0x400 != 0 {
                    offset |= 0xF800;
                }
                let offset = ((offset as i16) << 1) as i32;
                self.pc = ((self.pc as i32) + offset + 2) as u32;
            }
            0x1A..=0x1B => {
                let condition = (instruction >> 8) & 0xF;
                if condition != 0xF && self.check_condition(condition as u32) {
                    let mut offset = instruction & 0xFF;
                    if offset & 0x80 != 0 {
                        offset |= 0xFF00;
                    }
                    let offset = ((offset as i16) << 1) as i32;
                    self.pc = ((self.pc as i32) + offset + 2) as u32;
                }
            }
            0x1E => {
                let offset_high = instruction & 0x7FF;
                let mut full_offset = (offset_high as u32) << 12;
                if offset_high & 0x400 != 0 {
                    full_offset |= 0xFF800000;
                }
                self.lr = (self.pc as i32 + full_offset as i32 + 2) as u32;
            }
            0x1F => {
                let offset_low = instruction & 0x7FF;
                let target = self.lr + ((offset_low as u32) << 1);
                self.lr = self.pc | 1;
                self.pc = target;
            }
            0x4 => {
                let rd = ((instruction >> 8) & 0x7) as usize;
                let imm = (instruction & 0xFF) as u32;
                self.registers[rd] = imm;
            }
            // will add more thumb
            _ => {
                // again for debugging
                // println!("Unimplemented Thumb instruction: 0x{:04X} at PC: 0x{:08X}", instruction, self.pc - 2);
            }
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    pub bios: Vec<u8>,
    pub ewram: Vec<u8>,
    pub iwram: Vec<u8>, 
    pub vram: Vec<u8>,
    pub palette_ram: Vec<u8>,
    pub oam: Vec<u8>,
    pub rom: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            bios: vec![0; 0x4000],        // 16KB
            ewram: vec![0; 0x40000],      // 256KB  
            iwram: vec![0; 0x8000],       // 32KB
            vram: vec![0; 0x18000],       // 96KB
            palette_ram: vec![0; 0x400],  // 1KB
            oam: vec![0; 0x400],          // 1KB
            rom: Vec::new(),
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;
        self.rom.clear();
        file.read_to_end(&mut self.rom)?;
        println!("Loaded ROM: {} bytes", self.rom.len());
        Ok(())
    }

    pub fn read_u8(&self, address: u32) -> u8 {
        match address {
            0x00000000..=0x00003FFF => self.bios[(address & 0x3FFF) as usize],
            0x02000000..=0x0203FFFF => self.ewram[(address & 0x3FFFF) as usize],
            0x03000000..=0x03007FFF => self.iwram[(address & 0x7FFF) as usize],
            0x06000000..=0x06017FFF => self.vram[(address & 0x17FFF) as usize],
            0x05000000..=0x050003FF => self.palette_ram[(address & 0x3FF) as usize],
            0x07000000..=0x070003FF => self.oam[(address & 0x3FF) as usize],
            0x08000000..=0x09FFFFFF => {
                let rom_addr = (address - 0x08000000) as usize;
                if rom_addr < self.rom.len() {
                    self.rom[rom_addr]
                } else {
                    0xFF
                }
            }
            _ => {
                println!("Unhandled memory read at 0x{:08X}", address);
                0xFF
            }
        }
    }

    pub fn read_u16(&self, address: u32) -> u16 {
        let low = self.read_u8(address) as u16;
        let high = self.read_u8(address + 1) as u16;
        low | (high << 8)
    }

    pub fn read_u32(&self, address: u32) -> u32 {
        let low = self.read_u16(address) as u32;
        let high = self.read_u16(address + 2) as u32;
        low | (high << 16)
    }

    pub fn write_u8(&mut self, address: u32, value: u8) {
        match address {
            0x02000000..=0x0203FFFF => self.ewram[(address & 0x3FFFF) as usize] = value,
            0x03000000..=0x03007FFF => self.iwram[(address & 0x7FFF) as usize] = value,
            0x06000000..=0x06017FFF => self.vram[(address & 0x17FFF) as usize] = value,
            0x05000000..=0x050003FF => self.palette_ram[(address & 0x3FF) as usize] = value,
            0x07000000..=0x070003FF => self.oam[(address & 0x3FF) as usize] = value,
            _ => {
                println!("Unhandled memory write at 0x{:08X} = 0x{:02X}", address, value);
            }
        }
    }
}

#[derive(Debug)]
pub struct Ppu {
    pub vcount: u16,
    pub frame_buffer: Vec<u16>,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            vcount: 0,
            frame_buffer: vec![0; 240 * 160],
        }
    }

    pub fn step(&mut self, _memory: &Memory) {
        self.vcount = (self.vcount + 1) % 228;
        
        // TODO: Implement actual rendering logic
        // - Read background control registers
        // - Render backgrounds based on mode
        // - Render sprites
        // - Handle palette lookups
    }
}

pub struct Gba {
    pub cpu: Cpu,
    pub memory: Memory,
    pub ppu: Ppu,
    pub cycles: u64,
}

impl Gba {
    pub fn new() -> Self {
        Gba {
            cpu: Cpu::new(),
            memory: Memory::new(),
            ppu: Ppu::new(),
            cycles: 0,
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), std::io::Error> {
        self.memory.load_rom(path)
    }

    pub fn step(&mut self) {
        self.cpu.step(&mut self.memory);
        
        self.ppu.step(&self.memory);
        
        self.cycles += 1;
        
        // TODO: Handle interrupts, timers, DMA, etc.
    }

    pub fn run_frame(&mut self) {
        // Run until complete one frame (about 280,896 cycles)
        let target_cycles = self.cycles + 280_896;
        while self.cycles < target_cycles {
            self.step();
        }
    }
}

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
    
    println!("\nBasic functionality achieved\nI am officially cool");
}
