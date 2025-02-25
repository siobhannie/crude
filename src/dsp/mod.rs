use std::{fs::File, io::Read, ops::Deref, sync::{atomic::{AtomicU16, AtomicU8, Ordering}, Arc, RwLock}};

use bitmatch::bitmatch;
use byteorder::{BigEndian, ByteOrder};
use client::DSPClient;
use stacks::DSPStacks;

pub mod client;
mod stacks;
mod dsp_int_arithmetic;
mod dsp_config;
pub mod dsp_interface;
mod dsp_control_flow;
mod dsp_load_store;
mod dsp_bitwise;
mod dsp_misc;
mod util;

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

const SR_LZ: u16 = 1 << 6;

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
    control: Arc<DSPControlRegister>,
    cpu_mbox_h: Arc<AtomicU16>,
    cpu_mbox_l: Arc<AtomicU16>,
    dsp_mbox_h: Arc<AtomicU16>,
    dsp_mbox_l: Arc<AtomicU16>,
}


impl DSP {
    pub fn new(aram: Arc<Vec<AtomicU8>>) -> (Self, DSPClient) {
	let control =  Arc::new(DSPControlRegister(AtomicU16::new(0x5)));
	let cpu_mbox_h = Arc::new(AtomicU16::new(0));
	let cpu_mbox_l = Arc::new(AtomicU16::new(0));
	let dsp_mbox_h = Arc::new(AtomicU16::new(0));
	let dsp_mbox_l = Arc::new(AtomicU16::new(0));
	let mut irom_dump = Vec::new();
	let mut irom = [0u16; 0x1000];
	let irom_file = File::open("get-your-own/dsp_rom.bin").unwrap().read_to_end(&mut irom_dump);
	for i in 0..(irom.len() / 2) {
	    let val = BigEndian::read_u16(&irom_dump[(i * 2)..]);
	    irom[i] = val;
	}
	(Self {
	    registers: [0; 32],
	    pc: 0,
//	    iram: [0; 0x1000],
	    dram: [0; 0x1000],
	    irom,
	    drom: [0; 0x0800],
	    exceptions: 0,
	    stacks: DSPStacks::new(),
	    aram,
	    control: control.clone(),
	    cpu_mbox_h: cpu_mbox_h.clone(),
	    cpu_mbox_l: cpu_mbox_l.clone(),
	    dsp_mbox_h: dsp_mbox_h.clone(),
	    dsp_mbox_l: dsp_mbox_l.clone(),
	},
	 DSPClient {
	     control_reg: control,
	     cpu_mbox_h,
	     cpu_mbox_l,
	     dsp_mbox_h,
	     dsp_mbox_l,
	 })
    }

    pub fn read_reg(&mut self, reg: usize) -> u16 {
	match reg {
	    REG_ST0 | REG_ST1 | REG_ST2 | REG_ST3 => {
		self.pop_stack(reg - REG_ST0)
	    },
	    REG_AR0 | REG_AR1 | REG_AR2 | REG_AR3 | REG_IX0 | REG_IX1 | REG_IX2 | REG_IX3 | 0x8 | 0x9 | 0xa | 0xb | REG_PROD_L | REG_PROD_M1 | REG_PROD_M2 | REG_AX0_L | REG_AX1_L | REG_AX0_H | REG_AX1_H | REG_AC0_L | REG_AC1_L | REG_CONFIG | REG_SR | REG_PROD_H | REG_AC0_H | REG_AC1_H => {
		self.registers[reg]
	    },
	    REG_AC0_M | REG_AC1_M => {
		if (self.registers[REG_SR] & 0x4000) != 0 {
		    let long = self.acc(reg - REG_AC0_M) as i64;

		    if long != ((long as i32) as i64) {
			if long > 0 {
			    0x7fff
			} else {
			    0x8000
			}
		    } else {
			self.registers[reg]
		    }
		} else {
		    self.registers[reg]
		}
	    },
	    _ => unreachable!(),
	}
    }

