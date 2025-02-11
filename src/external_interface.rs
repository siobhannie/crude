pub mod ad16;
pub mod null;
pub mod bootrom;
pub mod no_device;

use std::{intrinsics::unreachable, sync::{Arc, Mutex, RwLock}};

use ad16::AD16;
use bootrom::Bootrom;
use log::debug;
use no_device::NoDevice;
use null::NullDevice;

use crate::{sram::Sram, Gamecube};

pub struct ExternalInterface {
    channel0: EXIChannel,
    channel1: EXIChannel,
    channel2: EXIChannel,
}

impl ExternalInterface {
    pub fn new(bootrom: Vec<u8>, sram: Arc<RwLock<Sram>>) -> Self {
        Self {
	    channel0: EXIChannel::new([Box::new(NullDevice), Box::new(Bootrom::new(bootrom, sram)), Box::new(NoDevice)]),
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
    debug!("EXI write_u32 to channel {channel_idx} in reg {reg:#X} with val {val:#X}");
    channel.write(&mut gc.memory, reg, val);
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
    debug!("EXI read_u32 to channel {channel_idx} in reg {reg:#X}");
    channel.read(reg)
}

pub trait EXIDevice {
    fn transfer_byte(&mut self, byte: &mut u8);
    
    fn imm_write(&mut self, mut data: u32, mut size: u32) {
	while size != 0 {
	    size -= 1;
	    let mut byte = (data >> 24) as u8;
	    self.transfer_byte(&mut byte);
	    data <<= 8;
	}
    }
    fn imm_read(&mut self, mut size: u32) -> u32 {
	let mut position = 0u32;
	let mut result = 0u32;

	while size != 0 {
	    size -= 1;
	    let mut byte = 0u8;
	    self.transfer_byte(&mut byte);
	    result |= (byte as u32) << (24 - (position * 8));
	    position += 1;
	}

	result
    }

    
    fn dma_write(&mut self, memory: &mut Vec<u8>, mut addr: u32, mut size: u32) {
	while size != 0 {
	    size -= 1;
	    let mut byte = *memory.get(addr as usize).unwrap();
	    self.transfer_byte(&mut byte);
	    addr += 1;
	}
    }

    fn dma_read(&mut self, memory: &mut Vec<u8>, mut addr: u32, mut size: u32) {
	while size != 0 {
	    size -= 1;
	    let mut byte = 0u8;
	    self.transfer_byte(&mut byte);
	    *memory.get_mut(addr as usize).unwrap() = byte;
	    addr += 1;
	}
    }
    
    fn select(&mut self);
}

pub struct EXIChannel {
    params: EXIChannelParams,
    dma_start: u32,
    dma_length: u32,
    control: EXIChannelControl,
    devices: [Box<dyn EXIDevice>; 3],
    imm_data: u32,
}

impl EXIChannel {
    pub fn new(devices: [Box<dyn EXIDevice>; 3]) -> Self {
	Self {
	    params: EXIChannelParams(0),
	    dma_start: 0,
	    dma_length: 0,
	    control: EXIChannelControl(0),
	    devices,
	    imm_data: 0,
	}
    }

    fn choose_device(&mut self) -> &mut Box<dyn EXIDevice> {
	match self.params.cs() {
	    0b001 => &mut self.devices[0],
	    0b010 => &mut self.devices[1],
	    0b100 => &mut self.devices[2],
	    _ => &mut self.devices[0],
	}
    }

    pub fn read(&mut self, reg: u32) -> u32 {
	match reg {
	    0x0 => self.params.0,
	    0x4 => self.dma_start,
	    0x8 => self.dma_length,
	    0xC => self.control.0,
	    0x10 => self.imm_data,
	    _ => unreachable!("read from unsupported EXI reg: {reg:#X}"),
	}
    }

    pub fn write(&mut self, mem: &mut Vec<u8>, reg: u32, val: u32) {
	match reg {
	    0x0 => {
		self.params = EXIChannelParams(val);
		debug!("new device: {:#0b}", self.params.cs());
		self.choose_device().select();
		//TODO: make this more like dolphin once interrupts are actually implemented.
	    },
	    0x4 => self.dma_start = val,
	    0x8 => self.dma_length = val,
	    0xC => {
		self.control = EXIChannelControl(val);
		
		if self.control.t_start() {
		    if self.control.dma() {
			let dma_addr = self.dma_start;
			let dma_size = self.dma_length;
			
			match self.control.rw() {
			    0 => {
				self.choose_device().dma_read(mem, dma_addr, dma_size);
			    },
			    1 => {
				self.choose_device().dma_write(mem, dma_addr, dma_size);
			    }
			    _ => {
				unimplemented!("unsupported mode for dma!");
			    }
			}
		    } else {
			match self.control.rw() {
			    0 => {
				debug!("read!");
				let t_len = self.control.t_len() + 1;
				self.imm_data = self.choose_device().imm_read(t_len as u32);
			    },
			    1 => {
				debug!("write!");
				let data = self.imm_data;
				let t_len = self.control.t_len() + 1;
				self.choose_device().imm_write(data, t_len as u32);
			    },
			    rw => {
				unimplemented!("unimplemented rw mode {rw:#X}");
			    },
			}
		    }
		}
		self.control.clear_t_start();
	    }
	    0x10 => self.imm_data = val,
	    _ => unreachable!("write to unsupported EXI reg: {reg:#X} with val {val:#X}"),
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

    pub fn clear_exi_int(&mut self) {
	self.0 &= !(1 << 1);
    }

    pub fn tc_int_mask(&self) -> bool {
	((self.0 >> 2) & 1) != 0
    }

    pub fn tc_int(&self) -> bool {
	((self.0 >> 3) & 1) != 0
    }

    pub fn clear_tc_int(&mut self) {
	self.0 &= !(1 << 3);
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

    pub fn clear_ext_int(&mut self) {
	self.0 &= !(1 << 11);
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

    pub fn clear_t_start(&mut self) {
	self.0 &= !1;
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
