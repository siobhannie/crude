use super::DSP;

impl DSP {
    pub fn op_halt(&mut self) {
	self.control.set_halt();
    }
}
