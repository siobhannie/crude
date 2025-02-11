use std::ops::{Shl, Shr};

use crate::Gamecube;

use super::{instr::Instruction, util::mask};

pub fn ori(gc: &mut Gamecube, instr: &Instruction) {
    let val = gc.cpu.gprs[instr.s()];
    gc.cpu.gprs[instr.a()] = val | instr.uimm() as u32;
}

pub fn oris(gc: &mut Gamecube, instr: &Instruction) {
    let val = gc.cpu.gprs[instr.s()];
    gc.cpu.gprs[instr.a()] = val | ((instr.uimm() as u32) << 16);
}

pub fn rlwinm(gc: &mut Gamecube, instr: &Instruction) {
    let val = gc.cpu.gprs[instr.s()];
    let rotated = val.rotate_left(instr.sh() as u32);
    gc.cpu.gprs[instr.a()] = rotated & mask(instr.mb(), instr.me());
    if instr.rc() {
	gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
    }
}

pub fn rlwimi(gc: &mut Gamecube, instr: &Instruction) {
    let r = gc.cpu.gprs[instr.s()].rotate_left(instr.sh() as u32);
    let m = mask(instr.mb(), instr.me());
    gc.cpu.gprs[instr.a()] = (r & m) | (gc.cpu.gprs[instr.a()] & !m);
    if instr.rc() {
	gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
    }
}

pub fn or(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.a()] = gc.cpu.gprs[instr.s()] | gc.cpu.gprs[instr.b()];

    if instr.rc() {
	gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
    }
}

pub fn nor(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.a()] = !(gc.cpu.gprs[instr.s()] | gc.cpu.gprs[instr.b()]);

    if instr.rc() {
	gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
    }
}

pub fn and(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.a()] = gc.cpu.gprs[instr.s()] & gc.cpu.gprs[instr.b()];

    if instr.rc() {
	gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
    }
}

pub fn andi(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.a()] = gc.cpu.gprs[instr.s()] & (instr.uimm() as u32);

    gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
}

pub fn xor(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.a()] = gc.cpu.gprs[instr.s()] ^ gc.cpu.gprs[instr.b()];

    if instr.rc() {
	gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
    }
}

pub fn crxor(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.cr.set_reg(instr.d(), (gc.cpu.cr.get_reg(instr.a()) as u32) ^ (gc.cpu.cr.get_reg(instr.b()) as u32));
}

pub fn slw(gc: &mut Gamecube, instr: &Instruction) {
    let n = gc.cpu.gprs[instr.b()] & 0x1F;
    let r = gc.cpu.gprs[instr.s()].shl(n);
    gc.cpu.gprs[instr.a()] = r;
    if instr.rc() {
	gc.cpu.do_cr0(r);
    }
}

pub fn srw(gc: &mut Gamecube, instr: &Instruction) {
    let n = gc.cpu.gprs[instr.b()] & 0x1f;
    let r = gc.cpu.gprs[instr.s()].shr(n);
    gc.cpu.gprs[instr.a()] = r;
    if instr.rc() {
	gc.cpu.do_cr0(r);
    }
}

pub fn sraw(gc: &mut Gamecube, instr: &Instruction) {
    let b = gc.cpu.gprs[instr.b()];

    if (b & 0x20) != 0 {
	if (gc.cpu.gprs[instr.s()] & 0x80000000) != 0 {
	    gc.cpu.gprs[instr.a()] = 0xFFFFFFFF;
	    gc.cpu.xer.set_ca(true);
	} else {
	    gc.cpu.gprs[instr.a()] = 0x00000000;
	    gc.cpu.xer.set_ca(false);
	}
    } else {
	let n = b & 0x1f;
	let s = gc.cpu.gprs[instr.s()] as i32;
	gc.cpu.gprs[instr.a()] = (s >> n) as u32;

	gc.cpu.xer.set_ca(s < 0 && n > 0  && ((s as u32) << (32 - n)) != 0);
    }

    gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
}

pub fn andc(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.a()] = gc.cpu.gprs[instr.s()] & !gc.cpu.gprs[instr.b()];

    if instr.rc() {
	gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
    }
}

pub fn extsh(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.a()] = ((gc.cpu.gprs[instr.s()] as i16) as i32) as u32;

    if instr.rc() {
	gc.cpu.do_cr0(gc.cpu.gprs[instr.a()]);
    }
}

pub fn xoris(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.a()] = gc.cpu.gprs[instr.s()] ^ ((instr.uimm() as u32) << 16)
}
