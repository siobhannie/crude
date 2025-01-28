use super::EXIDevice;

const AD16_ID: u32 = 0x0412_0000;

pub struct AD16 {
    pub imm_data: u32,
}

impl AD16 {
    pub fn new() -> Self {
        Self {
	    imm_data: 0,
	}
    }
}

impl EXIDevice for AD16 {
    fn imm_write(&mut self) {
	match self.imm_data {
	    0x0 => {
		self.imm_data = AD16_ID;
	    },
	    a => unimplemented!("ad16 command {a:#X}"),
	}
    }

    fn imm_read(&mut self) -> u32 {
	self.imm_data
    }

    fn imm_data_write(&mut self, val: u32) {
	self.imm_data = val;
    }
}
