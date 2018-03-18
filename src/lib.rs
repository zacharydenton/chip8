pub struct Chip8 {
    pub pc: usize,
    pub sp: usize,
    pub op: u16,
    pub registers: [u8; 16],
    pub memory: [u8; 4096],
    pub graphics: [u8; 64 * 32],
    pub stack: [usize; 32],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            op: 0,
            pc: 0x200,
            sp: 0,
            registers: [0; 16],
            stack: [0; 32],
            memory: [0; 4096],
            graphics: [0; 64 * 32],
        };

        // Initalize fonts at the start of system memory.
        for i in 0..FONTS.len() {
            chip8.memory[i] = FONTS[i];
        }

        chip8
    }

    pub fn cycle(&mut self) {
        // 0x5XY0: Skip next instruction if VX = VY
        // 0x9XY0: Skip next instruction if VX != VY
        // 0xEX9E: Skip next instruction if VX = hexadecimal key (LSD)
        // 0xEXA1: Skip next instruction if VX != hexadecimal key (LSD)
        // 0x6XKK: Let VX = KK
        // 0xCXKK: Let VX = Random Byte (KK = Mask)
        // 0x7XKK: Let VX = VX + KK
        // 0x8XY0: Let VX = VY
        // 0x8XY1: Let VX = VX / VY (VF changed)
        // 0x8XY2: Let VX = VX & VY (VF changed)
        // 0x8XY4: Let VX = VX + VY (VF = 0x00 if VX + VY <= 0xFF, VF = 0x01 if VX + VY > 0xFF)
        // 0x8XY5: Let VX = VX - VY (VF = 0x00 if VX < VY, VF = 0x01 if VX >= VY)
        // 0xFX07: Let VX = current timer value
        // 0xFX0A: Let VX = hexadecimal key digit (waits for any key pressed)
        // 0xFX15: Set timer = VX (0x01 = 1/60 second)
        // 0xFX18: Set tone duration = VX (0x01 = 1/60 second)
        // 0xAMMM: Let I = 0x0MMM
        // 0xFX1E: Let I = I + VX
        // 0xFX29: Let I = 5 byte display pattern for LSD of VX
        // 0xFX33: Let MI = 3 decimal digit equivalent of VX (I unchanged)
        // 0xFX55: Let MI = V0 : VX (I = I + X + 1)
        // 0xFX65: Let V0 : VX = MI (I = I + X + 1)
        // 0x00E0: Erase display (all 0s)
        // 0xDXYN: Show n byte MI pattern at VX-VY coordinates. I unchanged. MI pattern is combined
        //         with existing display via EXCLUSIVE-OR function. VF = 0x01 if a 1 in MI pattern
        //         matches 1 in existing display.
        // 0x0MMM: Do machine language at 0x0MMM (subroutine must end with 0xD4 byte)
        match self.fetch_op() {
            (0x1, a, b, c) => {
                // 0x1MMM: Go to 0x0MMM
                let mmm: usize = ((a as usize) << 8) + ((b as usize) << 4) + (c as usize);
                self.go_to(mmm);
            }
            (0xB, a, b, c) => {
                // 0xBMMM: Go to 0x0MMM + V0
                let mmm: usize = ((a as usize) << 8) + ((b as usize) << 4) + (c as usize);
                let v0: usize = self.registers[0] as usize;
                self.go_to(mmm + v0);
            }
            (0x2, a, b, c) => {
                // 0x2MMM: Do subroutine at 0x0MMM (must end with 0x00EE)
                let mmm: usize = ((a as usize) << 8) + ((b as usize) << 4) + (c as usize);
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.go_to(mmm);
            }
            (0x0, 0x0, 0xE, 0xE) => {
                // 0x00EE: Return from subroutine
                self.sp -= 1;
                let return_address: usize = self.stack[self.sp] + 2;
                self.go_to(return_address);
            }
            (0x3, x, a, b) => {
                // 0x3XKK: Skip next instruction if VX = KK
                let kk = (a << 4) + b;
                if self.registers[x as usize] == kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            (0x4, x, a, b) => {
                // 0x4XKK: Skip next instruction if VX != KK
                let kk = (a << 4) + b;
                if self.registers[x as usize] != kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            (a, b, c, d) => {
                panic!(
                    "Attempted to execute unsupported instruction: 0x{:X}{:X}{:X}{:X}",
                    a, b, c, d
                );
            }
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
        chip8.memory[0x200] = 0x13;
        chip8.memory[0x201] = 0x5F;
        chip8.memory[0x35F] = 0x12;
        chip8.memory[0x35F + 1] = 0x00;
        chip8.cycle();
        assert!(chip8.pc == 0x35F);
        chip8.cycle();
        assert!(chip8.pc == 0x200);
    }

    #[test]
    fn op_bmmm() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x200] = 0xB3;
        chip8.memory[0x201] = 0x00;
        chip8.registers[0] = 0xF0;
        chip8.cycle();
        assert!(chip8.pc == 0x300 + 0xF0);
    }

    #[test]
    fn op_3xkk() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x200] = 0x33;
        chip8.memory[0x201] = 0x42;
        chip8.registers[3] = 0x41;
        chip8.cycle();
        assert!(chip8.pc == 0x202);
        chip8.pc = 0x200;
        chip8.registers[3] = 0x42;
        chip8.cycle();
        assert!(chip8.pc == 0x204);
    }

    #[test]
    fn op_4xkk() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x200] = 0x4F;
        chip8.memory[0x201] = 0xF0;
        chip8.registers[0xF] = 0xF0;
        chip8.cycle();
        assert!(chip8.pc == 0x202);
        chip8.pc = 0x200;
        chip8.registers[0xF] = 0x42;
        chip8.cycle();
        assert!(chip8.pc == 0x204);
    }

    #[test]
    fn subroutines() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x200] = 0x25;
        chip8.memory[0x201] = 0x00;
        chip8.memory[0x500] = 0x00;
        chip8.memory[0x501] = 0xEE;
        chip8.cycle();
        assert!(chip8.pc == 0x500);
        assert!(chip8.sp == 1);
        assert!(chip8.stack[0] == 0x200);
        chip8.cycle();
        assert!(chip8.pc == 0x202);
        assert!(chip8.sp == 0);
    }

    #[test]
    #[should_panic]
    fn op_unsupported() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x200] = 0x00;
        chip8.memory[0x201] = 0x00;
        chip8.cycle();
    }
}
