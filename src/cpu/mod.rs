pub mod instr;
pub mod float;
pub mod arithmetic;
pub mod config;
pub mod load_store;
pub mod bitwise;
pub mod cache;
pub mod mmu;
pub mod util;
pub mod control_flow;

use std::cmp::Ordering;

use arithmetic::{add, addc, adde, addi, addic, addicr, addis, cmp, cmpi, cmpl, cmpli, subf};
use bitwise::{and, crxor, nor, or, ori, oris, rlwinm};
use cache::isync;
use config::{mffs, mfmsr, mfspr, mftb, mtfsf, mtmsr, mtspr, mtsr};
use control_flow::{b, bc, bclr};
use float::{ps_mr, fmr};
use instr::Instruction;
use load_store::{lfd, lwz, psq_l, stfd, sth, stw, stwu};
use log::{debug, info};
use mmu::Mmu;

pub const RESET_EXCEPTION: u32   = 0x1;
pub const PROGRAM_EXCEPTION: u32 = 0x2;

use crate::Gamecube;

pub struct Cpu {
    pub cia: u32,
    pub nia: u32,
    pub gprs: [u32; 32],
    pub mmu: Mmu,
    pub hid0: u32,
    pub hid2: HID2,
    pub wpar: u32,
    pub gqrs: [GraphicsQuantizationRegister; 8],
    pub fprs: [FloatingPointRegister; 32],
    pub msr: MachineStateRegister,
    pub tb: u64,
    pub cr: ConditionRegister,
    pub ctr: u32,
    pub lr: u32,
    pub srr0: u32,
    pub srr1: u32,
    pub exceptions: u32,
    pub fpscr: FloatingPointStatusControlRegister,
    pub l2cr: u32,
    pub xer: XER,
    pub pmcs: [u32; 4],
    pub mmcr0: u32,
    pub mmcr1: u32,
}

impl Cpu {
    pub fn new() -> Self {
	Self {
	    cia: 0xFFF0_0100,
	    nia: 0xFFF0_0100,
	    gprs: [0; 32],
	    mmu: Mmu::new(),
	    hid0: 0,
	    hid2: HID2(0),
	    wpar: 0,
	    gqrs: [GraphicsQuantizationRegister(0); 8],
	    fprs: [FloatingPointRegister::from_u64(0); 32],
	    msr: MachineStateRegister(0),
	    tb: 0,
	    cr: ConditionRegister(0),
	    ctr: 0,
	    lr: 0,
	    srr0: 0,
	    srr1: 0,
	    exceptions: 0,
	    fpscr: FloatingPointStatusControlRegister(0),
	    l2cr: 0,
	    xer: XER(0),
	    pmcs: [0; 4],
	    mmcr0: 0,
	    mmcr1: 0,
	}
    }

    pub fn do_cr0(&mut self, val: u32) {
	let val = val as i32;

	let order = match val.cmp(&0) {
	    Ordering::Equal => 0x2,
	    Ordering::Greater => 0x4,
	    Ordering::Less => 0x8,
	};

	//xer here :3

	self.cr.set_reg(0, order);
    }

    pub fn exception(&mut self) {
	if self.exceptions & RESET_EXCEPTION != 0 {
	    if self.msr.ip() {
		self.cia = 0xFFF0_0100;
	    } else {
		self.cia = 0x100;
	    }

	    self.exceptions &= !RESET_EXCEPTION;
	} else if self.exceptions & PROGRAM_EXCEPTION != 0 {
	    self.srr0 = self.nia;
	    self.srr1 = self.msr.0 & 0x87C0_FFFF;
	    self.msr.set_le(self.msr.ile());
	    self.msr.0 &= !0x04EF36;

	    self.exceptions &= !PROGRAM_EXCEPTION;
	}
    }
}

