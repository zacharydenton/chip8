use std::time::{Duration, Instant};

extern crate rand;
use rand::Rng;

pub struct Chip8 {
    pub i: usize,
    pub pc: usize,
    pub sp: usize,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: [usize; 32],
    pub registers: [u8; 16],
    pub memory: [u8; 4096],
    pub graphics: [u8; 64 * 32],
    last_tick: Instant,
    timer_interval: Duration,
}

impl<'a> Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            i: 0,
            pc: 0x200,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            stack: [0; 32],
            memory: [0; 4096],
            graphics: [0; 64 * 32],
            last_tick: Instant::now(),
            timer_interval: Duration::from_secs(1).checked_div(60).unwrap(),
        };

        // Initalize fonts at the start of system memory.
        for i in 0..FONTS.len() {
            chip8.memory[i] = FONTS[i];
        }

        chip8
    }

    pub fn load(&mut self, program: &[u8]) {
        self.i = 0;
        self.pc = 0x200;
        self.sp = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        for i in 0..self.registers.len() {
            self.registers[i] = 0;
        }
        for i in 0..self.stack.len() {
            self.stack[i] = 0;
        }
        for i in 0..program.len() {
            self.memory[self.pc + i] = program[i];
        }
        for i in 0..self.graphics.len() {
            self.graphics[i] = 0;
        }
    }

    pub fn cycle<R: Rng>(&mut self, rng: &'a mut R) {
        // 0xEX9E: Skip next instruction if VX = hexadecimal key (LSD)
        // 0xEXA1: Skip next instruction if VX != hexadecimal key (LSD)
        // 0xFX0A: Let VX = hexadecimal key digit (waits for any key pressed)
        // 0xFX18: Set tone duration = VX (0x01 = 1/60 second)
        // 0x0MMM: Do machine language at 0x0MMM (subroutine must end with 0xD4 byte)
        match self.fetch_op() {
            (0x1, a, b, c) => {
                // 0x1MMM: Go to 0x0MMM
                let mmm = ((a as usize) << 8) + ((b as usize) << 4) + (c as usize);
                self.go_to(mmm);
            }
            (0xB, a, b, c) => {
                // 0xBMMM: Go to 0x0MMM + V0
                let mmm = ((a as usize) << 8) + ((b as usize) << 4) + (c as usize);
                let v0 = self.registers[0] as usize;
                self.go_to(mmm + v0);
            }
            (0x2, a, b, c) => {
                // 0x2MMM: Do subroutine at 0x0MMM (must end with 0x00EE)
                let mmm = ((a as usize) << 8) + ((b as usize) << 4) + (c as usize);
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.go_to(mmm);
            }
            (0x0, 0x0, 0xE, 0xE) => {
                // 0x00EE: Return from subroutine
                self.sp -= 1;
                let return_address = self.stack[self.sp];
                self.go_to(return_address);
                self.next();
            }
            (0x3, x, a, b) => {
                // 0x3XKK: Skip next instruction if VX = KK
                let kk = (a << 4) + b;
                let vx = self.registers[x as usize];
                self.skip_if(vx == kk);
            }
            (0x4, x, a, b) => {
                // 0x4XKK: Skip next instruction if VX != KK
                let kk = (a << 4) + b;
                let vx = self.registers[x as usize];
                self.skip_if(vx != kk);
            }
            (0x5, x, y, 0x0) => {
                // 0x5XY0: Skip next instruction if VX = VY
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                self.skip_if(vx == vy);
            }
            (0x9, x, y, 0x0) => {
                // 0x9XY0: Skip next instruction if VX != VY
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                self.skip_if(vx != vy);
            }
            (0x6, x, a, b) => {
                // 0x6XKK: Let VX = KK
                let kk = (a << 4) + b;
                self.registers[x as usize] = kk;
                self.next();
            }
            (0x7, x, a, b) => {
                // 0x7XKK: Let VX = VX + KK
                let kk = (a << 4) + b;
                let vx = self.registers[x as usize];
                self.registers[x as usize] = vx.wrapping_add(kk);
                self.next();
            }
            (0x8, x, y, 0x0) => {
                // 0x8XY0: Let VX = VY
                let vy = self.registers[y as usize];
                self.registers[x as usize] = vy;
                self.next();
            }
            (0x8, x, y, 0x1) => {
                // 0x8XY1: Let VX = VX | VY (VF changed)
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                self.registers[x as usize] = vx | vy;
                self.next();
            }
            (0x8, x, y, 0x2) => {
                // 0x8XY2: Let VX = VX & VY (VF changed)
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                self.registers[x as usize] = vx & vy;
                self.next();
            }
            (0x8, x, y, 0x4) => {
                // 0x8XY4: Let VX = VX + VY (VF = 0x00 if VX + VY <= 0xFF, VF = 0x01 if VX + VY > 0xFF)
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                let r = vx.wrapping_add(vy);
                self.registers[x as usize] = r;
                self.registers[0xF] = if r < vx { 1 } else { 0 };
                self.next();
            }
            (0x8, x, y, 0x5) => {
                // 0x8XY5: Let VX = VX - VY (VF = 0x00 if VX < VY, VF = 0x01 if VX >= VY)
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                let r = vx.wrapping_sub(vy);
                self.registers[x as usize] = r;
                self.registers[0xF] = if vx < vy { 0 } else { 1 };
                self.next();
            }
            (0xC, x, a, b) => {
                // 0xCXKK: Let VX = Random Byte (KK = Mask)
                let kk = (a << 4) + b;
                let rb: u8 = rng.gen();
                self.registers[x as usize] = rb & kk;
                self.next();
            }
            (0xA, a, b, c) => {
                // 0xAMMM: Let I = 0x0MMM
                let mmm = ((a as usize) << 8) + ((b as usize) << 4) + (c as usize);
                self.i = mmm;
                self.next();
            }
            (0xF, x, 0x1, 0xE) => {
                // 0xFX1E: Let I = I + VX
                let vx = self.registers[x as usize];
                self.i += vx as usize;
                self.next();
            }
            (0xF, x, 0x2, 0x9) => {
                // 0xFX29: Let I = 5 byte display pattern for LSD of VX
                let vx = self.registers[x as usize];
                let lsd = vx & 0xF;
                self.i = 5 * lsd as usize;
                self.next();
            }
            (0xF, x, 0x3, 0x3) => {
                // 0xFX33: Let MI = 3 decimal digit equivalent of VX (I unchanged)
                let vx = self.registers[x as usize];
                self.memory[self.i + 0] = vx / 100;
                self.memory[self.i + 1] = vx / 10 % 10;
                self.memory[self.i + 2] = vx % 10;
                self.next();
            }
            (0xF, x, 0x5, 0x5) => {
                // 0xFX55: Let MI = V0 : VX (I = I + X + 1)
                for i in 0..((x + 1) as usize) {
                    let vx = self.registers[i];
                    self.memory[self.i + i] = vx;
                }
                self.next();
            }
            (0xF, x, 0x6, 0x5) => {
                // 0xFX65: Let V0 : VX = MI (I = I + X + 1)
                for i in 0..((x + 1) as usize) {
                    let mx = self.memory[self.i + i];
                    self.registers[i] = mx;
                }
                self.next();
            }
            (0x0, 0x0, 0xE, 0x0) => {
                // 0x00E0: Erase display (all 0s)
                for i in 0..self.graphics.len() {
                    self.graphics[i] = 0;
                }
                self.next();
            }
            (0xD, x, y, n) => {
                // 0xDXYN: Show n byte MI pattern at VX-VY coordinates. I unchanged. MI pattern is
                // combined with existing display via EXCLUSIVE-OR function. VF = 0x01 if a 1 in MI
                // pattern matches 1 in existing display.
                self.registers[0xF] = 0x0;
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                for i in 0..n {
                    let mi = self.memory[self.i + i as usize];
                    for j in 0..8 {
                        let bit = (mi >> (7 - j)) & 1;
                        let index = 64 * ((vy + i) as usize) + ((vx + j) as usize);
                        if bit == 1 && self.graphics[index] == 1 {
                            self.registers[0xF] = 0x1;
                        }
                        self.graphics[index] ^= bit;
                    }
                }
                self.next();
            }
            (0xF, x, 0x0, 0x7) => {
                // 0xFX07: Let VX = current timer value
                self.registers[x as usize] = self.delay_timer;
                self.next();
            }
            (0xF, x, 0x1, 0x5) => {
                // 0xFX15: Set timer = VX (0x01 = 1/60 second)
                let vx = self.registers[x as usize];
                self.delay_timer = vx;
                self.next();
            }
            (a, b, c, d) => {
                panic!(
                    "Attempted to execute unsupported instruction: 0x{:X}{:X}{:X}{:X}",
                    a, b, c, d
                );
            }
        }

        if self.last_tick.elapsed() >= self.timer_interval {
            self.delay_timer = self.delay_timer.saturating_sub(1);
            self.sound_timer = self.sound_timer.saturating_sub(1);
            self.last_tick = Instant::now();
        }
    }

    fn fetch_op(&self) -> (u8, u8, u8, u8) {
        (
            self.memory[self.pc] >> 4,
            self.memory[self.pc] & 0xF,
            self.memory[self.pc + 1] >> 4,
            self.memory[self.pc + 1] & 0xF,
        )
    }

    fn go_to(&mut self, address: usize) {
        self.pc = address;
    }

    fn next(&mut self) {
        self.pc += 2;
    }

    fn skip_if(&mut self, condition: bool) {
        if condition {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }
}

static FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_chip8() {
        let chip8 = Chip8::new();
        assert!(chip8.memory.len() == 4096);
        assert!(chip8.stack.len() == 32);
        assert!(chip8.registers.len() == 16);
        assert!(chip8.pc == 0x200);
        for i in 0..FONTS.len() {
            assert!(chip8.memory[i] != 0);
        }
    }

    #[test]
    fn fetch_op() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x200] = 0xF0;
        chip8.memory[0x201] = 0x00;
        chip8.memory[0x202] = 0xD3;
        chip8.memory[0x203] = 0x40;
        assert!(chip8.fetch_op() == (0xF, 0x0, 0x0, 0x0));
        chip8.pc += 2;
        assert!(chip8.fetch_op() == (0xD, 0x3, 0x4, 0x0));
    }

    #[test]
    fn op_1mmm() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x13;
        chip8.memory[0x201] = 0x5F;
        chip8.memory[0x35F] = 0x12;
        chip8.memory[0x35F + 1] = 0x00;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x35F);
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x200);
    }

    #[test]
    fn op_bmmm() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0xB3;
        chip8.memory[0x201] = 0x00;
        chip8.registers[0] = 0xF0;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x300 + 0xF0);
    }

    #[test]
    fn op_3xkk() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x33;
        chip8.memory[0x201] = 0x42;
        chip8.registers[3] = 0x41;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x202);
        chip8.pc = 0x200;
        chip8.registers[3] = 0x42;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x204);
    }

    #[test]
    fn op_4xkk() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x4F;
        chip8.memory[0x201] = 0xF0;
        chip8.registers[0xF] = 0xF0;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x202);
        chip8.pc = 0x200;
        chip8.registers[0xF] = 0x42;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x204);
    }

    #[test]
    fn op_5xy0() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x50;
        chip8.memory[0x201] = 0xB0;
        chip8.registers[0] = 0x33;
        chip8.registers[0xB] = 0x23;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x202);
        chip8.pc = 0x200;
        chip8.registers[0xB] = 0x33;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x204);
    }

    #[test]
    fn op_9xy0() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x9C;
        chip8.memory[0x201] = 0xA0;
        chip8.registers[0xC] = 0xFF;
        chip8.registers[0xA] = 0xEE;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x204);
        chip8.pc = 0x200;
        chip8.registers[0xA] = 0xFF;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x202);
    }

    #[test]
    fn op_6xkk() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x68;
        chip8.memory[0x201] = 0x42;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[0x8] == 0x42);
    }

    #[test]
    fn op_7xkk() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x7A;
        chip8.memory[0x201] = 0x10;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[0xA] == 0x10);

        // Overflow should wrap around.
        chip8.pc = 0x200;
        chip8.memory[0x201] = 0xFF;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[0xA] == 0x10 - 1);
    }

    #[test]
    fn op_8xy0() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x8A;
        chip8.memory[0x201] = 0xB0;
        chip8.registers[0xB] = 0xF0;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[0xA] == 0xF0);
        assert!(chip8.registers[0xB] == 0xF0);
    }

    #[test]
    fn op_8xy1() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x83;
        chip8.memory[0x201] = 0x41;
        chip8.registers[0x3] = 0x39;
        chip8.registers[0x4] = 0xCD;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[0x3] == 0x39 | 0xCD);
    }

    #[test]
    fn op_8xy2() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x83;
        chip8.memory[0x201] = 0x42;
        chip8.registers[0x3] = 0x39;
        chip8.registers[0x4] = 0xCD;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[0x3] == 0x39 & 0xCD);
    }

    #[test]
    fn op_8xy4() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x83;
        chip8.memory[0x201] = 0x44;
        chip8.registers[0x3] = 0x39;
        chip8.registers[0x4] = 0x0D;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[0x3] == 0x39 + 0x0D);
        assert!(chip8.registers[0xF] == 0);

        // Overflow should wrap around.
        chip8.pc = 0x200;
        chip8.registers[0x4] = 0xFF;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[0x3] == (0x39 + 0x0D) - 1);
        assert!(chip8.registers[0xF] == 1);
    }

    #[test]
    fn op_8xy5() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x83;
        chip8.memory[0x201] = 0x45;
        chip8.registers[0x3] = 0x39;
        chip8.registers[0x4] = 0x0D;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[0x3] == 0x39 - 0x0D);
        assert!(chip8.registers[0xF] == 1);

        // Overflow should wrap around.
        chip8.pc = 0x200;
        chip8.registers[0x4] = 0xFF;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[0x3] == (0x39 - 0x0D) + 1);
        assert!(chip8.registers[0xF] == 0);
    }

    #[test]
    fn op_ammm() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0xA9;
        chip8.memory[0x201] = 0x08;
        chip8.cycle(&mut rng);
        assert!(chip8.i == 0x908);
    }

    #[test]
    fn op_fx1e() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0xF4;
        chip8.memory[0x201] = 0x1E;
        chip8.i = 0x500;
        chip8.registers[4] = 0x20;
        chip8.cycle(&mut rng);
        assert!(chip8.i == 0x500 + 0x20);
    }

    #[test]
    fn op_fx29() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0xFC;
        chip8.memory[0x201] = 0x29;
        chip8.registers[0xC] = 0x0A;
        chip8.cycle(&mut rng);
        for i in 0..5 {
            assert!(chip8.memory[chip8.i + i] == FONTS[(5 * 0xA) + i]);
        }
        chip8.pc = 0x200;
        chip8.memory[0x200] = 0xFC;
        chip8.memory[0x201] = 0x29;
        chip8.registers[0xC] = 0xD1;
        chip8.cycle(&mut rng);
        for i in 0..5 {
            assert!(chip8.memory[chip8.i + i] == FONTS[(5 * 0x1) + i]);
        }
    }

    #[test]
    fn op_fx33() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0xF1;
        chip8.memory[0x201] = 0x33;
        chip8.registers[1] = 243;
        chip8.i = 0x500;
        chip8.cycle(&mut rng);
        assert!(chip8.memory[chip8.i + 0] == 2);
        assert!(chip8.memory[chip8.i + 1] == 4);
        assert!(chip8.memory[chip8.i + 2] == 3);
        chip8.pc = 0x200;
        chip8.registers[1] = 91;
        chip8.cycle(&mut rng);
        assert!(chip8.memory[chip8.i + 0] == 0);
        assert!(chip8.memory[chip8.i + 1] == 9);
        assert!(chip8.memory[chip8.i + 2] == 1);
        chip8.pc = 0x200;
        chip8.registers[1] = 5;
        chip8.cycle(&mut rng);
        assert!(chip8.memory[chip8.i + 0] == 0);
        assert!(chip8.memory[chip8.i + 1] == 0);
        assert!(chip8.memory[chip8.i + 2] == 5);
    }

    #[test]
    fn op_fx55() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0xF7;
        chip8.memory[0x201] = 0x55;
        for i in 0..8 {
            chip8.registers[i] = 200 + (i as u8);
        }
        chip8.i = 0x450;
        chip8.cycle(&mut rng);
        for i in 0..8 {
            assert!(chip8.memory[0x450 + i] == 200 + (i as u8));
        }
    }

    #[test]
    fn op_fx65() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0xFE;
        chip8.memory[0x201] = 0x65;
        chip8.i = 0x600;
        for i in 0..0xF {
            chip8.memory[chip8.i + i] = 33 + (4 * i as u8);
        }
        chip8.cycle(&mut rng);
        for i in 0..8 {
            assert!(chip8.registers[i] == 33 + (4 * i as u8));
        }
    }

    #[test]
    fn op_00e0() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x00;
        chip8.memory[0x201] = 0xE0;
        for i in 0..chip8.graphics.len() {
            chip8.graphics[i] = 1;
        }
        chip8.cycle(&mut rng);
        for i in 0..chip8.graphics.len() {
            assert!(chip8.graphics[i] == 0);
        }
    }

    #[test]
    fn op_dxyn() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0xD4;
        chip8.memory[0x201] = 0x55;
        chip8.registers[4] = 10;
        chip8.registers[5] = 12;
        chip8.i = 0x500;
        for i in 0..5 {
            chip8.memory[chip8.i + i] = 0xFF;
        }
        chip8.cycle(&mut rng);
        for x in 0..8 {
            for y in 0..5 {
                assert!(chip8.graphics[64 * (12 + y) + (10 + x)] == 1);
            }
        }
        assert!(chip8.registers[0xF] == 0);
        chip8.pc = 0x200;
        chip8.cycle(&mut rng);
        for x in 0..8 {
            for y in 0..5 {
                assert!(chip8.graphics[64 * (12 + y) + (10 + x)] == 0);
            }
        }
        assert!(chip8.registers[0xF] == 1);
        chip8.registers[4] = 18;
        chip8.pc = 0x200;
        chip8.cycle(&mut rng);
        for x in 0..8 {
            for y in 0..5 {
                assert!(chip8.graphics[64 * (12 + y) + (18 + x)] == 1);
            }
        }
        assert!(chip8.registers[0xF] == 0);
    }

    #[test]
    fn op_cxkk() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::chacha::ChaChaRng::new_unseeded();
        chip8.memory[0x200] = 0xC3;
        chip8.memory[0x201] = 0xFF;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[3] == 118);
        chip8.pc = 0x200;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[3] == 160);
        chip8.pc = 0x200;
        chip8.memory[0x201] = 0x00;
        chip8.cycle(&mut rng);
        assert!(chip8.registers[3] == 0);
    }

    #[test]
    fn subroutines() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x25;
        chip8.memory[0x201] = 0x00;
        chip8.memory[0x500] = 0x00;
        chip8.memory[0x501] = 0xEE;
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x500);
        assert!(chip8.sp == 1);
        assert!(chip8.stack[0] == 0x200);
        chip8.cycle(&mut rng);
        assert!(chip8.pc == 0x202);
        assert!(chip8.sp == 0);
    }

    #[test]
    #[should_panic]
    fn op_unsupported() {
        let mut chip8 = Chip8::new();
        let mut rng = rand::thread_rng();
        chip8.memory[0x200] = 0x00;
        chip8.memory[0x201] = 0x00;
        chip8.cycle(&mut rng);
    }
}
