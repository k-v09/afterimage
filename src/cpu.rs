use crate::memory::Memory;

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
        if !self.check_condition((instruction >> 28) & 0xF) {
            return;
        }

        if (instruction >> 25) & 0x7 == 0x5 {
            self.execute_branch(instruction);
            return;
        }

        if (instruction >> 26) & 0x3 == 0x1 {
            self.execute_single_data_transfer(instruction, memory);
            return;
        }

        let opcode = (instruction >> 21) & 0xF;
        
        match opcode {
            0xD => {
                let rd = ((instruction >> 12) & 0xF) as usize;
                let operand = self.get_data_processing_operand(instruction);
                self.set_register(rd, operand);
            }
            0x4 => {
                let rd = ((instruction >> 12) & 0xF) as usize;
                let rn = ((instruction >> 16) & 0xF) as usize;
                let operand = self.get_data_processing_operand(instruction);
                let result = self.get_register(rn).wrapping_add(operand);
                self.set_register(rd, result);
            }
            0x2 => {
                let rd = ((instruction >> 12) & 0xF) as usize;
                let rn = ((instruction >> 16) & 0xF) as usize;
                let operand = self.get_data_processing_operand(instruction);
                let result = self.get_register(rn).wrapping_sub(operand);
                self.set_register(rd, result);
            }
            0xA => {
                let rn = ((instruction >> 16) & 0xF) as usize;
                let operand = self.get_data_processing_operand(instruction);
                let rn_val = self.get_register(rn);
                let result = rn_val.wrapping_sub(operand);
                
                self.cpsr &= !0xF0000000; // Clear flags
                if result == 0 { self.cpsr |= 1 << 30; }
                if result & 0x80000000 != 0 { self.cpsr |= 1 << 31; }
                if rn_val >= operand { self.cpsr |= 1 << 29; }
            }
            // will add more processing instructions
            _ => {
                // this is just for debugging
                // println!("Unimplemented ARM data processing: 0x{:08X} at PC: 0x{:08X}", instruction, self.pc - 4);
            }
        }
    }

    fn execute_single_data_transfer(&mut self, instruction: u32, memory: &mut Memory) {
        let load = (instruction >> 20) & 1 == 1;
        let byte = (instruction >> 22) & 1 == 1;
        let up = (instruction >> 23) & 1 == 1;
        let pre = (instruction >> 24) & 1 == 1;
        let writeback = (instruction >> 21) & 1 == 1;

        let rd = ((instruction >> 12) & 0xF) as usize;
        let rn = ((instruction >> 16) & 0xF) as usize;
        
        let base = self.get_register(rn);
        let offset = if (instruction >> 25) & 1 == 1 {
            0
        } else {
            instruction & 0xFFF
        };

        let offset = if up { offset } else { 0u32.wrapping_sub(offset) };
        
        let address = if pre {
            base.wrapping_add(offset)
        } else {
            base
        };

        if load {
            let value = if byte {
                memory.read_u8(address) as u32
            } else {
                memory.read_u32(address)
            };
            self.set_register(rd, value);
        } else {
            let value = self.get_register(rd);
            if byte {
                memory.write_u8(address, value as u8);
            } else {
                memory.write_u32(address, value);
            }
        }

        if !pre || writeback {
            let new_base = if pre { 
                address 
            } else { 
                base.wrapping_add(offset) 
            };
            self.set_register(rn, new_base);
        }
    }

    fn get_data_processing_operand(&self, instruction: u32) -> u32 {
        if (instruction >> 25) & 1 == 1 {
            let imm = instruction & 0xFF;
            let rotate = ((instruction >> 8) & 0xF) * 2;
            imm.rotate_right(rotate)
        } else {
            let rm = (instruction & 0xF) as usize;
            self.get_register(rm)
        }
    }

    fn get_register(&self, reg: usize) -> u32 {
        match reg {
            0..=12 => self.registers[reg],
            13 => self.sp,
            14 => self.lr, 
            15 => self.pc + 8,
            _ => 0,
        }
    }

    fn set_register(&mut self, reg: usize, value: u32) {
        match reg {
            0..=12 => self.registers[reg] = value,
            13 => self.sp = value,
            14 => self.lr = value,
            15 => {
                self.pc = value & !0x3;
                // should also handle Thumb mode bit, but simplified for now
            }
            _ => {}
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
                // Sign extend 11-bit offset
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
                self.lr = self.pc | 1; // Set thumb bit in return address
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
