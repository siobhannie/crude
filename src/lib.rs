#![feature(core_intrinsics)]
#![feature(bigint_helper_methods)]

use std::{fs::File, io::Write, sync::{Arc, RwLock}};

use audio_interface::{ai_read_u32, ai_write_u16, ai_write_u32, AudioInterface};
use byteorder::{BigEndian, ByteOrder};
use cpu::Cpu;
use dvd_interface::di_read_u32;
use external_interface::{exi_read_u32, exi_write_u32, ExternalInterface};
use memory_interface::{mi_write_u16, MemoryInterface};
use processor_interface::{pi_read_u32, pi_write_u32};
use serial_interface::{si_read_u32, si_write_u32, SerialInterface};
use sram::Sram;
use video_interface::vi_read_u16;

pub mod cpu;
pub mod audio_interface;
pub mod memory_interface;
pub mod processor_interface;
pub mod serial_interface;
pub mod external_interface;
pub mod video_interface;
pub mod dvd_interface;
pub mod sram;
pub mod dsp;

pub struct Gamecube {
    pub cpu: Cpu,
    pub bios: Vec<u8>,
    pub exi: ExternalInterface,
    pub si: SerialInterface,
    pub mi: MemoryInterface,
    pub ai: AudioInterface,
    pub sram: Arc<RwLock<Sram>>,
    pub memory: Vec<u8>,
}

impl Gamecube {
    pub fn new(bios: Vec<u8>) -> Self {
	let mut bios = bios;
	descramble(&mut bios[0x100..0x1AFF00]);
	let sram = Arc::new(RwLock::new(Sram::new()));
	Self {
	    cpu: Cpu::new(),
	    bios: bios.clone(),
	    exi: ExternalInterface::new(bios, sram.clone()),
	    si: SerialInterface::new(),
	    mi: MemoryInterface::new(),
	    ai: AudioInterface::new(),
	    sram,
	    memory: vec![0; 0x180_0000],
	}
    }

    pub fn read_u8(&mut self, addr: u32) -> u8 {
	let phys = self.cpu.mmu.translate_addr(false, addr, &self.cpu.msr);

	match phys {
	    0x0000_0000..=0x017F_FFFF => self.memory[phys as usize],
	    _ => unimplemented!("addr {phys:#010X} for read_u8"),
	}
    }

    pub fn read_u16(&mut self, addr: u32) -> u16 {
	let phys = self.cpu.mmu.translate_addr(false, addr, &self.cpu.msr);

	match phys {
	    0x0000_0000..=0x017F_FFFF => BigEndian::read_u16(&self.memory[(phys as usize)..]),
	    0x0C00_2000..=0x0C00_2FFF => vi_read_u16(self, phys - 0x0C00_2000),
	    _ => unimplemented!("addr {phys:#010X} for read_u16"),
	}
    }
    
    pub fn read_u32(&mut self, addr: u32, instr: bool) -> u32 {
	let phys = self.cpu.mmu.translate_addr(instr, addr, &self.cpu.msr);
	
	match phys {
	    0x0000_0000..=0x017F_FFFF => BigEndian::read_u32(&self.memory[(phys as usize)..]),
	    0x0C00_3000..=0x0C00_3FFF => pi_read_u32(self, phys - 0x0C00_3000),
	    0x0C00_6000..=0x0C00_63FF => di_read_u32(self, phys - 0x0C00_6000),
	    0x0C00_6400..=0x0C00_67FF => si_read_u32(self, phys - 0x0C00_6400),
	    0x0C00_6800..=0x0C00_6BFF => exi_read_u32(self, phys - 0x0C00_6800),
	    0x0C00_6C00..=0x0C00_6FFF => ai_read_u32(self, phys - 0x0C00_6C00),
	    0xFFF0_0000..=0xFFFF_FFFF => BigEndian::read_u32(&self.bios[(phys as usize - 0xFFF0_0000)..]),
	    _ => unimplemented!("addr {phys:#010X} for read_u32"),
	}
    }

    pub fn read_u64(&mut self, addr: u32) -> u64 {
	let phys = self.cpu.mmu.translate_addr(false, addr, &self.cpu.msr);

	match phys {
	    0x0000_0000..=0x017F_FFFF => BigEndian::read_u64(&self.memory[(phys as usize)..]),
	    _ => unimplemented!("addr {phys:#010X} for read_u64"),
	}
    }

    pub fn write_u8(&mut self, addr: u32, val: u8) {
	let phys = self.cpu.mmu.translate_addr(false, addr, &self.cpu.msr);

	match phys {
	    0x0000_0000..=0x017F_FFFF => self.memory[phys as usize] = val,
	    _ => unimplemented!("addr {phys:#010X} for write_u8"),
	}
    }

    pub fn write_u16(&mut self, addr: u32, val: u16) {
	let phys = self.cpu.mmu.translate_addr(false, addr, &self.cpu.msr);
	
	match phys {
	    0x0000_0000..=0x017F_FFFF => BigEndian::write_u16(&mut self.memory[(phys as usize)..], val),
	    0x0C00_4000..=0x0C00_4FFF => mi_write_u16(self, phys - 0x0C00_4000, val),
	    0x0C00_5000..=0x0C00_5FFF => ai_write_u16(self, phys - 0x0C00_5000, val),
	    _ => unimplemented!("addr {phys:#010X} for write_u16"),
	}
    }

    pub fn write_u32(&mut self, addr: u32, val: u32) {
	let phys = self.cpu.mmu.translate_addr(false, addr, &self.cpu.msr);

	match phys {
	    0x0000_0000..=0x017F_FFFF => BigEndian::write_u32(&mut self.memory[(phys as usize)..], val),
	    0x0C00_3000..=0x0C00_3FFF => pi_write_u32(self, phys - 0x0C00_3000, val),
	    0x0C00_6400..=0x0C00_67FF => si_write_u32(self, phys - 0x0C00_6400, val),
	    0x0C00_6800..=0x0C00_6BFF => exi_write_u32(self, phys - 0x0C00_6800, val),
	    0x0C00_6C00..=0x0C00_6FFF => ai_write_u32(self, phys - 0x0C00_6C00, val),
	    _ => unimplemented!("addr {phys:#010X} for write_u32"),
	}
    }

    pub fn write_u64(&mut self, addr: u32, val: u64) {
	let phys = self.cpu.mmu.translate_addr(false, addr, &self.cpu.msr);

	match phys {
	    0x0000_0000..=0x017F_FFFF => BigEndian::write_u64(&mut self.memory[(phys as usize)..], val),
	    _ => unimplemented!("addr {phys:#010X} for write_u64"),
	}
    }
}

pub fn step(gc: &mut Gamecube) {
    cpu::step(gc);
}

pub fn run(gc: &mut Gamecube) {
    loop {
	step(gc);
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
