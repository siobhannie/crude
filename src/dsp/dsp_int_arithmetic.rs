use super::DSP;

impl DSP {
    pub fn op_dar(&mut self, d: u16) {
	self.registers[d as usize] -= 1;
    }

    pub fn op_iar(&mut self, d: u16) {
	self.registers[d as usize] += 1;
    }

    pub fn op_addarn(&mut self, s: u16, d: u16) {
	self.registers[d as usize] = self.registers[d as usize].wrapping_add(self.registers[(s + 4) as usize]);
    }
}
