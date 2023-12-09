#![allow(non_snake_case)]
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

// Constants that we use to manipulate the p (status) register https://en.wikibooks.org/wiki/6502_Assembly#Registers
const NEGATIVE: u8 = 128;
const OVERFLOW: u8 = 64;
const UNUSED: u8 = 32;
const BREAK: u8 = 16;
const DECIMAL: u8 = 8;
const INTERRUPT: u8 = 4;
const ZERO: u8 = 2;
const CARRY: u8 = 1;

const BYTE_WIDTH: i8 = 8;
const ADDR_WIDTH: i8 = 16;

// pre set memory addresses for certain operations
const RESET: u16 = 0xfffc;
const NMI: u16 = 0xfffa;
const IRQ: u16 = 0xfffe;

fn main() {
    let mut test = Mpu6502::new();
    // Reading a file byte by byte: https://users.rust-lang.org/t/reading-binary-files-a-trivial-program-not-so-trivial-for-me/56166/2
    let my_buf = BufReader::new(File::open("./firmware/template/dump/mapache64.bin").unwrap());
    let mut idx: usize = 0;
    for byte_or_error in my_buf.bytes() {
        let byte = byte_or_error.unwrap();
        test.memory[idx] = byte as u8;
        idx += 1;
    }

    loop {
        if test.memory[test.pc as usize] == (0xdb as u8) {
            break;
        }
        test.step();
    }

    let mut dump = File::create("./dump.bin").unwrap();
    for byte in test.memory {
        // Writing bytes to a file:
        // https://www.simonwenkel.com/notes/programming_languages/rust/writing-files-with-rust-wav-file-example.html
        dump.write_all(&[byte as u8]).unwrap();
    }
    println!("{}", test.pc);
    println!("{}", test.memory[test.pc as usize]);
    println!("{}", test.ImmediateByte());
    print!("HI")
}

pub struct Mpu6502 {
    pc: i32,
    // acc is set as an i32 even though it really should be i8
    // This makes following along with the original python code easier
    // and makes checking for overflow easier.
    acc: i32,
    p: u8,
    // The sp is always added onto the spBase to determine
    // where in the stack we are storing the next stack
    // value, sp decreases each time
    sp: i32,
    x: i32,
    y: i32,

    memory: [u8; 0xffff + 1],

    spBase: i32,
    start_pc: i32,

    byteMask: i32,
    addrMask: i32,
    addrHighMask: i32,

    excycles: i32,
    addcycles: bool,
    processorCycles: i32,
    instructions: HashMap<u8, fn(&mut Mpu6502)>,
}

impl Mpu6502 {
    fn new() -> Self {
        let byteMask: i32 = (1 << BYTE_WIDTH) - 1;
        let start_pc = 0x5038;
        let instructions = initializeInstructions();
        Mpu6502 {
            pc: start_pc,
            sp: byteMask,
            acc: 0,
            p: 0 | UNUSED | BREAK,
            x: 0,
            y: 0,
            byteMask: byteMask,
            addrMask: ((1 << ADDR_WIDTH) - 1),
            addrHighMask: (byteMask << BYTE_WIDTH),
            spBase: 1 << BYTE_WIDTH,
            excycles: 0,
            addcycles: false,
            processorCycles: 0,
            memory: [0; 0xffff + 1],
            start_pc: start_pc,
            instructions: instructions,
        }
    }

    pub fn step(&mut self) {
        let instructCode = self.memory[self.pc as usize] as u8;
        self.pc = (self.pc + 1) & self.addrMask;

        let getResult = self.instructions.get(&instructCode);
        println!("{:#04x}", instructCode);
        if (getResult.is_none()) {
            println!("PC: {}", self.pc);
            println!("instructCode: {:#04x}", instructCode);
            self.pc = (self.pc + 1) & self.addrMask;
            return;
        }
        let instruction = self.instructions.get(&instructCode).unwrap();

        instruction(self);
        self.pc &= self.addrMask;
    }

    pub fn stPush(&mut self, z: i32) {
        self.memory[(self.sp + self.spBase) as usize] = z as u8;
        self.sp -= 1;
        self.sp &= self.byteMask;
    }

    pub fn stPop(&mut self) -> i32 {
        self.sp += 1;
        self.sp &= self.byteMask;
        return self.ByteAt(self.sp + self.spBase);
    }

    pub fn stPushWord(&mut self, z: i32) {
        self.stPush((z >> BYTE_WIDTH) & self.byteMask);
        self.stPush(z & self.byteMask)
    }

    pub fn stPopWord(&mut self) -> i32 {
        let mut z = self.stPop();
        z += self.stPop() << BYTE_WIDTH;
        return z;
    }
    
    
    pub fn ByteAt(&mut self, addr: i32) -> i32 {
        let val = self.memory[addr as usize];
        val as i32
    }

    pub fn WordAt(&mut self, addr: i32) -> i32 {
        self.ByteAt(addr) + (self.ByteAt(addr + 1) << BYTE_WIDTH)
    }

    pub fn WrapAt(&mut self, addr: i32) -> i32 {
        let wrapped_addr = (addr & self.addrHighMask) + ((addr + 1) & self.byteMask);
        self.ByteAt(addr) + (self.ByteAt(wrapped_addr) << BYTE_WIDTH)
    }

    pub fn ProgramCounter(&mut self) -> i32 {
        self.pc
    }

    pub fn ImmediateByte(&mut self) -> i32 {
        return self.ByteAt(self.pc);
    }

    pub fn FlagsNZ(&mut self, value: i32) {
        self.p &= !(ZERO | NEGATIVE);
        if value == 0 {
            self.p = self.p | ZERO;
        } else {
            self.p = self.p | (value & NEGATIVE as i32) as u8;
        }
    }

