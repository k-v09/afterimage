use std::fs::File;
use std::io::Read;

// ARM7TDMI CPU State
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

    fn execute_arm(&mut self, instruction: u32, memory: &mut Memory) {
        let opcode = (instruction >> 21) & 0xF;
        
        match opcode {
            0xD => {
                let rd = ((instruction >> 12) & 0xF) as usize;
                let operand = instruction & 0xFFF;
                if rd < 13 {
                    self.registers[rd] = operand;
                }
            }
            // will add more ARM instructions here
            _ => {
                println!("Unimplemented ARM instruction: 0x{:08X}", instruction);
            }
        }
    }

    fn execute_thumb(&mut self, instruction: u16, memory: &mut Memory) {
        let opcode = (instruction >> 11) & 0x1F;
        
        match opcode {
            0x4 => {
                let rd = ((instruction >> 8) & 0x7) as usize;
                let imm = (instruction & 0xFF) as u32;
                self.registers[rd] = imm;
            }
            // will add more Thumb instructions here
            _ => {
                println!("Unimplemented Thumb instruction: 0x{:04X}", instruction);
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
                    0xFF // 0xFF for unmapped
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

    pub fn step(&mut self, memory: &Memory) {
        // just incrementing scanline
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
        let target_cycles = self.cycles + 280_896;
        while self.cycles < target_cycles {
            self.step();
        }
    }
}

fn main() {
    let mut gba = Gba::new();
    
    match gba.load_rom("pokemon_emerald.gba") {
        Ok(_) => println!("ROM loaded successfully"),
        Err(e) => {
            println!("Failed to load ROM: {}", e);
            return;
        }
    }
    
    for frame in 0..10 {
        gba.run_frame();
        println!("Completed frame {}, cycles: {}", frame, gba.cycles);
    }
}
