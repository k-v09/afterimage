use crate::cpu::Cpu;
use crate::memory::Memory;
use crate::ppu::Ppu;

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
