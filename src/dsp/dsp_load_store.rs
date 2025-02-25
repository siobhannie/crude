use crate::dsp::REG_CONFIG;

use super::{DSP, REG_AC0_H, REG_AC0_L, REG_AC0_M, REG_AC1_M, REG_SR};

impl DSP {
    pub fn op_lr(&mut self, d: u16) {
	let addr = self.imem_read(self.pc + 1);
	println!("{:#06X}: LR ${d}, @{addr:#06X}", self.pc);
	let val = self.dmem_read(addr);
	self.write_reg(d as usize, val);
	self.conditional_extend_accum(d as usize);
	self.pc += 2;
    }

    pub fn op_lrs(&mut self, d: u16, m: u16) {
	let m = (self.registers[REG_CONFIG] << 8) | m;
	println!("{:#06X}: LRS $(0x18 + {d}), @{m:#06X}", self.pc);
	let val = self.dmem_read(m);
	self.write_reg(0x18 + (d as usize), val);
	self.conditional_extend_accum(0x18 + (d as usize));
	self.pc += 1;
    }
    
    pub fn op_lri(&mut self, d: u16) {
	let imm = self.imem_read(self.pc + 1);
	println!("{:#06X}: LRI ${d}, #{imm:#06X}", self.pc);
	self.write_reg(d as usize, imm);
	self.conditional_extend_accum(d as usize);
	self.pc += 2;
    }

    pub fn op_lrri(&mut self, s: u16, d: u16) {
	let val = self.dmem_read(self.registers[s as usize]);
	println!("{:06X}: LRRI ${d}, @${s}", self.pc);
	self.registers[d as usize] = val;
	self.conditional_extend_accum(d as usize);
	self.registers[s as usize] += 1;
	self.pc += 1;
    }

    pub fn op_ilrr(&mut self, d: u16, s: u16, change_by: i16) {
	let reg = if d == 0 {
	    REG_AC0_M
	} else {
	    REG_AC1_M
	};

	println!("{:06X}: ILRR $ac{d}.m, @$ar{s}", self.pc);

	self.registers[reg] = self.imem_read(self.registers[s as usize]);
	self.conditional_extend_accum(reg);
	self.registers[s as usize] = self.registers[s as usize].wrapping_add_signed(change_by);
	self.pc += 1;
    }
    
    pub fn op_sr(&mut self, s: u16) {
	let addr = self.imem_read(self.pc + 1);
	println!("{:06X}: SR @{addr:#06X}, ${s}", self.pc);
	self.dmem_write(addr, self.registers[s as usize]);
	self.pc += 2;
    }

    pub fn op_srs(&mut self, s: u16, m: u16) {
	let m = (self.registers[REG_CONFIG] << 8) | m;
	println!("{:#06X}: SRS @{m:#06X}, $(0x18 + {s})", self.pc);
	self.dmem_write(m, self.registers[0x18 + (s as usize)]);
	self.pc += 1;
    }

    pub fn op_si(&mut self, m: u16) {
	let m = (m as i8) as u16;
	let imm = self.imem_read(self.pc + 1);
	println!("{:#06X}: SI @{m:#06X}, #{imm:#06X}", self.pc);
	self.dmem_write(m, imm);
	self.pc += 2;
    }
}
