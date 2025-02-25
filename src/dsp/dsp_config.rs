use super::DSP;

impl DSP {
    pub fn op_halt(&mut self) {
	println!("{:#06X}: HALT", self.pc);
	self.control.set_halt();
	self.pc += 1;
    }
}
