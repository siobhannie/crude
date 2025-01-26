use crate::Gamecube;

use super::{instr::Instruction, write_hid0, write_msr};

pub fn mtspr(gc: &mut Gamecube, instr: &Instruction) {
    let val = gc.cpu.gprs[instr.s()];
    let spr = instr.spr();

    match spr {
	0b10000_10000 => gc.cpu.mmu.write_ibatu(0, val),
	0b10000_10001 => gc.cpu.mmu.write_ibatl(0, val),
	0b10000_10010 => gc.cpu.mmu.write_ibatu(1, val),
	0b10000_10011 => gc.cpu.mmu.write_ibatl(1, val),
	0b10000_10100 => gc.cpu.mmu.write_ibatu(2, val),
	0b10000_10101 => gc.cpu.mmu.write_ibatl(2, val),
	0b10000_10110 => gc.cpu.mmu.write_ibatu(3, val),
	0b10000_10111 => gc.cpu.mmu.write_ibatl(3, val),	
	0b10000_11000 => gc.cpu.mmu.write_dbatu(0, val),
	0b10000_11001 => gc.cpu.mmu.write_dbatl(0, val),
	0b10000_11010 => gc.cpu.mmu.write_dbatu(1, val),
	0b10000_11011 => gc.cpu.mmu.write_dbatl(1, val),
	0b10000_11100 => gc.cpu.mmu.write_dbatu(2, val),
	0b10000_11101 => gc.cpu.mmu.write_dbatl(2, val),
	0b10000_11110 => gc.cpu.mmu.write_dbatu(3, val),
	0b10000_11111 => gc.cpu.mmu.write_dbatl(3, val),	
	0b11111_10000 => gc.cpu.hid0 = val,
	a => unimplemented!("mtspr {a:#012b}, instruction: {:#034b}", instr.0),
    }
}

pub fn mtsr(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.mmu.srs[instr.sr()] = gc.cpu.gprs[instr.s()];
}

pub fn mfspr(gc: &mut Gamecube, instr: &Instruction) {
    let spr = instr.spr();

    gc.cpu.gprs[instr.d()] = match spr {
	0b11111_10000 => gc.cpu.hid0,
	a => unimplemented!("mfspr {a:#012b}, instruction {:#034b}", instr.0),
    };
}

pub fn mftb(gc: &mut Gamecube, instr: &Instruction) {
    let reg = instr.tbr();

    gc.cpu.gprs[instr.d()] = match reg {
	268 => gc.cpu.tb.0, //TBL
	269 => gc.cpu.tb.1, //TBU
	_ => unimplemented!("tbl {reg}"),
    }
}

pub fn mfmsr(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.gprs[instr.d()] = gc.cpu.msr.0;
}

pub fn mtmsr(gc: &mut Gamecube, instr: &Instruction) {
    let val = gc.cpu.gprs[instr.s()];

    write_msr(gc, val);
}
