use log::debug;

use crate::Gamecube;

pub struct ProcessorInterface {
    
}

impl ProcessorInterface {
    pub fn new() -> Self {
        Self {  }
    }
}

pub fn pi_write_u32(gc: &mut Gamecube, offset: u32, val: u32) {
    debug!("STUB: PI write_u32 at offset {offset:#010X} with val {val:#06X}");
}

pub fn pi_read_u32(gc: &mut Gamecube, offset: u32) -> u32 {
    debug!("STUB: PI read_u32 at offset {offset:#010X}");
    0
}
