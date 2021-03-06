use rand::Rng;
use::minifb::{
    Key,
    Window
};
use crate::ram::Ram;

use crate::PROGRAM_START_ADDR;
use crate::NUM_GPR;
use crate::HEIGHT;
use crate::WIDTH;
use crate::PX_OFF;
use crate::PX_ON;

pub struct Cpu {
    // 16 8 bit general purpose registers
    reg_gpr: [u8; NUM_GPR],

    // 1 16 bit register, i
    reg_i: u16,

    // 2 special purpose 8 bit registers
    // delay and sound timers
    reg_dt: u8,
    reg_st: u8,

    // 16 bit program counter
    reg_pc: u16,

    // 8 bit stack pointer
    sp: u8,

    // stack
    stack: [u8; 16],

    // 8 bit graphics (gfx) array
    gfx: [u32; 64 * 32],

    // keyboard handling
    keys: [u8; 16]

}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            reg_gpr: [0; 16],
            reg_i: 0,
            reg_pc: PROGRAM_START_ADDR,
            sp: 0,
            stack: [0; 16],
            reg_dt: 0,
            reg_st: 0,
            gfx: [PX_OFF; 64 * 32],
            keys: [0; 16]
        }
    }

    pub fn reg_dt(&self) -> u8 {
        self.reg_dt
    }

    pub fn reg_st(&self) -> u8 {
        self.reg_st
    }

    pub fn reg_pc(&self) -> u16 {
        self.reg_pc
    }

    pub fn gfx(&self) -> [u32; 2048] {
        self.gfx
    }

    pub fn set_reg_dt(&mut self, reg_dt: u8) {
        self.reg_dt = reg_dt;
    }
    pub fn set_reg_st(&mut self, reg_st: u8) {
        self.reg_st = reg_st;
    }

    pub fn get_reg_vx(&self, opcode: u16) -> u8 {
        return self.reg_gpr[((opcode & 0x0F00) >> 8) as usize];
    }

    pub fn set_reg_vx(&mut self, opcode: u16, value: u8) {
        self.reg_gpr[((opcode & 0x0F00) >> 8) as usize] = value;
    }

    pub fn get_reg_vy(&self, opcode: u16) -> u8 {
        return self.reg_gpr[((opcode & 0x00F0) >> 4) as usize];
    }

    pub fn set_reg_vy(&mut self, opcode: u16, value: u8) {
        self.reg_gpr[((opcode & 0x00F0) >> 4) as usize] = value;
    }

    pub fn handle_keypress(&mut self, window: &Window) -> [u8; 16] {
        self.keys = [0; 16];
        window.get_keys().map(|keys_received| {
            for k in keys_received {
                match k {
                    Key::Key1 => self.keys[0x1] = 1,
                    Key::Key2 => self.keys[0x2] = 1,
                    Key::Key3 => self.keys[0x3] = 1,
                    Key::Key4 => self.keys[0xC] = 1,
                    Key::Q => self.keys[0x4] = 1,
                    Key::W => self.keys[0x5] = 1,
                    Key::E => self.keys[0x6] = 1,
                    Key::R => self.keys[0xD] = 1,
                    Key::A => self.keys[0x7] = 1,
                    Key::S => self.keys[0x8] = 1,
                    Key::D => self.keys[0x9] = 1,
                    Key::F => self.keys[0xE] = 1,
                    Key::Z => self.keys[0xA] = 1,
                    Key::X => self.keys[0x0] = 1,
                    Key::C => self.keys[0xB] = 1,
                    Key::V => self.keys[0xF] = 1,
                    _ => () // noop
                }
            }
        });
        self.keys
    }

    pub fn run_instruction(&mut self, ram: &mut Ram) {
        // fetch opcode Big Endian
        let hi = ram.read_byte(self.reg_pc) as u16;
        let lo = ram.read_byte(self.reg_pc+1) as u16;
        let instruction: u16 = (hi << 8) | lo;
        // decode and execute the opcode
        let reg_vx = self.get_reg_vx(instruction);
        let reg_vy = self.get_reg_vy(instruction);
        let nnn = instruction & 0x0FFF;
        let nn: u8 = (instruction & 0x00FF) as u8;
        let reg_v0 = self.reg_gpr[0] as u16;

        match instruction & 0xF000 {
            0x0000 => match instruction & 0x000F {
                0x0000 => {
                    // 0x00E0: clear screen
                    for index in 0..2048 {
                        self.gfx[index] = PX_OFF;
                    }
                    self.reg_pc += 2;
                },
                0x000E => {
                    // 0x00EE: return from subroutine
                    // restores program counter and then removes stack address
                    self.sp -= 1;
                    self.reg_pc = self.stack[self.sp as usize] as u16;
                    self.reg_pc += 2;
                },
                _ => println!("Invalid opcode {}", instruction)
            },
            0x1000 => {
                // 0x1NNN: jumps to address NNN
                self.reg_pc = nnn;
            },
            0x2000 => {
                // 0x2NNN: calls subroutine at NNN
                self.stack[self.sp as usize] = self.reg_pc as u8;
                self.sp += 1;
                self.reg_pc = instruction & 0x0FFF;
            },
            0x3000 => {
                // 0x3XNN: skips the next instruction if VX === NNN
                if reg_vx == nn {
                    self.reg_pc += 2;
                }
                self.reg_pc += 2;
            },
            0x4000 => {
                // 0x4XNN: skips the next instruction if VX !== NNN
                if reg_vx != nn {
                    self.reg_pc += 2;
                }
                self.reg_pc += 2;
            },
            0x5000 => {
                // 0x5XY0: skips the next instruction if VX === VY
                if reg_vx == reg_vy {
                    self.reg_pc += 2;
                }
                self.reg_pc += 2;
            },
            0x6000 => {
                // 0x6XNN: sets VX to NN
                self.set_reg_vx(instruction, nn);
                self.reg_pc += 2;
            },
            0x7000 => {
                // 0x7XNN: Adds NN to VX (carry flag is not changed)
                self.set_reg_vx(instruction, reg_vx.wrapping_add(nn));
                self.reg_pc += 2;
            },
            0x8000 => {
                match instruction & 0x000F {
                    0x0000 => {
                        // 0x8XY0: sets VX = VY
                        self.set_reg_vx(instruction, reg_vy);
                        self.reg_pc += 2;
                    },
                    0x0001 => {
                        // 0x8XY1: bitwise OR -> VX | VY, store in VX
                        self.set_reg_vx(instruction, reg_vx | reg_vy);
                        self.reg_pc += 2;
                    },
                    0x0002 => {
                        // 0x8XY2: bitwise AND -> VX & VY
                        self.set_reg_vx(instruction, reg_vx & reg_vy);
                        self.reg_pc += 2;
                    },
                    0x0003 => {
                        // 0x8XY3: XOR -> VX XOR VY
                        self.set_reg_vx(instruction, reg_vx ^ reg_vy);
                        self.reg_pc += 2;
                    },
                    0x0004 => {
                        // 0x8XY4: adds VY to VX. VF is set to 1 when there's a carry
                        // and a 0 when when there isn't
                        if reg_vy > (0x00FF - reg_vx) {
                            self.reg_gpr[0xF] = 1;
                        } else {
                            self.reg_gpr[0xF] = 0;
                        }
                        self.set_reg_vx(instruction, reg_vx.wrapping_add(reg_vy));
                        self.reg_pc += 2;

                    },
                    0x0005 => {
                        // 0x8XY5: subtracts VY from VX. VF is set to 1 when there's a carry
                        // and a 0 when when there isn't
                        if reg_vx > reg_vy {
                            self.reg_gpr[0xF] = 1;
                        } else {
                            self.reg_gpr[0xF] = 0;
                        }
                        self.set_reg_vx(instruction, reg_vx - reg_vy);
                        self.reg_pc += 2;
                    },
                    0x0006 => {
                        // 0x8XY6: stores the LSB of VX in VF and then shifts VX to the right by 1
                        self.reg_gpr[0xF] = reg_vx & 1;
                        self.set_reg_vx(instruction, reg_vx >> 1);
                        self.reg_pc += 2;
                    },
                    0x0007 => {
                        // 0x8XY7: sets VX to VY minus VX.
                        // VF is set to 0 when there's a borrow, and 1 when there isn't
                        if reg_vy > reg_vx {
                            self.reg_gpr[0xF] = 1;
                        } else {
                            self.reg_gpr[0xF] = 0;
                        }
                        self.set_reg_vy(instruction, reg_vy - reg_vx);
                        self.reg_pc += 2;
                    },
                    0x000E => {
                        // 0x8XYE: stores the MSB of VX in VF and then shifts VX to the left by 1
                        self.reg_gpr[0xF] = (reg_vx >> 3) & 1;
                        self.set_reg_vx(instruction, reg_vx << 1);
                        self.reg_pc += 2;
                    },
                    _ => println!("Invalid opcode {}", instruction)
                }
            },
            0x9000 => {
                // 0x9XY0: skips the next instruction if VX doesn't equal VY
                if reg_vx != reg_vy {
                    self.reg_pc += 2;
                }
                self.reg_pc += 2;
            },
            0xA000 => {
                // 0xANNN: sets I to the address NNN
                self.reg_i = nnn;
                self.reg_pc += 2;
            },
            0xB000 => {
                // 0xBNNN: jumps to the address NNN plus V0
                self.reg_pc = nnn + reg_v0;
            },
            0xC000 => {
                // 0xCXNN: sets VX to the result of a bitwise AND operation
                // on a random number (Typically: 0 to 255) and NN
                let mut rng = rand::thread_rng();
                let rand_num: u8 = rng.gen();
                self.set_reg_vx(instruction, rand_num & nn as u8);
                self.reg_pc += 2;
            },
            0xD000 => {
                // 0xDXYN: draws a sprite at coordinate (VX, VY), has a width of 8 pixels and
                // a height of N + 1 pixels.
                let x = reg_vx;
                let y = reg_vy;
                let height = (instruction & 0x000F) as u8;
                let mut pixel: u8;
                // set VF to 0
                self.reg_gpr[0xF] = 0;

                for y_line in 0..height {
                    // get one byte of sprite data from the mem address in the i register
                    // pixel = ram.memory[(self.reg_i + y_line as u16) as usize];
                    pixel = ram.read_byte(self.reg_i + y_line as u16);
                    for x_line in 0..8 {
                        if (pixel & (0x0080 >> x_line)) != 0 {
                            let pos_x: u32 = (x as u32 + x_line as u32) % WIDTH as u32;
                            let pos_y: u32 = (y as u32 + y_line as u32) % HEIGHT as u32;
                            if self.gfx[(pos_x + (pos_y * 64 as u32)) as usize] == PX_ON {
                                self.reg_gpr[0xF] = 1;
                            }
                            self.gfx[(pos_x + (pos_y * 64 as u32)) as usize] ^= PX_ON;
                        }
                    }
                }
                self.reg_pc += 2;
            },
            0xE000 => {
                match instruction & 0x000F {
                    0x000E => {
                        // 0xEX9E: skips the next instruction if the key stored in VX is pressed
                        if self.keys[reg_vx as usize] != 0 {
                            self.reg_pc += 2;
                        }
                        self.reg_pc += 2;
                    },
                    0x0001 => {
                        // 0xEXA1: skips the next instruction if the key stored in VX isn't pressed
                        if self.keys[reg_vx as usize] == 0 {
                            self.reg_pc += 2;
                        }
                        self.reg_pc += 2;
                    },
                    _ => println!("Invalid opcode! {:#X}", instruction)
                }
            },
            0xF000 => {
                match instruction & 0x000F {
                    0x0007 => {
                        // 0xFX07: the value of DT is placed in VX
                        self.set_reg_vx(instruction, self.reg_dt);
                        self.reg_pc += 2;
                    },
                    0x000A => {
                        // 0xFX0A: wait for key press, store the value of key into VX
                        // keypad logic
                        let mut key_pressed = false;
                        for i in 0..self.keys.len() {
                            if self.keys[i] != 0 {
                                key_pressed = true;
                                self.set_reg_vx(instruction, i as u8);
                                break;
                            }
                        }

                        // if no key is pressed, try again
                        if !key_pressed {
                            return;
                        }

                        self.reg_pc += 2;
                    },
                    0x0005 => {
                        match instruction & 0x00FF {
                            0x0015 => {
                                // 0xFX15: set DT to VX
                                self.reg_dt = reg_vx;
                                self.reg_pc += 2;
                            },
                            0x0055 => {
                                // 0xFX55: store registers V0 -> VX in memory starting at
                                // location I
                                let x = (instruction & 0x0F00) >> 8;
                                for index in 0..=x {
                                    ram.write_byte(self.reg_i + index, self.reg_gpr[index as usize]);
                                }
                            },
                            0x0065 => {
                                // 0xFX65: read registers V0 -> Vx from memory starting at
                                // location I
                                let x = (instruction & 0x0F00) >> 8;
                                for index in 0..=x {
                                    self.set_reg_vx(instruction, ram.read_byte(self.reg_i + index));
                                }
                            },
                            _ => println!("Invalid opcode! {:#X}", instruction)
                        }
                    },
                    0x0008 => {
                        // 0xFX18: set ST to VX
                        self.reg_st = reg_vx;
                        self.reg_pc += 2;
                    },
                    0x000E => {
                        // 0xFX1E: set I = I + VX
                        self.reg_i += reg_vx as u16;
                        self.reg_pc += 2;
                    },
                    0x0009 => {
                        // 0xFX29: set I = location of sprite for digit VX
                        self.reg_i = self.gfx[reg_vx as usize] as u16;
                        self.reg_pc += 2;
                    },
                    0x0003 => {
                        // 0xFX33: store BCD representation of VX in memory locations
                        // I, I+1, I+2
                        ram.write_byte(self.reg_i, reg_vx / 100);
                        ram.write_byte(self.reg_i + 1, (reg_vx / 10) % 10);
                        ram.write_byte(self.reg_i + 2, (reg_vx % 100) % 10);
                        self.reg_pc += 2;
                    },
                    _ => println!("Invalid opcode! {:#X}", instruction)
                }
            }
            _ => println!("Invalid opcode! {:#X}", instruction)
        }
    }
}