    pub fn write_reg(&mut self, reg: usize, val: u16) {
	match reg {
	    REG_AC0_H | REG_AC1_H => {
		self.registers[reg] = (val as i8) as u16;
	    },
	    REG_ST0 | REG_ST1 | REG_ST2 | REG_ST3 => {
		self.push_stack(reg - REG_ST0, val);
	    },
	    REG_AR0 | REG_AR1 | REG_AR2 | REG_AR3 | REG_IX0 | REG_IX1 | REG_IX2 | REG_IX3 | 0x8 | 0x9 | 0xa | 0xb | REG_PROD_L | REG_PROD_M1 | REG_PROD_M2 | REG_AX0_L | REG_AX1_L | REG_AX0_H | REG_AX1_H | REG_AC0_L | REG_AC1_L | REG_AC0_M | REG_AC1_M => {
		self.registers[reg] = val;
	    },
	    REG_CONFIG | REG_PROD_H => {
		self.registers[reg] = val & 0x00ff;
	    },
	    REG_SR => {
		self.registers[reg] = val & !0x100;
	    }
	    _ => unreachable!(),
	}
    }

    pub fn acc(&self, acc: usize) -> u64 {
	let h = self.registers[REG_AC0_H + acc] as u64;
	let m = self.registers[REG_AC0_M + acc] as u64;
	let l = self.registers[REG_AC0_L + acc] as u64;

	(h << 32) | (m << 16) | l
    }

    #[bitmatch]
    pub fn step(&mut self) {
	if self.control.reset() {
	    //do reset!!!!!!!
	    self.pc = 0x8000;
	    self.control.clear_reset();
	}
	if !self.control.halt() {
	    let instr = self.imem_read(self.pc);
	    println!("instr: {instr:#018b}, pc: {:#06X}", self.pc);
	    #[bitmatch]
	    match instr {
		"0000_0000_0000_0000" => {self.pc += 1}, //NOP
		"0000_0000_0000_01dd" => self.op_dar(d),
		"0000_0000_0000_10dd" => self.op_iar(d),
		"0000_0000_0001_ssdd" => self.op_addarn(s, d),
		"0000_0000_0010_0001" => self.op_halt(),
		"0000_0000_010r_rrrr" => self.op_loop(r),
		"0000_0000_011r_rrrr" => self.op_bloop(r),
		"0000_0000_100d_dddd" => self.op_lri(d),
		"0000_0000_110d_dddd" => self.op_lr(d),
		"0000_0000_111s_ssss" => self.op_sr(s),
		"0000_001d_0001_00ss" => self.op_ilrr(d, s, 0),
		"0000_001d_0001_01ss" => self.op_ilrr(d, s, -1),
		"0000_001d_0001_10ss" => self.op_ilrr(d, s, 1),
		"0000_001d_0001_11ss" => self.op_ilrr(d, s, self.registers[(s + 4) as usize] as i16),
		"0000_0010_0111_cccc" => self.op_if(c),
		"0000_0010_1001_cccc" => self.op_j(c),
		"0000_0010_1011_cccc" => self.op_call(c),
		"0000_0010_1101_cccc" => self.op_ret(c),
		"0000_001r_1010_0000" => self.op_andcf(r),
		"0000_001r_1100_0000" => self.op_andf(r),
		"0001_0010_0000_0iii" => self.op_sbset(i),
		"0001_0110_iiii_iiii" => self.op_si(i),
		"0001_1001_0ssd_dddd" => self.op_lrri(s, d),
		"0001_11dd_ddds_ssss" => self.op_mrr(d, s),
		"0010_0ddd_mmmm_mmmm" => self.op_lrs(d, m),
		"0010_1sss_mmmm_mmmm" => self.op_srs(s, m),
		"1000_r001_xxxx_xxxx" => {
		    self.op_clr(r);
		    self.extension(x as u8);
		},
		"1000_0010_xxxx_xxxx" => {
		    self.op_cmp();
		    self.extension(x as u8);
		},
		"1000_1bbb_xxxx_xxxx" => {
		    self.op_srbit(b);
		    self.extension(x as u8);
		}
		_ => unimplemented!("{instr:#018b}"),
	    }
	}
    }

