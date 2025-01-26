use audio_interface::ai_write_u16;
use byteorder::{BigEndian, ByteOrder};
use cpu::Cpu;
use log::debug;
use memory_interface::mi_write_u16;
use processor_interface::{pi_read_u32, pi_write_u32};

pub mod cpu;
pub mod audio_interface;
pub mod memory_interface;
pub mod processor_interface;

pub struct Gamecube {
    cpu: Cpu,
    bios: Vec<u8>,
}

impl Gamecube {
    pub fn new() -> Self {
	Self {
	    cpu: Cpu::new(),
	    bios: Vec::new(),
	}
    }
    
    pub fn load_bios(&mut self, bios: Vec<u8>) {
	self.bios = bios;

	descramble(&mut self.bios[0x100..0x1AFF00]);
    }

    pub fn read_u32(&mut self, addr: u32, instr: bool) -> u32 {
	let phys = self.cpu.mmu.translate_addr(instr, addr, &self.cpu.msr);
	
	match phys {
	    0x0C00_3000..=0x0C00_3FFF => pi_read_u32(self, phys - 0x0C00_3000),
	    0xFFF0_0000..=0xFFFF_FFFF => BigEndian::read_u32(&self.bios[(phys as usize - 0xFFF0_0000)..]),
	    _ => unimplemented!("addr {phys:#010X} for read_u32"),
	}
    }

    pub fn write_u16(&mut self, addr: u32, val: u16) {
	let phys = self.cpu.mmu.translate_addr(false, addr, &self.cpu.msr);
	
	match phys {
	    0x0C00_4000..=0x0C00_4FFF => mi_write_u16(self, phys - 0x0C00_4000, val),
	    0x0C00_5000..=0x0C00_5FFF => ai_write_u16(self, phys - 0x0C00_5000, val),
	    _ => unimplemented!("addr {phys:#010X} for write_u16"),
	}
    }

    pub fn write_u32(&mut self, addr: u32, val: u32) {
	let phys = self.cpu.mmu.translate_addr(false, addr, &self.cpu.msr);

	match phys {
	    0x0C00_3000..=0x0C00_3FFF => pi_write_u32(self, phys - 0x0C00_3000, val),
	    _ => unimplemented!("addr {phys:#010X} for write_u32"),
	}
    }
}

pub fn run(gc: &mut Gamecube) {
    loop {
	cpu::step(gc);
    }
}

//translated from dolphin :3
//https://github.com/dolphin-emu/dolphin/blob/master/Source/Core/Core/HW/EXI/EXI_DeviceIPL.cpp
// bootrom descrambler reversed by segher
// Copyright 2008 Segher Boessenkool <segher@kernel.crashing.org>
fn descramble(data: &mut [u8]) {
    let size = data.len();
    let mut acc: u8 = 0;
    let mut nacc: u8 = 0;

    let mut t: u16 = 0x2953;
    let mut u: u16 = 0xd9c2;
    let mut v: u16 = 0x3ff1;

    let mut x: u8 = 1;

    let mut it = 0;

    while it < size {
        let t0 = t & 1;
        let t1 = (t >> 1) & 1;
        let u0 = u & 1;
        let u1 = (u >> 1) & 1;
        let v0 = v & 1;

        x ^= (t1 ^ v0) as u8;
        x ^= (u0 | u1) as u8;
        x ^= ((t0 ^ u1 ^ v0) & (t0 ^ u0)) as u8;

        if t0 == u0 {
            v >>= 1;
            if v0 != 0 {
                v ^= 0xb3d0;
            }
        }

        if t0 == 0 {
            u >>= 1;
            if u0 != 0 {
                u ^= 0xfb10;
            }
        }

        t >>= 1;
        if t0 != 0 {
            t ^= 0xa740;
        }

        nacc += 1;
        acc = (2 * u16::from(acc) + u16::from(x)) as u8;
        if nacc == 8 {
            data[it] ^= acc;
            it += 1;
            nacc = 0;
        }
    }
}
