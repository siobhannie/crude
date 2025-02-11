use std::{cmp::Ordering, ops::Mul};

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
    let (r1, c1) = (!gc.cpu.gprs[instr.a()]).overflowing_add(gc.cpu.gprs[instr.b()]);
    let (r, c2) = r1.overflowing_add(1);

    gc.cpu.gprs[instr.d()] = r;

    if instr.rc() {
	gc.cpu.do_cr0(r);
    }

    if instr.oe() {
	unimplemented!("xer");
    }
}

pub fn subfc(gc: &mut Gamecube, instr: &Instruction) {
    let (r1, c1) = (!gc.cpu.gprs[instr.a()]).overflowing_add(gc.cpu.gprs[instr.b()]);
    let (r, c2) = r1.overflowing_add(1);

    gc.cpu.gprs[instr.d()] = r;

    gc.cpu.xer.set_ca(c1 | c2);
    
    if instr.rc() {
	gc.cpu.do_cr0(r);
    }

    if instr.oe() {
	unimplemented!("xer");
    }
}

pub fn subfic(gc: &mut Gamecube, instr: &Instruction) {
    let simm = ((instr.simm() as i32) as u32);
    let (r1, c1) = (!gc.cpu.gprs[instr.a()]).overflowing_add(simm);
    let (r, c2) = r1.overflowing_add(1);

    gc.cpu.gprs[instr.d()] = r;

    gc.cpu.xer.set_ca(c1 | c2);

    if instr.rc() {
	gc.cpu.do_cr0(r);
    }

    if instr.oe() {
	unimplemented!("xer");
    }
}

pub fn subfe(gc: &mut Gamecube, instr: &Instruction) {
    let (r1, c1) = (!gc.cpu.gprs[instr.a()]).overflowing_add(gc.cpu.gprs[instr.b()]);
    let (r, c2) = r1.overflowing_add(gc.cpu.xer.ca() as u32);

    gc.cpu.gprs[instr.d()] = r;

    gc.cpu.xer.set_ca(c1 | c2);
    
    if instr.rc() {
	gc.cpu.do_cr0(r);
    }

    if instr.oe() {
	unimplemented!("xer");
    }    
}

pub fn mullw(gc: &mut Gamecube, instr: &Instruction) {
    let a = (gc.cpu.gprs[instr.a()] as i32) as i64;
    let b = (gc.cpu.gprs[instr.b()] as i32) as i64;

    let r = a.wrapping_mul(b);

    gc.cpu.gprs[instr.d()] = r as u32;

    if instr.oe() {
	gc.cpu.xer.set_ov((r < -0x80000000) | (r > 0x7FFFFFFF));
    }

    if instr.rc() {
	gc.cpu.do_cr0(r as u32);
    }
}

pub fn mulli(gc: &mut Gamecube, instr: &Instruction) {
    let a = (gc.cpu.gprs[instr.a()] as i32) as i64;
    let i = (instr.simm() as i32) as i64;
    gc.cpu.gprs[instr.d()] = a.wrapping_mul(i) as u32;
}

pub fn mulhwu(gc: &mut Gamecube, instr: &Instruction) {
    let a = gc.cpu.gprs[instr.a()] as u64;
    let b = gc.cpu.gprs[instr.b()] as u64;

    let r = (a.wrapping_mul(b) >> 32) as u32;

    gc.cpu.gprs[instr.d()] = r;

    if instr.rc() {
	gc.cpu.do_cr0(r);
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

pub fn neg(gc: &mut Gamecube, instr: &Instruction) {
    let r = (!gc.cpu.gprs[instr.a()]).wrapping_add(1);

    gc.cpu.gprs[instr.d()] = r;

    if instr.oe() {
	gc.cpu.xer.set_so(r == 0x8000_0000);
    }

    if instr.rc() {
	gc.cpu.do_cr0(r);
    }
}

pub fn cntlzw(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.a()] = gc.cpu.gprs[instr.s()].leading_zeros();

    if instr.rc() {
	gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
    }
}
