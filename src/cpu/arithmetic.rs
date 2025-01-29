use std::cmp::Ordering;

use log::debug;

use crate::Gamecube;

use super::instr::Instruction;

pub fn addi(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.d()] = if instr.a() == 0 {
	i32::from(instr.simm()) as u32
    } else {
	gc.cpu.gprs[instr.a()].wrapping_add(i32::from(instr.simm()) as u32)
    }
}

pub fn addis(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.d()] = if instr.a() == 0 {
	(instr.uimm() as u32) << 16
    } else {
	gc.cpu.gprs[instr.a()].wrapping_add((instr.uimm() as u32) << 16)
    };
}

pub fn subf(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.d()] = (!gc.cpu.gprs[instr.a()]).wrapping_add(gc.cpu.gprs[instr.b()]).wrapping_add(1);
    
    if instr.rc() {
	unimplemented!("cr0");
    }

    if instr.oe() {
	unimplemented!("xer");
    }
}

pub fn cmpl(gc: &mut Gamecube, instr: &Instruction) {
    let a = gc.cpu.gprs[instr.a()];
    let b = gc.cpu.gprs[instr.b()];

    let c = match a.cmp(&b) {
	Ordering::Less => 0b1000,
	Ordering::Greater => 0b0100,
	Ordering::Equal => 0b0010,
    };

    //c |= gc.cpu.xer.so()

    gc.cpu.cr.set_reg(instr.crd(), c);
}

pub fn cmpli(gc: &mut Gamecube, instr: &Instruction) {
    let a = gc.cpu.gprs[instr.a()];
    let imm = instr.uimm() as u32;

    let c = match a.cmp(&imm) {
	Ordering::Less => 0b1000,
	Ordering::Greater => 0b0100,
	Ordering::Equal => 0b0010,
    };

    //c |= gc.cpu.xer.so()

    gc.cpu.cr.set_reg(instr.crd(), c);
}

pub fn cmpi(gc: &mut Gamecube, instr: &Instruction) {
    let a = gc.cpu.gprs[instr.a()] as i32;
    let imm = i32::from(instr.simm());

    let c = match a.cmp(&imm) {
	Ordering::Less => 0b1000,
	Ordering::Greater => 0b0100,
	Ordering::Equal => 0b0010,
    };

    //c |= gc.cpu.xer.so()

    gc.cpu.cr.set_reg(instr.crd(), c);
    
}
