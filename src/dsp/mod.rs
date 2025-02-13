use std::sync::{atomic::{AtomicU16, AtomicU8, Ordering}, Arc, RwLock};

use bitmatch::bitmatch;
use client::DSPClient;
use stacks::DSPStacks;

pub mod client;
mod stacks;
mod dsp_int_arithmetic;
mod dsp_config;

const REG_AR0: usize = 0;
const REG_AR1: usize = 1;
const REG_AR2: usize = 2;
const REG_AR3: usize = 3;
const REG_IX0: usize = 4;
const REG_IX1: usize = 5;
const REG_IX2: usize = 6;
const REG_IX3: usize = 7;
const REG_IX4: usize = 8;
const REG_ST0: usize = 12;
const REG_ST1: usize = 13;
const REG_ST2: usize = 14;
const REG_ST3: usize = 15;
const REG_AC0_H: usize = 16;
const REG_AC1_H: usize = 17;
const REG_CONFIG: usize = 18;
const REG_SR: usize = 19;
const REG_PROD_L: usize = 20;
const REG_PROD_M1: usize = 21;
const REG_PROD_H: usize = 22;
const REG_PROD_M2: usize = 23;
const REG_AX0_L: usize = 24;
const REG_AX1_L: usize = 25;
const REG_AX0_H: usize = 26;
const REG_AX1_H: usize = 27;
const REG_AC0_L: usize = 28;
const REG_AC1_L: usize = 29;
const REG_AC0_M: usize = 30;
const REG_AC1_M: usize = 31;

pub struct DSP {
    registers: [u16; 32],
    pc: u16,
//    GC_dsp.pdf says that the first 8kb of aram *is* iram, so it shouldn't be stored as its own thing probably
//    iram: [u16; 0x1000],
    dram: [u16; 0x1000],
    irom: [u16; 0x1000],
    drom: [u16; 0x0800],
    exceptions: u16,
    stacks: DSPStacks,
    aram: Arc<Vec<AtomicU8>>,
    client: DSPClient,
}


impl DSP {
    pub fn new(aram: Arc<Vec<AtomicU8>>) -> (Self, DSPClient) {
	let client = DSPClient::new();
	(Self {
	    registers: [0; 32],
	    pc: 0,
//	    iram: [0; 0x1000],
	    dram: [0; 0x1000],
	    irom: [0; 0x1000],
	    drom: [0; 0x0800],
	    exceptions: 0,
	    stacks: DSPStacks::new(),
	    aram,
	    client: client.clone(),
	}, client)
    }

    #[bitmatch]
    pub fn step(&mut self) {
	if !self.client.control_reg.halt() {
	    let instr = self.imem_read(self.pc);

	    #[bitmatch]
	    match instr {
		"0000_0000_0000_0000" => {}, //NOP
		"0000_0000_0000_01dd" => self.op_dar(d),
		"0000_0000_0000_10dd" => self.op_iar(d),
		"0000_0000_0001_ssdd" => self.op_addarn(s, d),
		"0000_0000_0010_0001" => self.op_halt(),
		_ => unimplemented!("{instr:#018b}"),
	    }
	    
	    self.pc += 1;
	}
    }
    
    fn imem_read(&mut self, addr: u16) -> u16 {
	match addr >> 12 {
	    0x0 => {
		let iram_addr = ((addr & 0x0FFF) as usize) * 2;
		let hi = self.aram.get(iram_addr).unwrap().load(Ordering::Relaxed) as u16;
		let lo = self.aram.get(iram_addr + 1).unwrap().load(Ordering::Relaxed) as u16;
		(hi << 8) | lo
	    },
	    0x8 => {
		let irom_addr = (addr & 0x0FFF) as usize;
		self.irom[irom_addr]
	    },
	    a => unimplemented!("address prefix {a:#06X}"),
	}
    }

    fn dmem_read(&mut self, addr: u16) -> u16 {
	todo!();
    }

    fn dmem_write(&mut self, addr: u16, val: u16) {
	todo!();
    }
}

pub struct DSPControlRegister(pub AtomicU16);

impl DSPControlRegister {
    pub fn halt(&self) -> bool {
	((self.0.load(Ordering::Relaxed) >> 2) & 1) != 0
    }
    
    pub fn set_halt(&self) {
	self.0.fetch_or(0x4, Ordering::Relaxed);
    }

    pub fn clear_halt(&self) {
	self.0.fetch_and(!0x4, Ordering::Relaxed);
    }
}
