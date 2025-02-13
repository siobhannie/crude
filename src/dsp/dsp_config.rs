use super::DSP;

impl DSP {
    pub fn op_halt(&mut self) {
	self.client.control_reg.set_halt();
    }
}
