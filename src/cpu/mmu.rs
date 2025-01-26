use log::debug;

use super::MachineStateRegister;

pub struct Mmu {
    pub dbats: [Bat; 4],
    pub ibats: [Bat; 4],
    pub srs: [u32; 16],
}

impl Mmu {
    pub fn new() -> Self {
	Self {
	    dbats: [Bat(0, 0); 4],
	    ibats: [Bat(0, 0); 4],
	    srs: [0; 16],
	}
    }

    pub fn write_dbatu(&mut self, idx: usize, val: u32) {
	self.dbats[idx].0 = val;
    }

    pub fn write_dbatl(&mut self, idx: usize, val: u32) {
	self.dbats[idx].1 = val;
    }

    pub fn write_ibatu(&mut self, idx: usize, val: u32) {
	self.ibats[idx].0 = val;
    }

    pub fn write_ibatl(&mut self, idx: usize, val: u32) {
	self.ibats[idx].1 = val;
    }

    pub fn translate_addr(&self, instr: bool, addr: u32, msr: &MachineStateRegister) -> u32 {
	let bats = if instr {
	    if !msr.ir() {
		return addr;
	    }
	    &self.ibats
	} else {
	    if !msr.dr() {
		return addr;
	    }
	    &self.dbats
	};

	for bat in bats {
	    let addr_15 = addr >> 17;
	    let addr_bepi = (addr_15 & 0x7800) ^ ((addr_15 & 0x7FF) & (!bat.bl()));

	    if addr_bepi == bat.bepi() && ((!msr.pr() && bat.vs()) || (msr.pr() && bat.vp())) {
		let upper = u32::from(bat.brpn() ^ ((addr_15 & 0x7FF) & bat.bl()));
		let lower = addr & 0x1FFFF;

		return (upper << 17) ^ lower;
	    }
	}

	unimplemented!("page or segment translation :(");
    }
}

#[derive(Debug, Copy, Clone)]
//             batu,    batl
pub struct Bat(pub u32, pub u32);

impl Bat {
    pub fn bepi(&self) -> u32 {
	(self.0 >> 17) & 0x7FFF
    }

    pub fn bl(&self) -> u32 {
	(self.0 >> 2) & 0x7FF
    }

    pub fn vs(&self) -> bool {
	((self.0 >> 1) & 1) != 0
    }

    pub fn vp(&self) -> bool {
	(self.0 & 1) != 0
    }

    pub fn brpn(&self) -> u32 {
	(self.1 >> 17) & 0x7FFF
    }

    pub fn wimg(&self) -> u32 {
	(self.1 >> 3) & 0x1F
    }

    pub fn pp(&self) -> u32 {
	self.1 & 3
    }
}
