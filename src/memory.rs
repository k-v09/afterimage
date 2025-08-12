use std::fs::File;
use std::io::Read;

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
            // return 0 for now
            0x04000000..=0x040003FF => {
                // TODO: Implement I/O register handling
                // This includes graphics, sound, timers, DMA, etc.
                0
            }
            _ => {
                // another debug
                // println!("Unhandled memory read at 0x{:08X}", address);
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
            0x04000000..=0x040003FF => {
                // TODO: Implement I/O register handling
            }
            _ => {
                // Remove Insect
                // println!("Unhandled memory write at 0x{:08X} = 0x{:02X}", address, value);
            }
        }
    }

    pub fn write_u16(&mut self, address: u32, value: u16) {
        self.write_u8(address, value as u8);
        self.write_u8(address + 1, (value >> 8) as u8);
    }

    pub fn write_u32(&mut self, address: u32, value: u32) {
        self.write_u16(address, value as u16);
        self.write_u16(address + 2, (value >> 16) as u16);
    }
}