    pub fn reset(&mut self) {
        self.pc = self.start_pc;
        self.sp = self.byteMask;
        self.acc = 0;
        self.x = 0;
        self.y = 0;
        self.p = BREAK | UNUSED;
        self.processorCycles = 0;
    }

    pub fn opSTZ(&mut self, x: i32) {
        self.memory[x as usize] = 0x00
    }

    pub fn opASL(&mut self, x: Option<i32>) {
        let mut tbyte = self.acc;
        self.p &= !(CARRY | NEGATIVE | ZERO);
        let mut addr = 0;

        if (!x.is_none()) {
            addr = x.unwrap();
            tbyte = self.ByteAt(addr);
        }

        if tbyte as u8 & NEGATIVE != 0 {
            self.p |= CARRY
        }
        tbyte = (tbyte << 1) & self.byteMask;
        if tbyte != 0 {
            self.p |= NEGATIVE & tbyte as u8;
        } else {
            self.p |= ZERO;
        }

        if (!x.is_none()) {
            self.memory[addr as usize] = tbyte as u8;
        }

        self.acc = tbyte;
        // println!("{:#b}", NEGATIVE);
        // println!("{:#b}", self.p);
    }

    pub fn opROL(&mut self, x: Option<i32>) {
        let mut tbyte = self.acc;
        let mut addr: i32 = 0;

        if !x.is_none() {
            addr = x.unwrap();
            tbyte = self.ByteAt(addr);
        }
        if (self.p & CARRY) != 0 {
            if (tbyte & (NEGATIVE as i32)) != 0 {
                /*pass*/
            } else {
                self.p = self.p | CARRY;
            }
            tbyte = (tbyte << 1) | 1;
        } else {
            if (tbyte & (NEGATIVE as i32)) != 0 {
                self.p |= CARRY;
            }
            tbyte = tbyte << 1;
        }
        tbyte &= self.byteMask;
        self.FlagsNZ(tbyte);
        if x.is_none() {
            self.acc = tbyte;
        } else {
            self.memory[addr as usize] = tbyte as u8;
        }
    }

    pub fn ZeroPageIndirectAddr(&mut self) -> i32{
        let byte_at = self.ByteAt(self.pc);
        return self.WordAt(255 & (byte_at));
    }
    pub fn AbsoluteYAddr(&mut self) -> i32 {
        if self.addcycles {
            let a1 = self.WordAt(self.pc);
            let a2 = (a1 + self.y) & self.addrMask;
            if (a1 & self.addrHighMask) != (a2 & self.addrHighMask) {
                self.excycles += 1;
            }
            return a2;
        }
        return (self.WordAt(self.pc) + self.y) & self.addrMask;
    }

    pub fn BranchRelAddr(&mut self) {
        self.excycles += 1;
        let mut addr = self.ImmediateByte();
        self.pc += 1;

        if (addr & (NEGATIVE as i32)) == 0 {
            addr = self.pc + addr;
        } else {
            addr = self.pc - (addr ^ self.byteMask) - 1;
        }

        if (self.pc & self.addrHighMask) != (addr & self.addrHighMask) {
            self.excycles += 1;
        }

        self.pc = addr & self.addrMask;
    }

    //__________________________________________________________________________________operations

    pub fn opORA(&mut self, x: i32) {
        self.acc = self.acc | self.ByteAt(x);
        self.FlagsNZ(self.acc);
    }

    pub fn opAND(&mut self, x: i32) {
        self.acc = self.acc & self.ByteAt(x);
        self.FlagsNZ(self.acc);
    }

    pub fn opEOR(&mut self, x: i32) {
        self.acc = self.acc ^ self.ByteAt(x);
        self.FlagsNZ(self.acc);
    }

    pub fn opBCL(&mut self, x: i32) {
        if ((self.p as i32) & x) != 0 {
            self.pc += 1;
        } else {
            self.BranchRelAddr();
        }
    }

    pub fn opBST(&mut self, x: i32) {
        if ((self.p as i32) & x) != 0 {
            self.BranchRelAddr();
        } else {
            self.pc += 1;
        }
    }

    pub fn opCLR(&mut self, x: i32) {
        self.p = self.p & !(x as u8);
    }

    pub fn opSET(&mut self, x: i32) {
        self.p = self.p | (x as u8);
    }

    pub fn opSTA(&mut self, x: i32) {
        self.memory[x as usize] = self.acc as u8;
    }

    pub fn opSTY(&mut self, x: i32) {
        self.memory[x as usize] = self.y as u8;
    }

    pub fn opBIT(&mut self, x: i32) {
        let tbyte = self.ByteAt(x);
        self.p = self.p & !(ZERO | NEGATIVE | OVERFLOW);
        if (self.acc & tbyte) == 0 {
            self.p = self.p | ZERO;
        }
        self.p = self.p | ((tbyte & ((NEGATIVE | OVERFLOW) as i32)) as u8);
    }

    pub fn opCMPR(&mut self, addr: i32, register_value: i32) {
        let tbyte = self.ByteAt(addr);
        self.p = self.p & !(CARRY | ZERO | NEGATIVE);
        if register_value == tbyte {
            self.p = self.p | CARRY | ZERO;
        } else if register_value > tbyte {
            self.p = self.p | CARRY;
        }
        self.p = self.p | (((register_value - tbyte) & NEGATIVE as i32) as u8);
    }

    pub fn opLSR(&mut self, x: Option<i32>) {
        let mut tbyte: i32;
        let mut addr: i32 = 0;
        if x.is_none() {
            tbyte = self.acc;
        } else {
            addr = x.unwrap();
            tbyte = self.ByteAt(addr);
        }

        self.p = self.p & !(CARRY | NEGATIVE | ZERO);
        self.p = self.p | ((tbyte & 1) as u8);

        tbyte = tbyte >> 1;
        if tbyte == 0 {
            self.p = self.p | ZERO;
        }

        if x.is_none() {
            self.acc = tbyte;
        } else {
            self.memory[addr as usize] = tbyte as u8;
        }
    }

