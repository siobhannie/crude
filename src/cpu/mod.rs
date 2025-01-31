pub mod instr;
pub mod arithmetic;
pub mod config;
pub mod load_store;
pub mod bitwise;
pub mod cache;
pub mod mmu;
pub mod util;
pub mod control_flow;

use std::cmp::Ordering;

use arithmetic::{addi, addis, cmpi, cmpl, cmpli, subf};
use bitwise::{and, nor, or, ori, rlwinm};
use cache::isync;
use config::{mfmsr, mfspr, mftb, mtmsr, mtspr, mtsr};
use control_flow::{b, bc, bclr};
use instr::Instruction;
use load_store::{lwz, sth, stw, stwu};
use log::{debug, info};
use mmu::Mmu;

use crate::Gamecube;

pub struct Cpu {
    pub cia: u32,
    pub nia: u32,
    pub gprs: [u32; 32],
    pub mmu: Mmu,
    pub hid0: u32,
    pub msr: MachineStateRegister,
    pub tb: u64,
    pub cr: ConditionRegister,
    pub ctr: u32,
    pub lr: u32,
}

impl Cpu {
    pub fn new() -> Self {
	Self {
	    cia: 0xFFF0_0100,
	    nia: 0xFFF0_0100,
	    gprs: [0; 32],
	    mmu: Mmu::new(),
	    hid0: 0,
	    msr: MachineStateRegister(0),
	    tb: 0,
	    cr: ConditionRegister(0),
	    ctr: 0,
	    lr: 0,
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
}

pub fn step(gc: &mut Gamecube) {
    let addr = gc.cpu.cia;

    let instruction = Instruction(gc.read_u32(addr, true));
    
    match instruction.opcd() {
	0b001010 => cmpli(gc, &instruction),
	0b001011 => cmpi(gc, &instruction),
	0b001110 => addi(gc, &instruction),
	0b001111 => addis(gc, &instruction),
	0b010000 => bc(gc, &instruction),
	0b010010 => b(gc, &instruction),
	0b010101 => rlwinm(gc, &instruction),
	0b011000 => ori(gc, &instruction),
	0b010011 => match instruction.sec_opcd() {
	    0b0000010000 => bclr(gc, &instruction),
	    0b0010010110 => isync(gc, &instruction),
	    a => unimplemented!("secondary opcode: {a:#012b}, priomary: 0b010011, instruction: {:#034b}", instruction.0),
	},
	0b011111 => match instruction.sec_opcd() {
	    0b0000011100 => and(gc, &instruction),
	    0b0000100000 => cmpl(gc, &instruction),
	    0b0000101000 => subf(gc, &instruction),
	    0b0001010011 => mfmsr(gc, &instruction),
	    0b0001111100 => nor(gc, &instruction),
	    0b0010010010 => mtmsr(gc, &instruction),
	    0b0011010010 => mtsr(gc, &instruction),
	    0b0101010011 => mfspr(gc, &instruction),
	    0b0101110011 => mftb(gc, &instruction),
	    0b0110111100 => or(gc, &instruction),
	    0b0111010011 => mtspr(gc, &instruction),
	    a => unimplemented!("secondary opcode: {a:#012b}, primary: 0b011111, instruction: {:#034b}", instruction.0),
	},
	0b100000 => lwz(gc, &instruction),
	0b100100 => stw(gc, &instruction),
	0b100101 => stwu(gc, &instruction),
	0b101100 => sth(gc, &instruction),
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
    pub fn pr(&self) -> bool {
	((self.0 >> 14) & 1) != 0
    }

    pub fn ir(&self) -> bool {
	((self.0 >> 5) & 1) != 0
    }

    pub fn dr(&self) -> bool {
	((self.0 >> 4) & 1) != 0
    }
}

pub struct ConditionRegister(pub u32);

impl ConditionRegister {
    pub fn set_reg(&mut self, index: usize, val: u32) {
	self.0 = (self.0 & (!(0xF000_0000 >> (index * 4)))) | (val << ((7 - index) * 4));
    }
}
