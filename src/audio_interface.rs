use log::debug;

use crate::Gamecube;

pub struct AudioInterface {
    
}

impl AudioInterface {
    pub fn new() -> Self {
        Self {  }
    }
}

pub fn ai_write_u16(gc: &mut Gamecube, offset: u32, val: u16) {
    debug!("STUB: AI write_u16 at offset {offset:#010X} with val {val:#06X}");
}
