use log::debug;

use crate::Gamecube;

use super::{instr::Instruction, util::{convert_to_double, convert_to_single, dequantized, sext_12}, PROGRAM_EXCEPTION};

fn b(gc: &mut Gamecube, instr: &Instruction) -> u32 {
    ((if instr.a() == 0 {
	0
    } else {
	gc.cpu.gprs[instr.a()]
    } as i32).wrapping_add(instr.simm() as i32)) as u32
}

pub fn stb(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    gc.write_u8(b, (gc.cpu.gprs[instr.s()] & 0xFF) as u8)
}

pub fn stbu(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    gc.write_u8(b, (gc.cpu.gprs[instr.s()] & 0xFF) as u8);
    gc.cpu.gprs[instr.a()] = b;
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
    let b = ((gc.cpu.gprs[instr.a()] as i32).wrapping_add(instr.simm() as i32)) as u32;
    gc.write_u32(b, gc.cpu.gprs[instr.s()]);
    gc.cpu.gprs[instr.a()] = b;
}

pub fn stwx(gc: &mut Gamecube, instr: &Instruction) {
    let b = if instr.a() == 0 {
	0
    } else {
	gc.cpu.gprs[instr.a()]
    }.wrapping_add(gc.cpu.gprs[instr.b()]);
    gc.write_u32(b, gc.cpu.gprs[instr.s()]);
}

pub fn stfs(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    gc.write_u32(b, convert_to_single(*gc.cpu.fprs[instr.s()].as_u64()));
}

pub fn stfsu(gc: &mut Gamecube, instr: &Instruction) {
    let b = gc.cpu.gprs[instr.a()].wrapping_add(gc.cpu.gprs[instr.b()]);
    gc.write_u32(b, convert_to_single(*gc.cpu.fprs[instr.s()].as_u64()));
    gc.cpu.gprs[instr.a()] = b;
}

pub fn lbz(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    gc.cpu.gprs[instr.d()] = gc.read_u8(b) as u32;
}

pub fn lbzu(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    gc.cpu.gprs[instr.d()] = gc.read_u8(b) as u32;
    gc.cpu.gprs[instr.a()] = b;
}

pub fn lhz(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    gc.cpu.gprs[instr.d()] = gc.read_u16(b) as u32;
}

pub fn lhzu(gc: &mut Gamecube, instr: &Instruction) {
    let b = (gc.cpu.gprs[instr.a()] as i32).wrapping_add(instr.simm() as i32) as u32;
    gc.cpu.gprs[instr.d()] = gc.read_u16(b) as u32;
    gc.cpu.gprs[instr.a()] = b;
}

pub fn lwz(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    gc.cpu.gprs[instr.d()] = gc.read_u32(b, false);
}

pub fn lwzu(gc: &mut Gamecube, instr: &Instruction) {
    if instr.a() == 0 || instr.a() == instr.d() {
	gc.cpu.exceptions |= PROGRAM_EXCEPTION;
    }
    let b = (gc.cpu.gprs[instr.a()] as i32).wrapping_add(instr.simm() as i32) as u32;
    gc.cpu.gprs[instr.d()] = gc.read_u32(b, false);
    gc.cpu.gprs[instr.a()] = b;
}

pub fn lfs(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);
    let val = gc.cpu.gprs[instr.a()];

    if !gc.cpu.hid2.pse() {
	*gc.cpu.fprs[instr.d()].as_u64_mut() = convert_to_double(val);
    } else {
	*gc.cpu.fprs[instr.d()].as_paired_u32_mut() = (val, val)
    }
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

pub fn stfd(gc: &mut Gamecube, instr: &Instruction) {
    let b = b(gc, instr);

    gc.write_u64(b, *gc.cpu.fprs[instr.s()].as_u64());
}

pub fn stmw(gc: &mut Gamecube, instr: &Instruction) {
    let mut b = b(gc, instr);

    let mut r = instr.s();

    while r <= 31 {
	gc.write_u32(b, gc.cpu.gprs[r]);
	r += 1;
	b += 4;
    }
}

pub fn lmw(gc: &mut Gamecube, instr: &Instruction) {
    let mut b = b(gc, instr);

    let mut r = instr.d();

    while r <= 31 {
	gc.cpu.gprs[r] = gc.read_u32(b, false);
	r += 1;
	b += 4;
    }
}

pub fn lhzx(gc: &mut Gamecube, instr: &Instruction) {
    let b = if instr.a() == 0 {
	0
    } else {
	gc.cpu.gprs[instr.a()]
    }.wrapping_add(gc.cpu.gprs[instr.b()]);

    gc.cpu.gprs[instr.d()] = gc.read_u16(b) as u32;
}

pub fn lwzx(gc: &mut Gamecube, instr: &Instruction) {
    let b = if instr.a() == 0 {
	0
    } else {
	gc.cpu.gprs[instr.a()]
    }.wrapping_add(gc.cpu.gprs[instr.b()]);

    gc.cpu.gprs[instr.d()] = gc.read_u32(b, false);    
}
