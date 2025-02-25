use super::{DSP, REG_ST0, REG_ST2, REG_ST3};

impl DSP {
    pub fn op_loop(&mut self, r: u16) {
	let r = self.registers[r as usize];
	println!("{:#06X}, LOOP ${r}", self.pc);
	self.pc += 1;
	for _ in r..0 {
	    self.step();
	}
	self.pc += 1;
    }

    pub fn op_bloop(&mut self, r: u16) {
	self.registers[REG_ST0] = self.pc + 2;
	let a = self.imem_read(self.pc + 1);
	println!("{:#06X}, BLOOP ${r}, {a:#06X}", self.pc);
	self.registers[REG_ST2] = a;
	self.registers[REG_ST3] = self.registers[r as usize];
	self.pc += 2;
	while self.registers[REG_ST3] != 0 {
	    self.registers[REG_ST3] -= 1;
	    while self.pc != self.registers[REG_ST2] {
		self.step();
	    }
	    self.pc = self.registers[REG_ST0];
	}
	self.pc = a + 1;
    }

    pub fn op_if(&mut self, c: u16) {
	println!("{:#06X}: IF({c:#04b})", self.pc);
	self.pc += 1;
	if self.condition(c) {
	    self.step();
	} else {
	    self.pc += 1;
	}
    }

    pub fn op_j(&mut self, c: u16) {
	println!("{:#06X}: J({c:#04b})", self.pc);
	if self.condition(c) {
	    let dest = self.imem_read(self.pc + 1);
	    self.pc = dest;
	} else {
	    self.pc += 2;
	}
    }

    pub fn op_call(&mut self, c: u16) {
	println!("{:#06X}: CALL({c:#04b})", self.pc);
	if self.condition(c) {
	    let dest = self.imem_read(self.pc + 1);
	    self.push_stack(0, self.pc + 2);
	    self.pc = dest;
	} else {
	    self.pc += 2;
	}
    }

    pub fn op_ret(&mut self, c: u16) {
	println!("{:#06X}: RET({c:#04b})", self.pc);
	if self.condition(c) {
	    self.pc = self.pop_stack(0);
	} else {
	    self.pc += 1;
	}
    }
}
