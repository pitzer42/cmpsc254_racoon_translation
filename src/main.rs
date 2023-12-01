
#![allow(non_snake_case)]
// Constants that we use to manipulate the p (status) register https://en.wikibooks.org/wiki/6502_Assembly#Registers
const NEGATIVE : u8 = 128;
const OVERFLOW : u8 = 64;
const UNUSED : u8 = 32;
const BREAK : u8 = 16;
const DECIMAL: u8 = 8;
const INTERRUPT : u8 = 4;
const ZERO : u8 = 2;
const CARRY : u8 = 1;

const BYTE_WIDTH : i8 = 8;
const ADDR_WIDTH : i8 = 16;

// pre set memory addresses for certain operations
const RESET: u16 = 0xfffc;
const NMI: u16 = 0xfffa;
const IRQ: u16 = 0xfffe;


fn main() {
    let mut test = Mpu6502::new();
    test.ImmediateByte();
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


    memory: [i8; 0xffff],

    spBase: i32,
    start_pc: i32,

    byteMask: i32,
    addrMask: i32,
    addrHighMask: i32,

    excycles: i32,
    addcycles: bool,
    processorCycles: i32,
    
}

impl Mpu6502 {
    fn new() -> Self {
        let byteMask : i32 = (1 << BYTE_WIDTH)-1;
        let start_pc = 0;
        Mpu6502 {
            pc: start_pc, 
            sp: byteMask,
            acc: 0, 
            p: 0 | UNUSED | BREAK,
            x: 0,
            y: 0,
            byteMask: byteMask,
            addrMask: ((1<< ADDR_WIDTH)-1),
            addrHighMask: (byteMask << BYTE_WIDTH),
            spBase: 1 << BYTE_WIDTH,
            excycles: 0,
            addcycles: false,
            processorCycles: 0,
            memory: [0; 0xffff],
            start_pc: start_pc,
        }
    }

    pub fn ByteAt(&mut self, addr: i32) -> i32{
        let val = self.memory[addr as usize];
        val as i32
    }

    pub fn WordAt(&mut self, addr: i32) -> i32{
        self.ByteAt(addr) + (self.ByteAt(addr + 1) << BYTE_WIDTH)
    }

    pub fn WrapAt(&mut self, addr: i32) -> i32{
        let wrapped_addr = (addr & self.addrHighMask) + ((addr+1) & self.byteMask);
        self.ByteAt(addr) + (self.ByteAt(wrapped_addr) << BYTE_WIDTH) 
    }
    pub fn ProgramCounter(&mut self) -> i32{
        self.pc
    }

    pub fn reset(&mut self){
        self.pc = self.start_pc;
        self.sp = self.byteMask;
        self.acc =0;
        self.x = 0;
        self.y = 0;
        self.p = BREAK | UNUSED;
        self.processorCycles = 0;
    }
    pub fn opAsl(&mut self){
        let mut tbyte = self.acc;
        self.p &= !(CARRY | NEGATIVE | ZERO);
        if tbyte as u8 & NEGATIVE != 0 {
            self.p |= CARRY
        }
        tbyte = (tbyte << 1) & self.byteMask;
        if tbyte != 0 {
            self.p |= NEGATIVE & tbyte as u8;
        } else {
            self.p |= ZERO;
        }
        self.acc = tbyte;
        println!("{:#b}", NEGATIVE);
        println!("{:#b}", self.p);

    }
    



    pub fn ImmediateByte(&mut self) -> i32{
        self.ByteAt(self.pc)
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
            return a2
        }
        else {
            (self.WrapAt(byte_at) + self.y) & self.addrMask
        }
    }
        

    pub fn AbsoluteAddr(&mut self) -> i32{
        self.WordAt(self.pc)
    }

    pub fn AbsoluteXAddr(&mut self) -> i32 {
        if self.addcycles {
            let a1 = self.WordAt(self.pc);
            let a2 = (a1 + self.x) & self.addrMask;
            if a1 & self.addrHighMask != a2 & self.addrHighMask {
                self.excycles += 1
            }
            return a2
        }
       
        else {
            return (self.WordAt(self.pc) + self.x) & self.addrMask

        }
    }
    // NEW OPS 11/30
    //TEMP FLAGSNZ
    pub fn FlagsNZ(&mut self, value: i32){
        self.p &= !(ZERO | NEGATIVE);
        if value == 0{
            self.p = self.p | ZERO;
        } else {
            self.p = self.p | (value & NEGATIVE as i32) as u8;
    	}
    }
    pub fn opSTX(&mut self, y: i32) {
        self.memory[y as usize] = self.x as i8;
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
    pub fn opDecr(&mut self, x: Option<i32>) {
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
            self.memory[addr as usize] = tbyte as i8;
        }

    }
    pub fn opIncr(&mut self, x: Option<i32>) {
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
            self.memory[addr as usize] = tbyte as i8;
        }
    }
    pub fn opADC(&mut self, x: i32) {
        let mut data = self.ByteAt(x);

        if (self.p & DECIMAL) != 0  {
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
            }
            else {
                self.p |= aluresult as u8 & NEGATIVE;
            }
            if decimalcarry == 1 {
                self.p |= CARRY;
            }
            if ((!(self.acc ^ data) & (self.acc ^ aluresult)) & NEGATIVE as i32) != 0 {
                self.p |= OVERFLOW;
            }
            self.acc = (nibble1 << 4) + nibble0;
        }
        else {
            let mut tmp: i32 = 0;
            if (self.p & CARRY) != 0 {
                tmp = 1;
            }
            else {
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
            }
            else {
                self.p |= data as u8 & NEGATIVE;

            }
            self.acc = data;
        }
    }
    pub fn opSBC(&mut self, x:i32) {
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

            if((self.acc ^ data) & (self.acc ^ result) & NEGATIVE as i32) != 0 {
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
