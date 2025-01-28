pub mod ad16;
pub mod null;

use std::intrinsics::unreachable;

use ad16::AD16;
use log::debug;
use null::NullDevice;

use crate::Gamecube;

pub struct ExternalInterface {
    channel0: EXIChannel,
    channel1: EXIChannel,
    channel2: EXIChannel,
}

impl ExternalInterface {
    pub fn new() -> Self {
        Self {
	    channel0: EXIChannel::new([Box::new(NullDevice), Box::new(NullDevice), Box::new(NullDevice)]),
	    channel1: EXIChannel::new([Box::new(NullDevice), Box::new(NullDevice), Box::new(NullDevice)]),
	    channel2: EXIChannel::new([Box::new(AD16::new()), Box::new(NullDevice), Box::new(NullDevice)]),
	}
    }
}

pub fn exi_write_u32(gc: &mut Gamecube, offset: u32, val: u32) {
    let channel_idx = offset / 0x14;
    let channel = match channel_idx {
	0 => &mut gc.exi.channel0,
	1 => &mut gc.exi.channel1,
	2 => &mut gc.exi.channel2,
	_ => unreachable!("attempted to access exi channel {channel_idx}"),
    };
    let reg = offset % 0x14;

    channel.write(reg, val);
    
    debug!("EXI write channel: {channel_idx} at register: {reg:#X} with val: {val:#010X}");
}

pub fn exi_read_u32(gc: &mut Gamecube, offset: u32) -> u32 {
    let channel_idx = offset/0x14;
    let channel = match channel_idx {
	0 => &mut gc.exi.channel0,
	1 => &mut gc.exi.channel1,
	2 => &mut gc.exi.channel2,
	_ => unreachable!("attempted to access exi channel {channel_idx}"),
    };
    let reg = offset % 0x14;

    debug!("EXI read channel: {channel_idx} at register: {reg:#X}");

    channel.read(reg)
}

pub trait EXIDevice {
    fn imm_write(&mut self);
    fn imm_read(&mut self) -> u32;
    fn imm_data_write(&mut self, val: u32);
}

pub struct EXIChannel {
    params: EXIChannelParams,
    dma_start: u32,
    dma_length: u32,
    control: EXIChannelControl,
    devices: [Box<dyn EXIDevice>; 3],
}

impl EXIChannel {
    pub fn new(devices: [Box<dyn EXIDevice>; 3]) -> Self {
	Self {
	    params: EXIChannelParams(0),
	    dma_start: 0,
	    dma_length: 0,
	    control: EXIChannelControl(0),
	    devices,
	}
    }

    fn choose_device(&mut self) -> &mut Box<dyn EXIDevice> {
	match self.params.cs() {
	    0x1 => &mut self.devices[0],
	    0x2 => &mut self.devices[1],
	    0x4 => &mut self.devices[2],
	    _ => &mut self.devices[0],
	}
    }

    pub fn read(&mut self, reg: u32) -> u32 {
	let device = self.choose_device();
	match reg {
	    0x0 => self.params.0,
	    0xC => self.control.0,
	    0x10 => device.imm_read(),
	    _ => unreachable!("EXI Channel reg {reg:#X} read"),
	}
    }

    pub fn write(&mut self, reg: u32, val: u32) {
	let device = self.choose_device();
	match reg {
	    0x0 => {
		self.params = EXIChannelParams(val);
		let dma = self.control.dma();
		let rw = self.control.rw();
		let device = self.choose_device();
		if dma {
		    unimplemented!("dma transfer");
		} else {
		    if rw == 0x0 {
			//nothing to do...
		    } else if rw == 0x1 {
			device.imm_write();
		    }
		}
	    },
	    0xC => self.control = EXIChannelControl(val),
	    0x10 => device.imm_data_write(val),
	    _ => unreachable!("EXI Channel reg {reg:#X} with val {val:#X}"),
	}
    }
}

pub struct EXIChannelParams(pub u32);

impl EXIChannelParams {
    pub fn exi_int_mask(&self) -> bool {
	(self.0 & 1) != 0
    }

    pub fn exi_int(&self) -> bool {
	((self.0 >> 1) & 1) != 0
    }

    pub fn tc_int_mask(&self) -> bool {
	((self.0 >> 2) & 1) != 0
    }

    pub fn tc_int(&self) -> bool {
	((self.0 >> 3) & 1) != 0
    }

    pub fn clk(&self) -> usize {
	((self.0 >> 4) & 0x7) as usize
    }

    pub fn cs(&self) -> usize {
	((self.0 >> 7) & 0x7) as usize
    }

    pub fn ext_int_mask(&self) -> bool {
	((self.0 >> 10) & 1) != 0
    }

    pub fn ext_int(&self) -> bool {
	((self.0 >> 11) & 1) != 0
    }

    pub fn ext(&self) -> bool {
	((self.0 >> 12) & 1) != 0
    }

    pub fn rom_dis(&self) -> bool {
	((self.0 >> 13) & 1) != 0
    }
}

pub struct EXIChannelControl(pub u32);

impl EXIChannelControl {
    pub fn t_start(&self) -> bool {
	(self.0 & 1) != 0
    }

    pub fn dma(&self) -> bool {
	((self.0 >> 1) & 1) != 0
    }

    pub fn rw(&self) -> usize {
	((self.0 >> 2) & 0x3) as usize
    }

    pub fn t_len(&self) -> usize {
	((self.0 >> 4) & 0x3) as usize
    }
}
