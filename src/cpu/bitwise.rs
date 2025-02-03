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
	unimplemented!("cr0!");
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