    pub fn ZeroPageAddr(&mut self) -> i32 {
        self.ByteAt(self.pc)
    }
    pub fn ZeroPageXAddr(&mut self) -> i32 {
        self.byteMask & (self.x + self.ByteAt(self.pc))
    }

    pub fn ZeroPageYAddr(&mut self) -> i32 {
        self.byteMask & (self.y + self.ByteAt(self.pc))
    }

    pub fn IndirectXAddr(&mut self) -> i32 {
        let byte_at = self.ByteAt(self.pc);
        self.WrapAt(self.byteMask & (byte_at + self.x))
    }

    pub fn IndirectYAddr(&mut self) -> i32 {
        let byte_at: i32 = self.ByteAt(self.pc);
        if self.addcycles {
            let a1 = self.WrapAt(byte_at);
            let a2 = (a1 + self.y) & self.addrMask;
            if (a1 & self.addrHighMask) != (a2 & self.addrHighMask) {
                self.excycles += 1
            }
            return a2;
        } else {
            (self.WrapAt(byte_at) + self.y) & self.addrMask
        }
    }

    pub fn AbsoluteAddr(&mut self) -> i32 {
        self.WordAt(self.pc)
    }

    pub fn AbsoluteXAddr(&mut self) -> i32 {
        if self.addcycles {
            let a1 = self.WordAt(self.pc);
            let a2 = (a1 + self.x) & self.addrMask;
            if a1 & self.addrHighMask != a2 & self.addrHighMask {
                self.excycles += 1
            }
            return a2;
        } else {
            return (self.WordAt(self.pc) + self.x) & self.addrMask;
        }
    }
    // NEW OPS 11/30
    //TEMP FLAGSNZ

    pub fn opSTX(&mut self, y: i32) {
        self.memory[y as usize] = self.x as u8;
    }

    pub fn opLDA(&mut self, x: i32) {
        self.acc = self.ByteAt(x);
        self.FlagsNZ(self.acc);
    }
    pub fn opLDY(&mut self, x: i32) {
        self.y = self.ByteAt(x);
        self.FlagsNZ(self.y);
    }
    pub fn opLDX(&mut self, y: i32) {
        self.x = self.ByteAt(y);
        self.FlagsNZ(self.x);
    }
    pub fn opDECR(&mut self, x: Option<i32>) {
        let mut tbyte: i32;
        let mut addr: i32 = 0; // Needs to be initialized so setting addr to 0
        if x.is_none() {
            tbyte = self.acc;
        } else {
            addr = x.unwrap();
            tbyte = self.ByteAt(addr);
        }
        self.p &= !(ZERO | NEGATIVE);
        tbyte = (tbyte - 1) & self.byteMask;
        if tbyte != 0 {
            self.p |= tbyte as u8 & NEGATIVE;
        } else {
            self.p |= ZERO;
        }

        if x.is_none() {
            self.acc = tbyte;
        } else {
            self.memory[addr as usize] = tbyte as u8;
        }
    }
    pub fn opINCR(&mut self, x: Option<i32>) {
        let mut tbyte: i32;
        let mut addr: i32 = 0; // Needs to be initialized so setting addr to 0
        if x.is_none() {
            tbyte = self.acc;
        } else {
            addr = x.unwrap();
            tbyte = self.ByteAt(addr);
        }
        self.p &= !(ZERO | NEGATIVE);
        tbyte = (tbyte + 1) & self.byteMask;
        if tbyte != 0 {
            self.p |= tbyte as u8 & NEGATIVE;
        } else {
            self.p |= ZERO;
        }
        if x.is_none() {
            self.acc = tbyte;
        } else {
            self.memory[addr as usize] = tbyte as u8;
        }
    }
    pub fn opADC(&mut self, x: i32) {
        let mut data = self.ByteAt(x);

        if (self.p & DECIMAL) != 0 {
            let mut halfcarry = 0;
            let mut decimalcarry = 0;
            let mut adjust0 = 0;
            let mut adjust1 = 0;
            let mut nibble0 = (data & 0xf) + (self.acc & 0xf) + (self.p as i32 & CARRY as i32);
            if nibble0 > 9 {
                adjust0 = 6;
                halfcarry = 1;
            }
            let mut nibble1 = ((data >> 4) & 0xf) + ((self.acc >> 4) & 0xf) + halfcarry;
            if nibble1 > 9 {
                adjust1 = 6;
                decimalcarry = 1;
            }
            //the ALU outputs are not decimally adjusted
            nibble0 = nibble0 & 0xf;
            nibble1 = nibble1 & 0xf;
            let aluresult = (nibble1 << 4) + nibble0;

            // the final A contents will be decimally adjusted
            nibble0 = (nibble0 + adjust0) & 0xf;
            nibble1 = (nibble1 + adjust1) & 0xf;

            self.p &= !(CARRY | OVERFLOW | NEGATIVE | ZERO);

            if aluresult == 0 {
                self.p |= ZERO;
            } else {
                self.p |= aluresult as u8 & NEGATIVE;
            }
            if decimalcarry == 1 {
                self.p |= CARRY;
            }
            if ((!(self.acc ^ data) & (self.acc ^ aluresult)) & NEGATIVE as i32) != 0 {
                self.p |= OVERFLOW;
            }
            self.acc = (nibble1 << 4) + nibble0;
        } else {
            let mut tmp: i32 = 0;
            if (self.p & CARRY) != 0 {
                tmp = 1;
            } else {
                tmp = 0;
            }
            let result = data + self.acc + tmp;
            self.p &= !(CARRY | OVERFLOW | NEGATIVE | ZERO);
            if (!(self.acc ^ data) & (self.acc ^ result)) & NEGATIVE as i32 != 0 {
                self.p |= OVERFLOW;
            }
            data = result;

            if data > self.byteMask {
                self.p |= CARRY;
                data &= self.byteMask;
            }
            if data == 0 {
                self.p |= ZERO;
            } else {
                self.p |= data as u8 & NEGATIVE;
            }
            self.acc = data;
        }
    }
    pub fn opSBC(&mut self, x: i32) {
        let data = self.ByteAt(x);

        if self.p & DECIMAL != 0 {
            let mut halfcarry = 1;
            let mut decimalcarry = 0;
            let mut adjust0 = 0;
            let mut adjust1 = 0;

            let mut nibble0 = (self.acc & 0xf) + (!data & 0xf) + (self.p as i32 & CARRY as i32);
            if nibble0 <= 0xf {
                halfcarry = 0;
                adjust0 = 10;
            }
            let mut nibble1 = ((self.acc >> 4) & 0xf) + ((!data >> 4) & 0xf) + halfcarry;
            if nibble1 <= 0xf {
                adjust1 = 10 << 4;
            }
            let mut aluresult = self.acc + (!data & self.byteMask) + (self.p as i32 + CARRY as i32);
            if aluresult > self.byteMask {
                decimalcarry = 1;
            }
            aluresult &= self.byteMask;
            nibble0 = (aluresult + adjust0) & 0xf;
            nibble1 = ((aluresult + adjust1) >> 4) & 0xf;

            self.p &= !(CARRY | ZERO | NEGATIVE | OVERFLOW);
            if aluresult == 0 {
                self.p |= ZERO;
            } else {
                self.p |= aluresult as u8 & NEGATIVE;
            }
            if decimalcarry == 1 {
                self.p |= CARRY;
            }
            if ((self.acc ^ data) & (self.acc ^ aluresult)) & NEGATIVE as i32 != 0 {
                self.p |= OVERFLOW;
            }
            self.acc = (nibble1 << 4) + nibble0;
        } else {
            let result = self.acc + (!data & self.byteMask) + (self.p as i32 & CARRY as i32);
            self.p &= !(CARRY | ZERO | OVERFLOW | NEGATIVE);

            if ((self.acc ^ data) & (self.acc ^ result) & NEGATIVE as i32) != 0 {
                self.p |= OVERFLOW;
            }
            let data = result & self.byteMask;
            if data == 0 {
                self.p |= ZERO;
            }
            if result > self.byteMask {
                self.p |= CARRY;
            }
            self.p |= data as u8 & NEGATIVE;
            self.acc = data;
        }
    }
}

