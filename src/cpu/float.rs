use crate::{cpu::instr::Instruction, Gamecube};

pub fn ps_mr(gc: &mut Gamecube, instr: &Instruction) {
    if !gc.cpu.hid2.pse() {
	unimplemented!("exception");
    }
    gc.cpu.fprs[instr.d()] = gc.cpu.fprs[instr.b()];
    if instr.rc() {
	unimplemented!("cr1");
    }
}

pub fn fmr(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.fprs[instr.d()] = gc.cpu.fprs[instr.b()];
    if instr.rc() {
	unimplemented!("cr1");
    }
}
