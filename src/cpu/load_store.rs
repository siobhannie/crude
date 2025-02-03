use log::debug;

use crate::Gamecube;

use super::{instr::Instruction, util::{dequantized, sext_12}};

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

pub fn psq_l(gc: &mut Gamecube, instr: &Instruction) {
    if !(gc.cpu.hid2.pse() | gc.cpu.hid2.lsqe()) {
	unimplemented!("exception");
    }
    let b = if instr.a() == 0 {
	sext_12(instr.uimm_d()) as u32
    } else {
	gc.cpu.gprs[instr.a()].wrapping_add(sext_12(instr.uimm_d()) as u32)
    };

    let gqr = gc.cpu.gqrs[instr.i()];
    let ld_type = gqr.ld_type();
    let ld_scale = gqr.ld_scale();

    if instr.w() {
	let val = match ld_type {
	    4 | 6 => gc.read_u8(b) as u32,
	    5 | 7 => gc.read_u16(b) as u32,
	    _ => gc.read_u32(b, false),
	};
	
	*gc.cpu.fprs[instr.d()].as_paired_f32_mut() = (dequantized(val, ld_type, ld_scale), 1.);
    } else {
	let (val0, val1) = match ld_type {
	    4 | 6 => (gc.read_u8(b) as u32, gc.read_u8(b + 1) as u32),
	    5 | 7 => (gc.read_u16(b) as u32, gc.read_u16(b + 2) as u32),
	    _ => (gc.read_u32(b, false), gc.read_u32(b + 4, false)),
	};

	*gc.cpu.fprs[instr.d()].as_paired_f32_mut() = (dequantized(val0, ld_type, ld_scale), dequantized(val1, ld_type, ld_scale));
    }
}

pub fn lfd(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);

    *gc.cpu.fprs[instr.d()].as_u64_mut() = gc.read_u64(b);
}
