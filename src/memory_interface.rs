use log::debug;

use crate::Gamecube;

pub struct MemoryInterface {
    
}

impl MemoryInterface {
    pub fn new() -> Self {
        Self {  }
    }
}

pub fn mi_write_u16(gc: &mut Gamecube, offset: u32, val: u16) {
    debug!("STUB: MI  write_u16 at offset {offset:#010X} with val {val:#06X}");
}