fn initializeInstructions() -> HashMap<u8, fn(&mut Mpu6502)> {
    let mut instructions = HashMap::<u8, fn(&mut Mpu6502)>::new();

    // @instruction(name="BRK", mode="imp", cycles=7)
    instructions.insert(0x00, |self2| {
        let pc = (self2.pc) & self2.addrMask;
        self2.stPushWord(pc);

        self2.p |= BREAK;
        self2.stPush((self2.p | BREAK | UNUSED) as i32);

        self2.p |= INTERRUPT;
        self2.pc = self2.WordAt(IRQ as i32);
    });
    // ADC, inx
    instructions.insert(0x61, |self2| {
        let xAddr = self2.IndirectXAddr();
        self2.opADC(xAddr);
        self2.pc += 1;
    });

    //     @instruction(name="BPL", mode="rel", cycles=2, extracycles=2)
    instructions.insert(0x10, |self2| {
        self2.opBCL(NEGATIVE.into());
    });
    //     @instruction(name="CLC", mode="imp", cycles=2)
    instructions.insert(0x18, |self2| {
        self2.opCLR(CARRY.into());
    });
    //     @instruction(name="JSR", mode="abs", cycles=6)
    instructions.insert(0x20, |self2| {
        self2.stPushWord((self2.pc + 1) & self2.addrMask);
        self2.pc = self2.WordAt(self2.pc);
    });
    //     @instruction(name="SEC", mode="imp", cycles=2)
    instructions.insert(0x38, |self2| {
        self2.opSET(CARRY.into());
    });
    //     @instruction(name="EOR", mode="inx", cycles=6)
    instructions.insert(0x41, |self2| {
        let xAddr = self2.IndirectXAddr();
        self2.opEOR(xAddr);
        self2.pc += 1;
    });
    //     @instruction(name="PHA", mode="imp", cycles=3)
    instructions.insert(0x48, |self2| {
        self2.stPush(self2.acc);
    });
    //     @instruction(name="JMP", mode="abs", cycles=3)
    instructions.insert(0x4c, |self2| {
        self2.pc = self2.WordAt(self2.pc);
    });
    //     @instruction(name="BVC", mode="rel", cycles=2, extracycles=2)
    instructions.insert(0x50, |self2| {
        self2.opBCL(OVERFLOW.into());
    });
    //     @instruction(name="EOR", mode="zpx", cycles=4)
    instructions.insert(0x55, |self2| {
        let zpXAddr = self2.ZeroPageXAddr();
        self2.opEOR(zpXAddr);
        self2.pc += 1;
    });
    //     @instruction(name="EOR", mode="aby", cycles=4, extracycles=1)
    instructions.insert(0x59, |self2| {
        let absXAddr = self2.AbsoluteYAddr();
        self2.opEOR(absXAddr);
        self2.pc += 2;
    });
    //     @instruction(name="RTS", mode="imp", cycles=6)
    instructions.insert(0x60, |self2| {
        self2.pc = self2.stPopWord();
        self2.pc += 1;
    });
    //     @instruction(name="ADC", mode="zpg", cycles=3)
    instructions.insert(0x65, |self2| {
        let addr = self2.ZeroPageAddr();
        self2.opADC(addr);
        self2.pc += 1;
    });
    //     @instruction(name="PLA", mode="imp", cycles=4)
    instructions.insert(0x68, |self2| {
        self2.acc = self2.stPop();
        self2.FlagsNZ(self2.acc);
    });
    //     @instruction(name="JMP", mode="ind", cycles=5)
    instructions.insert(0x6c, |self2| {
        let ta = self2.WordAt(self2.pc);
        self2.pc = self2.WrapAt(ta);
    });
    //     @instruction(name="ADC", mode="iny", cycles=5, extracycles=1)
    instructions.insert(0x71, |self2| {
        let addr = self2.IndirectYAddr();
        self2.opADC(addr);
        self2.pc += 1;
    });
    //     @instruction(name="ADC", mode="aby", cycles=4, extracycles=1)
    instructions.insert(0x79, |self2| {
        let addr = self2.AbsoluteYAddr();
        self2.opADC(addr);
        self2.pc += 2;
    });
    //     @instruction(name="STA", mode="inx", cycles=6)
    instructions.insert(0x81, |self2| {
        let addr = self2.IndirectXAddr();
        self2.opSTA(addr);
        self2.pc += 1;
    });
    //     @instruction(name="STA", mode="zpg", cycles=3)
    instructions.insert(0x85, |self2| {
        let addr = self2.ZeroPageAddr();
        self2.opSTA(addr);
        self2.pc += 1;
    });
    //     @instruction(name="DEY", mode="imp", cycles=2)
    instructions.insert(0x88, |self2| {
        self2.y -= 1;
        self2.y &= self2.byteMask;
        self2.FlagsNZ(self2.y);
    });
    //     @instruction(name="STA", mode="abs", cycles=4)
    instructions.insert(0x8d, |self2| {
        let addr = self2.AbsoluteAddr();
        self2.opSTA(addr);
        self2.pc += 2;
    });
    //     @instruction(name="BCC", mode="rel", cycles=2, extracycles=2)
    instructions.insert(0x90, |self2| {
        self2.opBCL(CARRY.into());
    });
    //     @instruction(name="STA", mode="zpx", cycles=4)
    instructions.insert(0x95, |self2| {
        let addr = self2.ZeroPageXAddr();
        self2.opSTA(addr);
        self2.pc += 1;
    });
    //     @instruction(name="STA", mode="aby", cycles=5)
    instructions.insert(0x99, |self2| {
        let addr = self2.AbsoluteYAddr();
        self2.opSTA(addr);
        self2.pc += 2;
    });
    //     @instruction(name="STA", mode="abx", cycles=5)
    instructions.insert(0x9d, |self2| {
        let addr = self2.AbsoluteXAddr();
        self2.opSTA(addr);
        self2.pc += 2;
    });
    //     @instruction(name="LDA", mode="inx", cycles=6)
    instructions.insert(0xa1, |self2| {
        let addr = self2.IndirectXAddr();
        self2.opLDA(addr);
        self2.pc += 1;
    });
    //     @instruction(name="LDY", mode="zpg", cycles=3)
    instructions.insert(0xa4, |self2| {
        let addr = self2.ZeroPageAddr();
        self2.opLDY(addr);
        self2.pc += 1;
    });
    //     @instruction(name="LDX", mode="zpg", cycles=3)
    instructions.insert(0xa6, |self2| {
        let addr = self2.ZeroPageAddr();
        self2.opLDX(addr);
        self2.pc += 1;
    });
    //     @instruction(name="LDA", mode="imm", cycles=2)
    instructions.insert(0xa9, |self2| {
        let addr = self2.ProgramCounter();
        self2.opLDA(addr);
        self2.pc += 1;
    });
    //     @instruction(name="LDY", mode="abs", cycles=4)
    instructions.insert(0xac, |self2| {
        let addr = self2.AbsoluteAddr();
        self2.opLDY(addr);
        self2.pc += 2;
    });
    //     @instruction(name="LDX", mode="abs", cycles=4)
    instructions.insert(0xae, |self2| {
        let addr = self2.AbsoluteAddr();
        self2.opLDX(addr);
        self2.pc += 2;
    });
    //     @instruction(name="LDA", mode="iny", cycles=5, extracycles=1)
    instructions.insert(0xb1, |self2| {
        let addr = self2.IndirectYAddr();
        self2.opLDA(addr);
        self2.pc += 1;
    });
    //     @instruction(name="LDA", mode="zpx", cycles=4)
    instructions.insert(0xb5, |self2| {
        let addr = self2.ZeroPageXAddr();
        self2.opLDA(addr);
        self2.pc += 1;
    });
    //     @instruction(name="LDA", mode="aby", cycles=4, extracycles=1)
    instructions.insert(0xb9, |self2| {
        let addr = self2.AbsoluteYAddr();
        self2.opLDA(addr);
        self2.pc += 2;
    });
    //     @instruction(name="LDY", mode="abx", cycles=4, extracycles=1)
    instructions.insert(0xbc, |self2| {
        let addr = self2.AbsoluteXAddr();
        self2.opLDY(addr);
        self2.pc += 2;
    });
    //     @instruction(name="LDX", mode="aby", cycles=4, extracycles=1)
    instructions.insert(0xbe, |self2| {
        let addr = self2.AbsoluteYAddr();
        self2.opLDX(addr);
        self2.pc += 2;
    });
    //     @instruction(name="CMP", mode="inx", cycles=6)
    instructions.insert(0xc1, |self2| {
        let addr = self2.IndirectXAddr();
        self2.opCMPR(addr, self2.acc);
        self2.pc += 1;
    });
    //     @instruction(name="CMP", mode="zpg", cycles=3)
    instructions.insert(0xc5, |self2| {
        let addr = self2.ZeroPageAddr();
        self2.opCMPR(addr, self2.acc);
        self2.pc += 1;
    });
    //     @instruction(name="INY", mode="imp", cycles=2)
    instructions.insert(0xc8, |self2| {
        self2.y += 1;
        self2.y &= self2.byteMask;
        self2.FlagsNZ(self2.y);
    });
    //     @instruction(name="DEX", mode="imp", cycles=2)
    instructions.insert(0xca, |self2| {
        self2.x -= 1;
        self2.x &= self2.byteMask;
        self2.FlagsNZ(self2.x);
    });
    //     @instruction(name="CMP", mode="abs", cycles=4)
    instructions.insert(0xcd, |self2| {
        let addr = self2.AbsoluteAddr();
        self2.opCMPR(addr, self2.acc);
        self2.pc += 2;
    });
    //     @instruction(name="BNE", mode="rel", cycles=2, extracycles=2)
    instructions.insert(0xd0, |self2| {
        self2.opBCL(ZERO.into());
    });
    //     @instruction(name="CMP", mode="zpx", cycles=4)
    instructions.insert(0xd5, |self2| {
        let addr = self2.ZeroPageXAddr();
        self2.opCMPR(addr, self2.acc);
        self2.pc += 1;
    });
    //     @instruction(name="CMP", mode="aby", cycles=4, extracycles=1)
    instructions.insert(0xd9, |self2| {
        let addr = self2.AbsoluteYAddr();
        self2.opCMPR(addr, self2.acc);
        self2.pc += 2;
    });
    //     @instruction(name="CPX", mode="imm", cycles=2)
    instructions.insert(0xe0, |self2| {
        let addr = self2.ProgramCounter();
        self2.opCMPR(addr, self2.x);
        self2.pc += 1;
    });
    //     @instruction(name="CPX", mode="zpg", cycles=3)
    instructions.insert(0xe4, |self2| {
        let addr = self2.ZeroPageAddr();
        self2.opCMPR(addr, self2.x);
        self2.pc += 1;
    });
    //     @instruction(name="INX", mode="imp", cycles=2)
    instructions.insert(0xe8, |self2| {
        self2.x += 1;
        self2.x &= self2.byteMask;
        self2.FlagsNZ(self2.x);
    });
    //     @instruction(name="CPX", mode="abs", cycles=4)
    instructions.insert(0xec, |self2| {
        let addr = self2.AbsoluteAddr();
        self2.opCMPR(addr, self2.x);
        self2.pc += 2;
    });
    //     @instruction(name="SBC", mode="iny", cycles=5, extracycles=1)
    instructions.insert(0xf1, |self2| {
        let addr = self2.IndirectYAddr();
        self2.opSBC(addr);
        self2.pc += 1;
    });
    //     @instruction(name="SBC", mode="aby", cycles=4, extracycles=1)
    instructions.insert(0xf9, |self2| {
        let addr = self2.AbsoluteYAddr();
        self2.opSBC(addr);
        self2.pc += 2;
    });

    // @instruction(name="STZ", mode="abs", cycles=4)
    instructions.insert(0x9c, |self2| {
        let addr = self2.AbsoluteAddr();
        self2.opSTZ(addr);
        self2.pc += 2
    });

    // @instruction(name="SEI", mode="imp", cycles=2)
    instructions.insert(0x78, |self2| self2.opSET(INTERRUPT as i32));

    //     @instruction(name="ASL", mode="zpg", cycles=5)
    instructions.insert(0x06, |self2| {
        let zero_page_addr = self2.ZeroPageAddr();

        self2.opASL(Some(zero_page_addr));
        self2.pc += 1;
    });
    //     @instruction(name="ASL", mode="acc", cycles=2)
    instructions.insert(0x0a, |self2| {
        self2.opASL(None);
    });
    //     @instruction(name="ASL", mode="abs", cycles=6)
    instructions.insert(0x0e, |self2| {
        let addr = self2.AbsoluteAddr();
        self2.opASL(Some(addr));
        self2.pc += 2;
    });

    //     @instruction(name="ASL", mode="zpx", cycles=6)
    instructions.insert(0x16, |self2| {
        let addr = self2.ZeroPageXAddr();
        self2.opASL(Some(addr));
        self2.pc += 1;
    });
    //     @instruction(name="ASL", mode="abx", cycles=7)
    instructions.insert(0x1e, |self2| {
        let addr = self2.AbsoluteXAddr();
        self2.opASL(Some(addr));
        self2.pc += 2;
    });
    //     @instruction(name="ROL", mode="zpg", cycles=5)
    instructions.insert(0x26, |self2| {
        let addr = self2.ZeroPageAddr();
        self2.opROL(Some(addr));
        self2.pc += 1;
    });
    //     @instruction(name="ROL", mode="acc", cycles=2)
    instructions.insert(0x2a, |self2| {
        self2.opROL(None);
    });
    //     @instruction(name="ROL", mode="abs", cycles=6)
    instructions.insert(0x2e, |self2| {
        let addr = self2.AbsoluteAddr();
        self2.opROL(Some(addr));
        self2.pc += 2;
    });
    //     @instruction(name="ROL", mode="zpx", cycles=6)
    instructions.insert(0x36, |self2| {
        let addr = self2.ZeroPageXAddr();
        self2.opROL(Some(addr));
        self2.pc += 1;
    });

    //     @instruction(name="ROL", mode="abx", cycles=7)
    instructions.insert(0x3e, |self2| {
        let x = self2.AbsoluteXAddr();
        self2.opROL(Some(x));
        self2.pc += 2;
    });
    //     @instruction(name="EOR", mode="zpg", cycles=3)
    instructions.insert(0x45, |self2| {
        let x = self2.ZeroPageAddr();
        self2.opEOR(x);
        self2.pc += 1;
    });

    //     @instruction(name="EOR", mode="imm", cycles=2)
    instructions.insert(0x49, |self2| {
        let x = self2.ProgramCounter();
        self2.opEOR(x);
        self2.pc += 1;

    });

        //     @instruction(name="EOR", mode="abs", cycles=4)
    instructions.insert(0x4d, |self2| {
        let x = self2.AbsoluteAddr();
        self2.opEOR(x);
        self2.pc += 2;

    });

        //     @instruction(name="EOR", mode="iny", cycles=5, extracycles=1)
    instructions.insert(0x51, |self2| {
        let x = self2.IndirectYAddr();
        self2.opEOR(x);
        self2.pc += 1;

    });

        //     @instruction(name="EOR", mode="abx", cycles=4, extracycles=1)
    instructions.insert(0x5d, |self2| {
        let x = self2.AbsoluteXAddr();
        self2.opEOR(x);
        self2.pc += 2;

    });

        //     @instruction(name="ADC", mode="inx", cycles=6)
    instructions.insert(0x61, |self2| {
        let x = self2.IndirectXAddr();
        self2.opADC(x);
        self2.pc += 1;

    });
        //     @instruction(name="ADC", mode="imm", cycles=2)
    instructions.insert(0x69, |self2| {
        let x = self2.ProgramCounter();
        self2.opADC(x);
        self2.pc += 1;

    });
        //     @instruction(name="ADC", mode="abs", cycles=4)
    instructions.insert(0x6d, |self2| {
        let x = self2.AbsoluteAddr();
        self2.opADC(x);
        self2.pc += 2;

    });

        //     @instruction(name="ADC", mode="zpx", cycles=4)
    instructions.insert(0x75, |self2| {
        let x = self2.ZeroPageXAddr();
        self2.opADC(x);
        self2.pc += 1;

    });
        //     @instruction(name="ADC", mode="abx", cycles=4, extracycles=1)
    instructions.insert(0x7d, |self2| {
        let x = self2.AbsoluteXAddr();
        self2.opADC(x);
        self2.pc += 2;

    });

        //     @instruction(name="TXA", mode="imp", cycles=2)
    instructions.insert(0x8a, |self2| {
        self2.acc = self2.x;
        self2.FlagsNZ(self2.acc);

    });

        //     @instruction(name="STA", mode="iny", cycles=6)
    instructions.insert(0x91, |self2| {
        let x = self2.IndirectYAddr();
        self2.opSTA(x);
        self2.pc += 1;

    });

        //     @instruction(name="LDY", mode="imm", cycles=2)
    instructions.insert(0xa0, |self2| {
        let x = self2.ProgramCounter();
        self2.opLDY(x);
        self2.pc += 1;

    });

        //     @instruction(name="LDX", mode="imm", cycles=2)
    instructions.insert(0xa2, |self2| {
        let y = self2.ProgramCounter();
        self2.opLDX(y);
        self2.pc += 1;

    });

        //     @instruction(name="LDA", mode="zpg", cycles=3)
    instructions.insert(0xa5, |self2| {
        let x = self2.ZeroPageAddr();
        self2.opLDA(x);
        self2.pc += 1;

    });

        //     @instruction(name="TAX", mode="imp", cycles=2)
    instructions.insert(0xaa, |self2| {
        self2.x = self2.acc;
        self2.FlagsNZ(self2.x);

    });

        //     @instruction(name="LDA", mode="abs", cycles=4)
    instructions.insert(0xad, |self2| {
        let x = self2.AbsoluteAddr();
        self2.opLDA(x);
        self2.pc += 2;

    });

        //     @instruction(name="BCS", mode="rel", cycles=2, extracycles=2)
    instructions.insert(0xb0, |self2| {
        self2.opBST(CARRY as i32);

    });

        //     @instruction(name="LDY", mode="zpx", cycles=4)
    instructions.insert(0xb4, |self2| {
        let x = self2.ZeroPageXAddr();
        self2.opLDY(x);
        self2.pc += 1;

    });

        //     @instruction(name="LDX", mode="zpy", cycles=4)
    instructions.insert(0xb6, |self2| {
        let y = self2.ZeroPageYAddr();
        self2.opLDX(y);
        self2.pc += 1;

    });

        //     @instruction(name="LDA", mode="abx", cycles=4, extracycles=1)
    instructions.insert(0xbd, |self2| {
        let x = self2.AbsoluteXAddr();
        self2.opLDA(x);
        self2.pc += 2;

    });

        //     @instruction(name="CMP", mode="imm", cycles=2)
    instructions.insert(0xc9, |self2| {
        let addr = self2.ProgramCounter();
        self2.opCMPR(addr, self2.acc);
        self2.pc += 1;

    });

        //     @instruction(name="CMP", mode="iny", cycles=5, extracycles=1)
    instructions.insert(0xd1, |self2| {
            let addr = self2.IndirectYAddr();
            self2.opCMPR(addr, self2.acc);
            self2.pc += 1;

    });

        //     @instruction(name="CMP", mode="abx", cycles=4, extracycles=1)
    instructions.insert(0xdd, |self2| {
        let addr = self2.AbsoluteXAddr();
        self2.opCMPR(addr, self2.acc);
        self2.pc += 2;

    });

        //     @instruction(name="SBC", mode="inx", cycles=6)
    instructions.insert(0xe1, |self2| {
        let x = self2.IndirectXAddr();
        self2.opSBC(x);
        self2.pc += 1;

    });

        //     @instruction(name="SBC", mode="zpg", cycles=3)
    instructions.insert(0xe5, |self2| {
        let x = self2.ZeroPageAddr();
        self2.opSBC(x);
        self2.pc += 1;

    });

        //     @instruction(name="SBC", mode="imm", cycles=2)
    instructions.insert(0xe9, |self2| {
        let x = self2.ProgramCounter();
        self2.opSBC(x);
        self2.pc += 1;

    });

        //     @instruction(name="SBC", mode="abs", cycles=4)
    instructions.insert(0xed, |self2| {
        let x = self2.AbsoluteAddr();
        self2.opSBC(x);
        self2.pc += 2;

    });

        //     @instruction(name="SBC", mode="zpx", cycles=4)
    instructions.insert(0xf5, |self2| {
        let x = self2.ZeroPageXAddr();
        self2.opSBC(x);
        self2.pc += 1;

    });

        //     @instruction(name="SBC", mode="abx", cycles=4, extracycles=1)
    instructions.insert(0xfd, |self2| {
        let x = self2.AbsoluteXAddr();
        self2.opSBC(x);
        self2.pc += 2;

    });

    // @instruction(name="RTI", mode="imp", cycles=6)
    instructions.insert(0x40, |self2| {
        self2.p = (self2.stPop() as u8 | BREAK | UNUSED) as u8;
        self2.pc = self2.stPopWord()
    });

    // @instruction(name="RTS", mode="imp", cycles=6)
    instructions.insert(0x60, |self2| {
        self2.pc = self2.stPopWord();
        self2.pc += 1
    });

    // @instruction(name="TXS", mode="imp", cycles=2)
    instructions.insert(0x9a, |self2|{
        self2.sp = self2.x
    });
    // @instruction(name="TSX", mode="imp", cycles=2)
    instructions.insert(0xba, |self2|{
        self2.x = self2.sp;
        self2.FlagsNZ(self2.x);
    });
    // @instruction(name="CLD", mode="imp", cycles=2)
    instructions.insert(0xd8, |self2| {
        self2.opCLR(DECIMAL as i32);
    });

    // @instruction(name="TYA", mode="imp", cycles=2)
    instructions.insert(0x98, |self2|{
        self2.acc = self2.y;
        self2.FlagsNZ(self2.acc);
    });
    
    // @instruction(name="TAY", mode="imp", cycles=2)
    instructions.insert(0xa8, |self2|{
        self2.y = self2.acc;
        self2.FlagsNZ(self2.y);
    });
    
    // @instruction(name="BEQ", mode="rel", cycles=2, extracycles=2)
    instructions.insert( 0xf0, |self2|{
        self2.opBST(ZERO as i32);
    });
    
    // @instruction(name="CPY", mode="imm", cycles=2)
    instructions.insert(0xc0, |self2|{
        let addr = self2.ProgramCounter();
        self2.opCMPR(addr, self2.y);
        self2.pc += 1
    });

    // @instruction(name="INC", mode="zpg", cycles=5)
    instructions.insert(0xe6, |self2|{
        let x = self2.ZeroPageAddr();
        self2.opINCR(Some(x));
        self2.pc += 1;
    });
    
    // @instruction(name="DEC", mode="zpg", cycles=5)
    instructions.insert(0xc6, |self2|{
        let x = self2.ZeroPageAddr();
        self2.opDECR(Some(x));
        self2.pc += 1;
    });

    // @instruction(name="STA", mode="zpi", cycles=5)
    instructions.insert(0x92, |self2|{
        let x = self2.ZeroPageIndirectAddr();
        self2.opSTA(x);
        self2.pc += 1;
    });
    
        //     @instruction(name="ADC", mode="zpi", cycles=5)
    instructions.insert(0x72, |self2| {
        let x = self2.ZeroPageIndirectAddr();
        self2.opADC(x);
        self2.pc += 1;

    });
    
        //     @instruction(name="EOR", mode="zpi", cycles=5)
    instructions.insert(0x52, |self2| {
        let x = self2.ZeroPageIndirectAddr();
        self2.opEOR(x);
        self2.pc += 1;

    });

        //     @instruction(name="LDA", mode="zpi", cycles=5)
    instructions.insert(0xb2, |self2| {
        let x = self2.ZeroPageIndirectAddr();
        self2.opLDA(x);
        self2.pc += 1;

    });
    
    // @instruction(name="ASL", mode="acc", cycles=2)
    instructions.insert(0x0a, |self2|{
        self2.opASL(None);
    });

    // @instruction(name="INC", mode="acc", cycles=2)
    instructions.insert(0x1a, |self2|{
        self2.opINCR(None)
    });
    
    // @instruction(name="DEC", mode="acc", cycles=2)
    instructions.insert(0x3a, |self2|{
        self2.opDECR(None);
    });
        
    // @instruction(name="STX", mode="zpg", cycles=3)
    instructions.insert( 0x86, |self2|{
        let y = self2.ZeroPageAddr();
        self2.opSTX(y);
        self2.pc += 1;
    });

        //     @instruction(name="BRA", mode="rel", cycles=1, extracycles=1)
    instructions.insert(0x80, |self2| {
        self2.BranchRelAddr();

    });

    // @instruction(name="ORA", mode="imm", cycles=2)
    instructions.insert(0x09, |self2|{
        let x = self2.ProgramCounter();
        self2.opORA(x);
        self2.pc += 1;
    });
    
    // @instruction(name="STY", mode="zpg", cycles=3)
    instructions.insert(0x84, |self2|{
        let x = self2.ZeroPageAddr();
        self2.opSTY(x);
        self2.pc += 1;
    });
       
        
    return instructions;
}
