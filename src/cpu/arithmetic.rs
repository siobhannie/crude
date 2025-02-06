use std::cmp::Ordering;

use log::debug;

use crate::Gamecube;

use super::instr::Instruction;

pub fn add(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.d()] = gc.cpu.gprs[instr.a()].wrapping_add(gc.cpu.gprs[instr.b()]);

    if instr.rc() {
	gc.cpu.do_cr0(gc.cpu.gprs[instr.d()]);
    }

    if instr.oe() {
	unimplemented!("xer");
    }
}

pub fn addi(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.d()] = if instr.a() == 0 {
	i32::from(instr.simm()) as u32
    } else {
	gc.cpu.gprs[instr.a()].wrapping_add(i32::from(instr.simm()) as u32)
    }
}

pub fn addc(gc: &mut Gamecube, instr: &Instruction) {
    let (result, carry) = gc.cpu.gprs[instr.a()].overflowing_add(gc.cpu.gprs[instr.b()]);
    gc.cpu.gprs[instr.d()] = result;
    if instr.rc() {
	gc.cpu.do_cr0(result);
    }
    gc.cpu.xer.set_ca(carry);
    if instr.oe() {
	unimplemented!("xer");
    }
}

pub fn adde(gc: &mut Gamecube, instr: &Instruction) {
    let (result, carry) = gc.cpu.gprs[instr.a()].carrying_add(gc.cpu.gprs[instr.b()], gc.cpu.xer.ca());
    gc.cpu.gprs[instr.d()] = result;
    if instr.rc() {
	gc.cpu.do_cr0(result);
    }
    gc.cpu.xer.set_ca(carry);
    if instr.oe() {
	unimplemented!("xer");
    }    
}

pub fn addis(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.d()] = if instr.a() == 0 {
	(instr.uimm() as u32) << 16
    } else {
	gc.cpu.gprs[instr.a()].wrapping_add((instr.uimm() as u32) << 16)
    };
}

pub fn addic(gc: &mut Gamecube, instr: &Instruction) {
    let (result, carry) = gc.cpu.gprs[instr.a()].overflowing_add(i32::from(instr.simm()) as u32);
    gc.cpu.gprs[instr.d()] = result;
    gc.cpu.xer.set_ca(carry);
}

pub fn addicr(gc: &mut Gamecube, instr: &Instruction) {
    let (result, carry) = gc.cpu.gprs[instr.a()].overflowing_add(i32::from(instr.simm()) as u32);
    gc.cpu.gprs[instr.d()] = result;
    gc.cpu.do_cr0(result);
    gc.cpu.xer.set_ca(carry);
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

pub fn cmp(gc: &mut Gamecube, instr: &Instruction) {
    let a = gc.cpu.gprs[instr.a()] as i32;
    let b = gc.cpu.gprs[instr.b()] as i32;

    let mut c = match a.cmp(&b) {
	Ordering::Less => 0b1000,
	Ordering::Greater => 0b0100,
	Ordering::Equal => 0b0010,
    };

    c |= gc.cpu.xer.so() as u32;

    gc.cpu.cr.set_reg(instr.crd(), c);
}

pub fn cmpl(gc: &mut Gamecube, instr: &Instruction) {
    let a = gc.cpu.gprs[instr.a()];
    let b = gc.cpu.gprs[instr.b()];

    let mut c = match a.cmp(&b) {
	Ordering::Less => 0b1000,
	Ordering::Greater => 0b0100,
	Ordering::Equal => 0b0010,
    };

    c |= gc.cpu.xer.so() as u32;

    gc.cpu.cr.set_reg(instr.crd(), c);
}

pub fn cmpli(gc: &mut Gamecube, instr: &Instruction) {
    let a = gc.cpu.gprs[instr.a()];
    let imm = instr.uimm() as u32;

    let mut c = match a.cmp(&imm) {
	Ordering::Less => 0b1000,
	Ordering::Greater => 0b0100,
	Ordering::Equal => 0b0010,
    };

    c |= gc.cpu.xer.so() as u32;

    gc.cpu.cr.set_reg(instr.crd(), c);
}

pub fn cmpi(gc: &mut Gamecube, instr: &Instruction) {
    let a = gc.cpu.gprs[instr.a()] as i32;
    let imm = i32::from(instr.simm());

    let mut c = match a.cmp(&imm) {
	Ordering::Less => 0b1000,
	Ordering::Greater => 0b0100,
	Ordering::Equal => 0b0010,
    };

    c |= gc.cpu.xer.so() as u32;

    gc.cpu.cr.set_reg(instr.crd(), c);
    
}
