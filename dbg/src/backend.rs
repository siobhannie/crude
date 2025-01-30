use std::{fs::File, io::Read, path::PathBuf, sync::mpsc::{channel, Receiver, Sender}, thread};

use crude::Gamecube;

pub fn start_emu(ipl_path: impl ToString) -> (Sender<Command>, Receiver<Message>) {
    let mut bios_data = Vec::new();
    File::open(ipl_path.to_string()).unwrap().read_to_end(&mut bios_data).unwrap();
    let (tx, rx) = channel();
    let (tx_m, rx_m) = channel();
    thread::spawn(move || {
	let mut gamecube = Gamecube::new();
	gamecube.load_bios(bios_data);

	loop {
	    match rx.recv().unwrap() {
		Command::Run => {
		    crude::run(&mut gamecube);
		},
		Command::Step => {
		    let instr = gamecube.read_u32(gamecube.cpu.cia, true);
		    tx_m.send(Message::Instruction(instr)).unwrap();
		    crude::step(&mut gamecube);
		}
	    }
	}
    });

    (tx, rx_m)
}

pub enum Command {
    Run,
    Step,
}

pub enum Message {
    Instruction(u32)
}