pub fn step(gc: &mut Gamecube) {
    let addr = gc.cpu.cia;

    let instruction = Instruction(gc.read_u32(addr, true));
    
    match instruction.opcd() {
	0b000100 => ps_mr(gc, &instruction),
	0b001010 => cmpli(gc, &instruction),
	0b001011 => cmpi(gc, &instruction),
	0b001100 => addic(gc, &instruction),
	0b001101 => addicr(gc, &instruction),
	0b001110 => addi(gc, &instruction),
	0b001111 => addis(gc, &instruction),
	0b010000 => bc(gc, &instruction),
	0b010010 => b(gc, &instruction),
	0b010101 => rlwinm(gc, &instruction),
	0b011000 => ori(gc, &instruction),
	0b011001 => oris(gc, &instruction),
	0b010011 => match instruction.sec_opcd() {
	    0b0000010000 => bclr(gc, &instruction),
	    0b0010010110 => isync(gc, &instruction),
	    0b0011000001 => crxor(gc, &instruction),
	    a => unimplemented!("secondary opcode: {a:#012b}, primary: 0b010011, instruction: {:#034b}", instruction.0),
	},
	0b011111 => match instruction.sec_opcd() {
	    0b0000000000 => cmp(gc, &instruction),
	    0b0000001010 => addc(gc, &instruction),
	    0b0000011100 => and(gc, &instruction),
	    0b0000100000 => cmpl(gc, &instruction),
	    0b0000101000 => subf(gc, &instruction),
	    0b0001010011 => mfmsr(gc, &instruction),
	    0b0001111100 => nor(gc, &instruction),
	    0b0010001010 => adde(gc, &instruction),
	    0b0010010010 => mtmsr(gc, &instruction),
	    0b0011010010 => mtsr(gc, &instruction),
	    0b0100001010 => add(gc, &instruction),
	    0b0101010011 => mfspr(gc, &instruction),
	    0b0101110011 => mftb(gc, &instruction),
	    0b0110111100 => or(gc, &instruction),
	    0b0111010011 => mtspr(gc, &instruction),
	    0b1001010110 => {}, //sync
	    a => unimplemented!("secondary opcode: {a:#012b}, primary: 0b011111, instruction: {:#034b}", instruction.0),
	},
	0b100000 => lwz(gc, &instruction),
	0b100100 => stw(gc, &instruction),
	0b100101 => stwu(gc, &instruction),
	0b101100 => sth(gc, &instruction),
	0b110010 => lfd(gc, &instruction),
	0b110110 => stfd(gc, &instruction),
	0b111000 => psq_l(gc, &instruction),
	0b111111 => match instruction.sec_opcd() {
	    0b0001001000 => fmr(gc, &instruction),
	    0b1001000111 => mffs(gc, &instruction),
	    0b1011000111 => mtfsf(gc, &instruction),
	    a => unimplemented!("secondary opcode {a:#012b}, primary: 0b111111, instruction: {:#034b}", instruction.0),
	}
	a => unimplemented!("opcode: {a:#08b}, instruction: {:#034b}", instruction.0),
    }
    
    gc.cpu.cia = gc.cpu.nia;

    gc.cpu.tb += 1;

    gc.cpu.nia = gc.cpu.cia.wrapping_add(4);
}

pub fn write_hid0(gc: &mut Gamecube, val: u32) {
    debug!("STUB: hid0 write with val {val:#034b}");
    gc.cpu.hid0 = val;
}

pub fn write_msr(gc: &mut Gamecube, val: u32) {
    debug!("STUB: msr write with val {val:#034b}");
    gc.cpu.msr = MachineStateRegister(val);
}

pub struct MachineStateRegister(pub u32);

impl MachineStateRegister {
    pub fn pow(&self) -> bool {
	((self.0 >> 18) & 1) != 0
    }

    pub fn ile(&self) -> bool {
	((self.0 >> 16) & 1) != 0
    }

    pub fn ee(&self) -> bool {
	((self.0 >> 15) & 1) != 0
    }
    
    pub fn pr(&self) -> bool {
	((self.0 >> 14) & 1) != 0
    }

