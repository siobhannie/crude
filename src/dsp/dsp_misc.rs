use super::{DSP, REG_AC0_H, REG_AC0_L, REG_AC0_M, REG_SR};

impl DSP {    
    pub fn op_clr(&mut self, r: u16) {
	println!("{:#06X}: CLR ${r}", self.pc);
	let i = r as usize;
	self.registers[REG_AC0_H + i] = 0;
	self.registers[REG_AC0_L + i] = 0;
	self.registers[REG_AC0_M + i] = 0;
	self.registers[REG_SR] |= 1 << 2;
	self.pc += 1;
    }

    pub fn op_mrr(&mut self, d: u16, s: u16) {
	println!("{:#06X}: MRR ${d}, ${s}", self.pc);
	self.registers[d as usize] = self.registers[s as usize];
	self.conditional_extend_accum(d as usize);
	self.pc += 1;
    }
}
