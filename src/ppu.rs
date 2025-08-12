use crate::memory::Memory;

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
