use super::{DSP, REG_AC0_H, REG_AC0_L, REG_AC0_M, REG_AC1_M, REG_SR};

impl DSP {
    pub fn conditional_extend_accum(&mut self, reg: usize) {
	if reg != REG_AC0_M && reg != REG_AC1_M {
	    return;
	}

	if !((self.registers[REG_SR] & 0x4000) != 0) {
	    return;
	}

	let ac = reg - REG_AC0_M;
	let val = self.registers[ac + REG_AC0_M];
	self.registers[ac + REG_AC0_H] = if (val & 0x8000) != 0 {
	    0xFFFF
	} else {
	    0x0000
	};
	self.registers[ac + REG_AC0_L] = 0x0;
    }
}
