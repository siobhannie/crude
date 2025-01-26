use log::debug;

use crate::Gamecube;

use super::instr::Instruction;

pub fn isync(gc: &mut Gamecube, instr: &Instruction) {
    debug!("STUB: isync");
}
