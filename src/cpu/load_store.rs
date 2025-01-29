use log::debug;

use crate::Gamecube;

use super::instr::Instruction;

fn b(gc: &mut Gamecube, instr: &Instruction) -> u32 {
    ((if instr.a() == 0 {
	0
    } else {
	gc.cpu.gprs[instr.a()]
    } as i32) + instr.simm() as i32) as u32
}

pub fn sth(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    gc.write_u16(b, (gc.cpu.gprs[instr.s()] & 0xFFFF) as u16);
}

pub fn stw(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    gc.write_u32(b, gc.cpu.gprs[instr.s()]);
}

pub fn stwu(gc: &mut Gamecube, instr: &Instruction) {
    let b = ((gc.cpu.gprs[instr.a()] as i32) + instr.simm() as i32) as u32;
    gc.write_u32(b, gc.cpu.gprs[instr.s()]);
    gc.cpu.gprs[instr.a()] = b;
}

pub fn lwz(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    gc.cpu.gprs[instr.d()] = gc.read_u32(b, false);
}
