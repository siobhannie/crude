use client::DSPClient;
use stacks::DSPStacks;

pub mod client;
mod stacks;

pub struct DSP {
    registers: [u16; 32],
    pc: u16,
    imem: [u16; 0x1000],
    dmem: [u16; 0x1000],
    irom: [u16; 0x1000],
    drom: [u16; 0x0800],
    exceptions: u16,
    stacks: DSPStacks,
}


impl DSP {
    pub fn new() -> (Self, DSPClient) {
	(Self {
	    registers: [0; 32],
	    pc: 0,
	    imem: [0; 0x1000],
	    dmem: [0; 0x1000],
	    irom: [0; 0x1000],
	    drom: [0; 0x0800],
	    exceptions: 0,
	    stacks: DSPStacks::new(),
	}, DSPClient::new())
    }

    pub fn step(&mut self) {
	let instr = self.imem_read(self.pc);
    }

    fn imem_read(&mut self, addr: u16) -> u16 {
	todo!();
    }

    fn imem_write(&mut self, addr: u16, val: u16) {
	todo!();
    }

    fn dmem_reda(&mut self, addr: u16) -> u16 {
	todo!();
    }

    fn dmem_write(&mut self, addr: u16, val: u16) {
	todo!();
    }
}
