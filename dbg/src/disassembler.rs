use crude::cpu::instr::Instruction;

pub fn disassemble(instr: u32) -> String {
    let instr = Instruction(instr);
    match instr.opcd() {
	15 => {
	    disasm_addis(instr)
	}
	_ => String::from("unknown_opcd"),
    }
}

fn disasm_addis(instr: Instruction) -> String {
    let mut out = String::from("addis ");
    out.push_str(format!("r{},r{},{:#06X}", instr.d(), instr.a(), instr.simm()).as_str());
    out
}
