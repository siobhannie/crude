use std::io::stdin;

use log::debug;

use super::EXIDevice;

const AD16_ID: u32 = 0x0412_0000;

pub struct AD16 {
    reg: u32,
    position: u32,
    command: u32,
}

impl AD16 {
    pub fn new() -> Self {
        Self {
	    reg: 0,
	    position: 0,
	    command: 0,
	}
    }
}

impl EXIDevice for AD16 {
    fn transfer_byte(&mut self, byte: &mut u8) {
	if self.position == 0 {
	    self.command = (*byte) as u32;
	} else {
	    match self.command {
		0 => {
		    self.reg = AD16_ID;
		    
		    match self.position {
			2 => *byte = self.reg as u8,
			3 => *byte = (self.reg >> 8) as u8,
			4 => *byte = (self.reg >> 16) as u8,
			5 => *byte = (self.reg >> 24) as u8,
			_ => {},
			
		    }
		},
		0xA2 => {
		    panic!();
		},
		0xA0 => {
		    panic!();
		    match self.position {
			1 => {
			    self.reg &= !0xFF;
			    self.reg |= *byte as u32;
			}
			2 => {
			    self.reg &= !(0xFF << 8);
			    self.reg |= (*byte as u32) << 8;
			}
			3 => {
			    panic!();
			    self.reg &= !(0xFF << 16);
			    self.reg |= (*byte as u32) << 16;
			},
			4 => {
			    panic!();
			    self.reg &= !(0xFF << 24);
			    self.reg |= (*byte as u32) << 24;
			    println!("{:#010X}", self.reg);
			    stdin().read_line(&mut String::new()).unwrap();
			}
			_ => {},
			
		    }
		}
		a => unimplemented!("ad16 command {a:#X}"),
	    }
	}

	self.position += 1;
    }

    fn select(&mut self) {
        self.position = 0;
	self.reg = 0;
	self.command = 0;
    }

    
}
