use crate::Gamecube;

pub struct DSPInterface {
    pub status: DSPControlStatusRegister,
    
}

impl DSPInterface {
    pub fn new() -> Self {
	Self {
	    status: DSPControlStatusRegister(0),
	}
    }
}

pub fn dsp_read_u16(gc: &mut Gamecube, offset: u32) -> u16 {
    match offset {
	0x04 => 0,
	0x06 => 0,
	0x0a => gc.dsp.status.0,
	_ => unimplemented!("offset {offset:#010X} for dsp read_u16!"),
    }
}

pub fn dsp_write_u32(gc: &mut Gamecube, offset: u32, val: u32) {
    todo!();
}

pub struct DSPControlStatusRegister(pub u16);

