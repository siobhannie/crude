use log::debug;

use crate::Gamecube;

pub struct SerialInterface {
    
}

pub fn si_write_u32(gc: &mut Gamecube, offset: u32, val: u32) {
    debug!("STUB: SI write_u32 at offset {offset:#010X} with val {val:#010X}");
}
