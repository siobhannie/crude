use super::DSP;

impl DSP {
    pub fn op_dar(&mut self, d: u16) {
	println!("{:#06X}: DAR ${d}", self.pc);
	self.registers[d as usize] -= 1;
	self.pc += 1;
    }

    pub fn op_iar(&mut self, d: u16) {
	println!("{:#06X}: IAR ${d}", self.pc);
	self.registers[d as usize] += 1;
	self.pc += 1;
    }

    pub fn op_addarn(&mut self, s: u16, d: u16) {
	println!("{:#06X}: ADDARN ${d}, ${s}", self.pc);
	self.registers[d as usize] = self.registers[d as usize].wrapping_add(self.registers[(s + 4) as usize]);
	self.pc += 1;
    }

    pub fn op_cmp(&mut self) {
	println!("{:#06X}: CMP", self.pc);
	let acc0 = self.acc(0) as i64;
	let acc1 = self.acc(1) as i64;
	let diff = ((acc0 - acc1) << 24) >> 24;

	let (carry, overflow) = sub_flags(acc0, acc1, diff);
	self.do_sr(diff, carry, overflow);
	self.pc += 1;
    }
}

pub fn sub_flags(a: i64, b: i64, res: i64) -> (bool, bool) {

    let carry = b >= res;
    let overflow = ((a ^ res) & (b ^ res)) < 0;
    (carry, overflow)
}
