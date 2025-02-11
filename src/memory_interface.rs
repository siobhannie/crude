use log::debug;

use crate::Gamecube;

pub struct MemoryInterface {
    pub mask_ints: u16,
}

impl MemoryInterface {
    pub fn new() -> Self {
        Self {
	    mask_ints: 0,
	}
    }
}

pub fn mi_write_u16(gc: &mut Gamecube, offset: u32, val: u16) {
    match offset {
	0x1C => gc.mi.mask_ints = val,
	0x26 => {}, //ipl tries to set this, i couldn't find anything documenting what it means...
	_ => unimplemented!("MI write_u16 at offset {offset:#010X} with val {val:#06X}")
    }
    debug!("STUB: MI  write_u16 at offset {offset:#010X} with val {val:#06X}");
}
