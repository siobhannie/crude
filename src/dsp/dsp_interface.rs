use std::sync::atomic::Ordering;

use log::debug;

use crate::Gamecube;

pub struct DSPInterface {
    ar_size: u16,
    ar_refresh: u16,
}

impl DSPInterface {
    pub fn new() -> Self {
	Self {
	    ar_size: 0,
	    ar_refresh: 0,
	}
    }
}

pub fn dsp_read_u16(gc: &mut Gamecube, offset: u32) -> u16 {
    debug!("STUB: DSP read_u16 at offset {offset:#010X}!");
    match offset {
	0x04 => gc.dsp_client.cpu_mbox_h.load(Ordering::Relaxed),
	0x06 => gc.dsp_client.cpu_mbox_l.load(Ordering::Relaxed),
	0x0A => gc.dsp_client.control_reg.load(Ordering::Relaxed),
	_ => unimplemented!("Unknown offset {offset:#010X} for dsp read_u16!"),
    }
}

pub fn dsp_write_u16(gc: &mut Gamecube, offset: u32, val: u16) {
    debug!("STUB: DSP write_u16 at offset {offset:#010X} with val {val:#06X}");
    match offset {
	0x0A => gc.dsp_client.control_reg.store(val, Ordering::Relaxed),
	0x12 => gc.dsp.ar_size = val,
	0x1A => gc.dsp.ar_refresh = val,
	_ => unimplemented!("Unknown offset {offset:#010X} with val {val:#06X} for dsp write_u16!"),
    }
}
