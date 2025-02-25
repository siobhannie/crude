use std::sync::atomic::Ordering;

use log::debug;

use crate::Gamecube;

pub struct DSPInterface {
    ar_size: u16,
    ar_refresh: u16,
    ar_dma_mmaddr: u32,
    ar_dma_araddr: u32,
    ar_dma_cnt: u32,
}

impl DSPInterface {
    pub fn new() -> Self {
	Self {
	    ar_size: 0,
	    ar_refresh: 0,
	    ar_dma_mmaddr: 0,
	    ar_dma_araddr: 0,
	    ar_dma_cnt: 0,
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
	0x00 => gc.dsp_client.dsp_mbox_h.store(val, Ordering::Relaxed),
	0x02 => gc.dsp_client.dsp_mbox_l.store(val, Ordering::Relaxed),
	0x0A => {
	    gc.dsp_client.control_reg.store(val, Ordering::Relaxed);
	    if gc.dsp_client.control_reg.reset() {
		for i in 0..0x2000 {
		    let val = gc.read_u8(0x8100_0000 + (i as u32));
		    gc.aram[i].store(val, Ordering::Relaxed);
		}
	    }
	},
	0x12 => gc.dsp.ar_size = val,
	0x1A => gc.dsp.ar_refresh = val,
	_ => unimplemented!("Unknown offset {offset:#010X} with val {val:#06X} for dsp write_u16!"),
    }
}

pub fn dsp_write_u32(gc: &mut Gamecube, offset: u32, val: u32) {
    debug!("STUB: DSP write_u32 at offset {offset:#010X} with val {val:#010X}");
    match offset {
	0x20 => gc.dsp.ar_dma_mmaddr = val,
	0x24 => gc.dsp.ar_dma_araddr = val,
	0x28 => {
	    gc.dsp.ar_dma_cnt = val;
	    let read = ((gc.dsp.ar_dma_cnt >> 31) & 1) != 0;
	    let length = (gc.dsp.ar_dma_cnt & !(1 << 31)) * 2;
	    println!("length: {length:#010X}");
	    if read {
		for i in 0..length {
		    gc.memory[(gc.dsp.ar_dma_mmaddr + i) as usize] = gc.aram[(gc.dsp.ar_dma_araddr + i) as usize].load(Ordering::Relaxed);
		}
	    } else {
		for i in 0..length {
		    let val = gc.memory[(gc.dsp.ar_dma_mmaddr + i) as usize];
		    println!("{val:#04X}");
		    gc.aram[(gc.dsp.ar_dma_araddr + i) as usize].store(val, Ordering::Relaxed);
		}
	    }
//	    panic!();
	},
	_ => unimplemented!("Unknown offset {offset:#010X} with val {val:#06X} for dsp write_u32!"),
    }
}