    pub fn fp(&self) -> bool {
	((self.0 >> 13) & 1) != 0
    }

    pub fn me(&self) -> bool {
	((self.0 >> 12) & 1) != 0
    }

    pub fn fe0(&self) -> bool {
	((self.0 >> 11) & 1) != 0
    }

    pub fn se(&self) -> bool {
	((self.0 >> 10) & 1) != 0
    }

    pub fn be(&self) -> bool {
	((self.0 >> 9) & 1) != 0
    }

    pub fn fe1(&self) -> bool {
	((self.0 >> 8) & 1) != 0
    }

    pub fn ip(&self) -> bool {
	((self.0 >> 6) & 1) != 0
    }

    pub fn ir(&self) -> bool {
	((self.0 >> 5) & 1) != 0
    }

    pub fn dr(&self) -> bool {
	((self.0 >> 4) & 1) != 0
    }

    pub fn pm(&self) -> bool {
	((self.0 >> 2) & 1) != 0
    }

    pub fn ri(&self) -> bool {
	((self.0 >> 1) & 1) != 0
    }

    pub fn le(&self) -> bool {
	(self.0 & 1) != 0
    }

    pub fn set_le(&mut self, val: bool) {
	self.0 = (self.0 & !1) | (val as u32);
    }
}

pub struct ConditionRegister(pub u32);

impl ConditionRegister {
    pub fn set_reg(&mut self, index: usize, val: u32) {
	self.0 = (self.0 & (!(0xF000_0000 >> (index * 4)))) | (val << ((7 - index) * 4));
    }
    
    pub fn get_reg(&self, index: usize) -> usize {
	((self.0 >> (28 - (index * 4))) & 0xF) as usize 
    }
}

#[derive(Copy, Clone)]
pub struct GraphicsQuantizationRegister(pub u32);

impl GraphicsQuantizationRegister {
    pub fn st_type(&self) -> usize {
	(self.0 & 0x7) as usize
    }

    pub fn st_scale(&self) -> usize {
	((self.0 >> 8) & 0x3F) as usize
    }

    pub fn ld_type(&self) -> usize {
	((self.0 >> 16) & 0x7) as usize
    }