    #[bitmatch]
    fn extension(&mut self, instr: u8) {
	#[bitmatch]
	match instr {
	    "00000_0000" => {}, //NOP
	    _ => unimplemented!("{instr:#010b}"),
	}
    }

    pub fn do_sr(&mut self, res: i64, carry: bool, overflow: bool) {
	self.registers[REG_SR] &= !0x3f;

	if carry {
	    self.registers[REG_SR] |= 0x1;
	}

	if overflow {
	    self.registers[REG_SR] |= 0x2;
	    self.registers[REG_SR] |= 0x80;
	}

	if res == 0 {
	    self.registers[REG_SR] |= 0x4;
	}

	if res < 0 {
	    self.registers[REG_SR] |= 0x8;
	}

	if res != ((res as i32) as i64) {
	    self.registers[REG_SR] |= 0x10;
	}

	if ((res & 0xc000_0000) == 0) || ((res & 0xc000_0000) == 0xc000_0000) {
	    self.registers[REG_SR] |= 0x20;
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

    fn condition(&self, c: u16) -> bool {
	match c {
	    0b0101 => (self.registers[REG_SR] & 0x4) != 0,
	    0b1100 => !((self.registers[REG_SR] & SR_LZ) != 0),
	    0b1111 => true,
	    a => unimplemented!("condition code {a:#06b}"),
	}
    }

    fn dmem_read(&mut self, addr: u16) -> u16 {
	match addr >> 12 {
	    0x0 => {
		let dram_addr = (addr & 0x0FFF) as usize;
		self.dram[dram_addr]
	    },
	    0xF => {
		match addr {
		    0xFFFC => self.dsp_mbox_h.load(Ordering::Relaxed),
		    0xFFFE => self.cpu_mbox_h.load(Ordering::Relaxed),
		    0xFFFF => self.cpu_mbox_l.load(Ordering::Relaxed),
		    _ => unimplemented!("HW register {addr:#06X}"),
		}
	    }
	    a => unimplemented!("address prefix {a:#06X}"),
	}
    }

    fn dmem_write(&mut self, addr: u16, val: u16) {
	match addr >> 12 {
	    0x0 => {
		let dram_addr = (addr & 0x0FFF) as usize;
		self.dram[dram_addr] = val;
	    }
	    0xF => {
		match addr {
		    0xFFFC => self.dsp_mbox_h.store(val, Ordering::Relaxed),
		    0xFFFD => self.dsp_mbox_l.store(val, Ordering::Relaxed),
		    _ => unimplemented!("HW register {addr:#06X}"),
		}
	    }
	    a => unimplemented!("address prefix {a:#06X}"),
	}
    }

    fn push_stack(&mut self, stack: usize, val: u16) {
	self.stacks.pointers[stack] += 1;
	self.stacks.pointers[stack] &= 0x1f;
	self.stacks.stacks[stack][self.stacks.pointers[stack]] = self.registers[REG_ST0 + stack];

	self.registers[REG_ST0 + stack] = val;
    }

    fn pop_stack(&mut self, stack: usize) -> u16 {
	let val = self.registers[REG_ST0 + stack];

	self.registers[REG_ST0 + stack] = self.stacks.stacks[stack][self.stacks.pointers[stack]];
	self.stacks.pointers[stack] -= 1;
	self.stacks.pointers[stack] &= 0x1f;
	
	val
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

    pub fn reset(&self) -> bool {
	(self.0.load(Ordering::Relaxed) & 1) != 0
    }

    pub fn clear_reset(&self) {
	self.0.fetch_and(!0x1, Ordering::Relaxed);
    }

    pub fn init(&self) -> bool {
	(self.0.load(Ordering::Relaxed) & 0x0800) != 0
    }
}

impl Deref for DSPControlRegister {
    type Target = AtomicU16;

    fn deref(&self) -> &Self::Target {
	&self.0
    }
}
