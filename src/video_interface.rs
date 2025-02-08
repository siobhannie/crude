use crate::Gamecube;

pub struct VideoInterface {
    
}

impl VideoInterface {
    pub fn new() -> Self {
	Self {
	    
	}
    }
}

pub fn vi_read_u16(gc: &mut Gamecube, offset: u32) -> u16 {
    match offset {
	0x6C => 0,
	_ => unimplemented!("vi read_u16 at offset {:#010X}", offset),
    }
}
