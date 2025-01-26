use crate::Gamecube;

use super::{instr::Instruction, util::mask};

pub fn ori(gc: &mut Gamecube, instr: &Instruction) {
    let val = gc.cpu.gprs[instr.s()];
    gc.cpu.gprs[instr.a()] = val | instr.uimm() as u32;
}

pub fn rlwinm(gc: &mut Gamecube, instr: &Instruction) {
    let val = gc.cpu.gprs[instr.s()];
    let rotated = val.rotate_left(instr.sh() as u32);
    gc.cpu.gprs[instr.a()] = rotated & mask(instr.mb(), instr.me());
    if instr.rc() {
	unimplemented!("cr0!");
    }
}
