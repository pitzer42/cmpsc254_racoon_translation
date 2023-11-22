
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
    
    pub fn ImmediateByte(&mut self) -> i32{
        return self.ByteAt(self.pc)
    }
    
    pub fn FlagsNZ(&mut self, foo: i32 ){
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
        if ((tbyte as u8 & NEGATIVE) != 0){
            self.p |= CARRY
        }
        tbyte = (tbyte << 1) & self.byteMask;
        if (tbyte != 0){
            self.p |= NEGATIVE & tbyte as u8;
        } else {
            self.p |= ZERO;
        }
        self.acc = tbyte;
        println!("{:#b}", NEGATIVE);
        println!("{:#b}", self.p);

    }
    
    pub fn opROL(&mut self, x: i32) {
        let mut tbyte = self.acc;
        let mut addr:i32 = 0;
        
        if x != -1{
            //addr = x(); not sure what is the rust equivalent to this line
            tbyte = self.ByteAt(addr);
        }
        if (self.p != 0) & (CARRY != 0) {
            if (tbyte != 0) & (NEGATIVE != 0) {
                /*pass*/
            } else {
                self.p = self.p | CARRY;
            }
            tbyte = (tbyte << 1) | 1;
        } else {
            if (tbyte != 0) & (NEGATIVE != 0) {
                self.p |= CARRY;
            }
            tbyte = tbyte << 1;
        }
        tbyte &= self.byteMask;
        self.FlagsNZ(tbyte);
        if x == -1 {
            self.acc = tbyte;
        } else {
            self.memory[addr as usize] = tbyte as i8;
        }
    }
    
    pub fn AbsoluteYAddr(&mut self) -> i32{
        if self.addcycles {
            let a1 = self.WordAt(self.pc);
            let a2 = (a1 + self.y) & self.addrMask;
            if(a1 & self.addrHighMask) != (a2 & self.addrHighMask){
                self.excycles += 1;
            }
            return a2;
        }
        return (self.WordAt(self.pc) + self.y) & self.addrMask;
    }
    
    pub fn BranchRelAddr(&mut self){
    	self.excycles += 1;
    	let mut addr = self.ImmediateByte();
    	self.pc += 1;
    	
    	if (addr & (NEGATIVE as i32)) == 0{
    	    addr = self.pc + addr;
    	} else {
    	    addr = self.pc - (addr ^ self.byteMask) -1;
    	}
    	
    	if(self.pc & self.addrHighMask) != (addr & self.addrHighMask){
    	    self.excycles += 1;
    	}
    	
    	self.pc = addr & self.addrMask;
    }
}
