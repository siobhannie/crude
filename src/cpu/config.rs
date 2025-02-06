use crate::Gamecube;

use super::{instr::Instruction, write_hid0, write_msr};

pub fn mtspr(gc: &mut Gamecube, instr: &Instruction) {
    let val = gc.cpu.gprs[instr.s()];
    let spr = instr.spr();

    match spr {
	0b00000_01000 => gc.cpu.lr = val,
	0b00000_01001 => gc.cpu.ctr = val,
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
	0b11100_10000 => gc.cpu.gqrs[0].0 = val,
	0b11100_10001 => gc.cpu.gqrs[1].0 = val,
	0b11100_10010 => gc.cpu.gqrs[2].0 = val,
	0b11100_10011 => gc.cpu.gqrs[3].0 = val,
	0b11100_10100 => gc.cpu.gqrs[4].0 = val,
	0b11100_10101 => gc.cpu.gqrs[5].0 = val,
	0b11100_10110 => gc.cpu.gqrs[6].0 = val,
	0b11100_10111 => gc.cpu.gqrs[7].0 = val,
	0b11100_11000 => gc.cpu.hid2.0 = val,
	0b11100_11001 => gc.cpu.wpar = val,
	0b11101_11000 => gc.cpu.mmcr0 = val,
	0b11101_11001 => gc.cpu.pmcs[0] = val,
	0b11101_11010 => gc.cpu.pmcs[1] = val,
	0b11101_11100 => gc.cpu.mmcr1 = val,
	0b11101_11101 => gc.cpu.pmcs[2] = val,
	0b11101_11110 => gc.cpu.pmcs[3] = val,
	0b11111_11001 => gc.cpu.l2cr = val,
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
	0b00000_01000 => gc.cpu.lr,
	0b11100_10000 => gc.cpu.gqrs[0].0,
	0b11100_10001 => gc.cpu.gqrs[1].0,
	0b11100_10010 => gc.cpu.gqrs[2].0,
	0b11100_10011 => gc.cpu.gqrs[3].0,
	0b11100_10100 => gc.cpu.gqrs[4].0,
	0b11100_10101 => gc.cpu.gqrs[5].0,
	0b11100_10110 => gc.cpu.gqrs[6].0,
	0b11100_10111 => gc.cpu.gqrs[7].0,
	0b11100_11000 => gc.cpu.hid2.0,
	0b11100_11001 => gc.cpu.wpar,
	0b11101_11000 => gc.cpu.mmcr0,
	0b11101_11001 => gc.cpu.pmcs[0],
	0b11101_11010 => gc.cpu.pmcs[1],
	0b11101_11100 => gc.cpu.mmcr1,
	0b11101_11101 => gc.cpu.pmcs[2],
	0b11101_11110 => gc.cpu.pmcs[3],
	0b11111_11001 => gc.cpu.l2cr,
	0b11111_10000 => gc.cpu.hid0,
	a => unimplemented!("mfspr {a:#012b}, instruction {:#034b}", instr.0),
    };
}

pub fn mftb(gc: &mut Gamecube, instr: &Instruction) {
    let reg = instr.tbr();

    gc.cpu.gprs[instr.d()] = match reg {
	268 => (gc.cpu.tb & 0xFFFFFFFF) as u32, //TBL
	269 => ((gc.cpu.tb >> 32) & 0xFFFFFFFF) as u32, //TBU
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

pub fn mffs(gc: &mut Gamecube, instr: &Instruction) {
    gc.cpu.fprs[instr.d()].as_paired_u32_mut().0 = gc.cpu.fpscr.0;

    if instr.rc() {
	unimplemented!("cr1");
    }
}

pub fn mtfsf(gc: &mut Gamecube, instr: &Instruction) {
    let fm = instr.fm();
    let mut m = 0u32;

    for i in 0..8 {
	if (fm & (1 << i)) != 0 {
	    m |= 0xF << (i * 4);
	}
    }

    gc.cpu.fpscr.0 = (gc.cpu.fpscr.0 & (!m)) | (gc.cpu.fprs[instr.b()].as_paired_u32().0 & m);

    if instr.rc() {
	unimplemented!("cr1");
    }
}
