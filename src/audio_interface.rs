use log::debug;

use crate::Gamecube;

pub struct AudioInterface {
    pub control: u32,
}

impl AudioInterface {
    pub fn new() -> Self {
        Self {
	    control: 0,
	}
    }
}

pub fn ai_write_u16(gc: &mut Gamecube, offset: u32, val: u16) {
    debug!("STUB: AI write_u16 at offset {offset:#010X} with val {val:#06X}");
}

pub fn ai_write_u32(gc: &mut Gamecube, offset: u32, val: u32) {
    match offset {
	0x00 => gc.ai.control = val,
	_ => unimplemented!("AI write_u32 at offset {offset:#010X} with val {val:#010X}"),
    }
}

pub fn ai_read_u32(gc: &mut Gamecube, offset: u32) -> u32 {
    match offset {
	0x00 => gc.ai.control,
	_ => unimplemented!("AI read_u32 at offset {offset:#010X}"),
    }
}
