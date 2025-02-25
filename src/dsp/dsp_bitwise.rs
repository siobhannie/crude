use super::{DSP, REG_AC0_M, REG_SR, SR_LZ};

impl DSP {
    pub fn op_sbset(&mut self, i: u16) {
	self.registers[REG_SR] |= (1 << (i + 6));
	self.pc += 1;
    }

    pub fn op_andf(&mut self, r: u16) {
	let imm = self.imem_read(self.pc + 1);
	if (self.registers[REG_AC0_M + (r as usize)] & imm) == 0 {
	    self.registers[REG_SR] |= SR_LZ;
	} else {
	    self.registers[REG_SR] &= !SR_LZ;
	}
	self.pc += 2;
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

    pub fn op_srbit(&mut self, b: u16) {
	match b {
	    2 => self.registers[REG_SR] &= !0x2000,
	    3 => self.registers[REG_SR] |= 0x2000,
	    4 => self.registers[REG_SR] &= !0x8000,
	    5 => self.registers[REG_SR] |= 0x8000,
	    6 => self.registers[REG_SR] &= !0x4000,
	    7 => self.registers[REG_SR] |= 0x4000,
	    _ => {},
	}
	self.pc += 1;
    }
}
