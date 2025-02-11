use std::sync::{Arc, Mutex, RwLock};

use crate::sram::Sram;

use super::EXIDevice;

pub struct Bootrom {
    rom: Vec<u8>,
    sram: Arc<RwLock<Sram>>,
    command_bytes_received: u32,
    command: u32,
    cursor: u32,
}

impl Bootrom {
    pub fn new(rom: Vec<u8>, sram: Arc<RwLock<Sram>>) -> Self {
	Self {
	    rom,
	    sram,
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
	    if addr < 0x0020_0000 {
		if !self.is_write() {
		    *byte = self.rom[(addr + self.cursor) as usize];
		    self.cursor += 1;
		}
	    } else if addr >= 0x0080_0000 && addr < 0x0080_0044 {
		let dev_addr = addr - 0x0080_0000 + self.cursor;
		if self.is_write() {
		    self.sram.write().unwrap().as_byte_array_mut()[dev_addr as usize] = *byte;
		} else {
		    *byte = self.sram.read().unwrap().as_byte_array()[dev_addr as usize];
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
