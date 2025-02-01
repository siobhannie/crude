use log::debug;

use crate::Gamecube;

pub struct SerialInterface {
    clock_lock: u32,
}

impl SerialInterface {
    pub fn new() -> Self {
	Self {
	    clock_lock: 0,
	}
    }
}

pub fn si_read_u32(gc: &mut Gamecube, offset: u32) -> u32 {
    debug!("STUB: SI read_u32 at offset {offset:#010X}");
    match offset {
	0x3C => gc.si.clock_lock,
	_ => 0,
    }
}

pub fn si_write_u32(gc: &mut Gamecube, offset: u32, val: u32) {
    match offset {
	0x3C => gc.si.clock_lock = val,
	_ => {},
    }
    debug!("STUB: SI write_u32 at offset {offset:#010X} with val {val:#010X}");
}