    pub fn ld_scale(&self) -> usize {
	((self.0 >> 24) & 0x3F) as usize
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union FloatingPointRegister {
    pub ps_i: (u32, u32),
    pub ps_f: (f32, f32),
    pub double: f64,
    pub int: u64,
}

//i don't know if i need all of these fields and methods but i kinda zoned out while writing them and now they're here so might as well keep them
impl FloatingPointRegister {
    pub fn from_paired_u32(ints: (u32, u32)) -> Self {
	Self {
	    ps_i: ints
	}
    }

    pub fn from_paired_f32(floats: (f32, f32)) -> Self {
	Self {
	    ps_f: floats
	}
    }

    pub fn from_f64(double: f64) -> Self {
	Self {
	    double,
	}
    }

    pub fn from_u64(int: u64) -> Self {
	Self {
	    int,
	}
    }
    
    pub fn as_paired_u32(&self) -> &(u32, u32) {
	unsafe {
	    &self.ps_i
	}
    }

    pub fn as_paired_u32_mut(&mut self) -> &mut (u32, u32) {
	unsafe {
	    &mut self.ps_i
	}
    }

    pub fn as_paired_f32(&self) -> &(f32, f32) {
	unsafe {
	    &self.ps_f
	}
    }

    pub fn as_paired_f32_mut(&mut self) -> &mut (f32, f32) {
	unsafe {
	    &mut self.ps_f
	}
    }

    pub fn as_f64(&self) -> &f64 {
	unsafe {
	    &self.double
	}
    }

    pub fn as_f64_mut(&mut self) -> &mut f64 {
	unsafe {
	    &mut self.double
	}
    }

    pub fn as_u64(&self) -> &u64 {
	unsafe {
	    &self.int
	}
    }

    pub fn as_u64_mut(&mut self) -> &mut u64 {
	unsafe {
	    &mut self.int
	}
    }
}

pub struct HID2(pub u32);

impl HID2 {
    pub fn lsqe(&self) -> bool {
	((self.0 >> 31) & 1) != 0
    }

    pub fn wpe(&self) -> bool {
	((self.0 >> 30) & 1) != 0
    }

    pub fn pse(&self) -> bool {
	((self.0 >> 29) & 1) != 0
    }

    pub fn lce(&self) -> bool {
	((self.0 >> 28) & 1) != 0
    }

    pub fn dmaql(&self) -> usize {
	((self.0 >> 24) & 0xF) as usize
    }
}

pub struct FloatingPointStatusControlRegister(pub u32);

impl FloatingPointStatusControlRegister {
    pub fn rn(&self) -> usize {
	(self.0 & 0x3) as usize
    }

    pub fn ni(&self) -> bool {
	((self.0 >> 2) & 1) != 0
    }

    pub fn xe(&self) -> bool {
	((self.0 >> 3) & 1) != 0
    }

    pub fn ze(&self) -> bool {
	((self.0 >> 4) & 1) != 0
    }

    pub fn ue(&self) -> bool {
	((self.0 >> 5) & 1) != 0
    }

    pub fn oe(&self) -> bool {
	((self.0 >> 6) & 1) != 0
    }

    pub fn ve(&self) -> bool {
	((self.0 >> 7) & 1) != 0
    }

    pub fn vxcvi(&self) -> bool {
	((self.0 >> 8) & 1) != 0
    }

    pub fn vxsqrt(&self) -> bool {
	((self.0 >> 9) & 1) != 0
    }

    pub fn vxsoft(&self) -> bool {
	((self.0 >> 10) & 1) != 0
    }

    pub fn fprf(&self) -> usize {
	((self.0 >> 12) & 0x1F) as usize
    }

    pub fn fi(&self) -> bool {
	((self.0 >> 17) & 1) != 0 
    }

    pub fn fr(&self) -> bool {
	((self.0 >> 18) & 1) != 0
    }

    pub fn vxvc(&self) -> bool {
	((self.0 >> 19) & 1) != 0
    }

    pub fn vximz(&self) -> bool {
	((self.0 >> 20) & 1) != 0
    }

    pub fn vxzdz(&self) -> bool {
	((self.0 >> 21) & 1) != 0
    }

    pub fn vxidi(&self) -> bool {
	((self.0 >> 22) & 1) != 0
    }

    pub fn vxisi(&self) -> bool {
	((self.0 >> 23) & 1) != 0
    }

    pub fn vxsnan(&self) -> bool {
	((self.0 >> 24) & 1) != 0
    }

    pub fn xx(&self) -> bool {
	((self.0 >> 25) & 1) != 0
    }

    pub fn zx(&self) -> bool {
	((self.0 >> 26) & 1) != 0
    }

    pub fn ux(&self) -> bool {
	((self.0 >> 27) & 1) != 0
    }

    pub fn ox(&self) -> bool {
	((self.0 >> 28) & 1) != 0
    }

    pub fn vx(&self) -> bool {
	((self.0 >> 29) & 1) != 0
    }

    pub fn fex(&self) -> bool {
	((self.0 >> 30) & 1) != 0
    }

    pub fn fx(&self) -> bool {
	((self.0 >> 31) & 1) != 0
    }
}

pub struct XER(pub u32);

impl XER {
    pub fn byte_count(&self) -> usize {
	(self.0 & 0x7F) as usize
    }

    pub fn ca(&self) -> bool {
	((self.0 >> 29) & 1) != 0
    }

    pub fn set_ca(&mut self, val: bool) {
	self.0 = (self.0 & !(1 << 29)) | ((val as u32) << 29);
    }

    pub fn ov(&self) -> bool {
	((self.0 >> 30) & 1) != 0
    }

    pub fn so(&self) -> bool {
	((self.0 >> 31) & 1) != 0
    }
}
