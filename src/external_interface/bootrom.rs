use std::sync::Arc;

use super::EXIDevice;

pub struct Bootrom {
    rom: Vec<u8>,
    command_bytes_received: u32,
    command: u32,
    cursor: u32,
}

impl Bootrom {
    pub fn new(rom: Vec<u8>) -> Self {
	Self {
	    rom,
	    command_bytes_received: 0,
	    command: 0,
	    cursor: 0,
	}
    }
    
    fn get_address(&self) -> u32 {
	(self.command >> 6) & 0x1ffffff
    }

    fn is_write(&self) -> bool {
	((self.command >> 31) & 1) != 0
    }
}

impl EXIDevice for Bootrom {
    fn transfer_byte(&mut self, byte: &mut u8) {
	if self.command_bytes_received < 4 {
	    self.command <<= 8;
	    self.command |= (*byte) as u32;
	    *byte = 0xff;
	    self.command_bytes_received += 1;
	} else {
	    let addr = self.get_address();
	    if addr < 0x00200000 {
		if !self.is_write() {
		    *byte = self.rom[(addr + self.cursor) as usize];
		    self.cursor += 1;
		}
	    } else {
		unimplemented!("{addr:#010X}");
	    }
	}
    }

    fn select(&mut self) {
	self.command = 0;
	self.command_bytes_received = 0;
	self.cursor = 0;
    }
}
