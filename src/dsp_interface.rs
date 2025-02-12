use crate::Gamecube;

pub struct DSPInterface {
    pub status: DSPControlStatusRegister,
    pub dma_mmem_addr: u32,
    pub dma_aram_addr: u32,
    pub dma_control: u32,
}

impl DSPInterface {
    pub fn new() -> Self {
	Self {
	    status: DSPControlStatusRegister(0),
	    dma_mmem_addr: 0,
	    dma_aram_addr: 0,
	    dma_control: 0,
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
    match offset {
	0x20 => gc.dsp.dma_mmem_addr = val,
	0x24 => gc.dsp.dma_aram_addr = val,
	0x28 => gc.dsp.dma_control = val,
	_ => unimplemented!("offset {offset:#010X} with val {val:#010X} for dsp write_u32!"),
    }
}

pub struct DSPControlStatusRegister(pub u16);

