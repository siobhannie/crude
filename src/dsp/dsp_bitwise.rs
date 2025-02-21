use super::{DSP, REG_AC0_M, REG_SR, SR_LZ};

impl DSP {
    pub fn op_sbset(&mut self, i: u16) {
	self.registers[REG_SR] |= (1 << (i + 6));
	self.pc += 1;
    }

    pub fn op_andcf(&mut self, r: u16) {
	let imm = self.imem_read(self.pc + 1);
	if (self.registers[REG_AC0_M + (r as usize)] & imm) == imm {
	    self.registers[REG_SR] |= SR_LZ;
	} else {
	    self.registers[REG_SR] &= !SR_LZ;
	}
	self.pc += 2;
    }
}
