use crate::Gamecube;

pub struct DVDInterface {
    
}

impl DVDInterface {
    pub fn new() -> Self {
	Self {
	    
	}
    }
}

pub fn di_read_u32(gc: &mut Gamecube, offset: u32) -> u32 {
    match offset {
	0x24 => 1,
	_ => unimplemented!("DI read_u32 at offset {offset:#010X}"),
    }
}
