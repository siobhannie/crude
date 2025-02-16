use super::DSP;

impl DSP {
    pub fn op_lr(&mut self, d: u16) {
	let addr = self.imem_read(self.pc + 1);
	self.registers[d as usize] = self.dmem_read(addr);
	self.pc += 2;
    }
    
    pub fn op_lri(&mut self, d: u16) {
	self.registers[d as usize] = self.imem_read(self.pc + 1);
	self.pc += 2;
    }

    pub fn op_sr(&mut self, s: u16) {
	let addr = self.imem_read(self.pc + 1);
	self.dmem_write(addr, self.registers[s as usize]);
    }
}
