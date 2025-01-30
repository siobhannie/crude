mod ext;
mod backend;
mod disassembler;

use std::env;

use backend::{start_emu, Command, Message};
use disassembler::disassemble;
use ext::System;
use imgui::Condition;
use log::LevelFilter;

fn main() {
    let (emu_commander, emu_messages) = start_emu(env::args().nth(1).unwrap());
    simple_logger::SimpleLogger::new().with_level(LevelFilter::Debug).init().unwrap();
    let mut message = String::from("Waiting...");
    System::new("shmeeeep :3").unwrap().run((), move |_, ui, _| {
	ui.window("Emu control")
	    .size([300.0, 110.0], Condition::FirstUseEver)
	    .build(|| {
		if ui.button("Step") {
		    emu_commander.send(Command::Step).unwrap();
		}
		if ui.button("Run") {
		    emu_commander.send(Command::Run).unwrap();
		}
	    });

	ui.window("Instructions")
	    .size([400.0, 600.0], Condition::Always)
	    .build(|| {
		if let Ok(Message::Instruction(instr)) = emu_messages.try_recv() {
		    let disasmed = disassemble(instr);
		    *(&mut message) = format!("Current Instruction: {disasmed}");
		}

		ui.text_wrapped(&message);
	    });
    }).unwrap();
}
