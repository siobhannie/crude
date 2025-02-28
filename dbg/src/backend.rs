use std::{collections::HashSet, fs::File, io::Read, panic::{catch_unwind, AssertUnwindSafe}, path::PathBuf, sync::{atomic::{AtomicU32, AtomicU8, Ordering}, mpsc::{channel, Receiver, Sender}, Arc}, thread};

use crude::{dsp::DSP, Gamecube};

pub type SharedInstructionBuffer = Arc<[(AtomicU32, AtomicU32)]>;

pub fn start_emu(ipl_path: impl ToString, instruction_buffer: SharedInstructionBuffer, processor_state: Arc<SharedProcessorState>) -> (Sender<Command>, Receiver<Message>) {
    let mut bios_data = Vec::new();
    File::open(ipl_path.to_string()).unwrap().read_to_end(&mut bios_data).unwrap();
    let (tx, rx) = channel();
    let (tx_m, rx_m) = channel();
    let mut breakpoints = HashSet::new();
    thread::spawn(move || {
	let aram = Arc::new(std::iter::repeat_with(|| AtomicU8::new(0)).take(0x0100_0000).collect::<Vec<_>>());
	let (mut dsp, client) = DSP::new(aram.clone());
	let mut gamecube = Gamecube::new(bios_data, aram, client);

	processor_state.update(&mut gamecube);
	update_instruction_buffer(&mut gamecube, &instruction_buffer);
	
	loop {
	    let result = catch_unwind(AssertUnwindSafe(|| {
		match rx.recv().unwrap() {
		    Command::Run => {
			'shmeep: loop {
			    for _ in 0..2000 {
				dsp.step();
				crude::step(&mut gamecube);
				if breakpoints.contains(&gamecube.cpu.cia) {
				    break 'shmeep;
				}
			    }

			    if let Ok(Command::Stop) = rx.try_recv() {
				break;
			    }
			    update_instruction_buffer(&mut gamecube, &instruction_buffer);
			    processor_state.update(&mut gamecube);
			}
		    },
		    Command::Step => {
			update_instruction_buffer(&mut gamecube, &instruction_buffer);
			processor_state.update(&mut gamecube);
			dsp.step();
			crude::step(&mut gamecube);
		    },
		    Command::Breakpoint(addr) => {
			breakpoints.insert(addr);
		    }
		    Command::Stop => {},
		}
	    }));
	    update_instruction_buffer(&mut gamecube, &instruction_buffer);
	    processor_state.update(&mut gamecube);
	}
    });

    (tx, rx_m)
}

pub struct SharedProcessorState {
    pub cia: AtomicU32,
    pub nia: AtomicU32,
    pub lr: AtomicU32,
    pub ctr: AtomicU32,
    pub srr0: AtomicU32,
    pub srr1: AtomicU32,
    pub gprs: Vec<AtomicU32>
}

impl SharedProcessorState {
    pub fn new() -> Self {
	let mut gprs = Vec::with_capacity(32);
	gprs.resize_with(32, || AtomicU32::new(0));
	Self {
	    cia: AtomicU32::new(0),
	    nia: AtomicU32::new(0),
	    lr: AtomicU32::new(0),
	    ctr: AtomicU32::new(0),
	    srr0: AtomicU32::new(0),
	    srr1: AtomicU32::new(0),
	    gprs,
	}
    }

    pub fn update(&self, gc: &mut Gamecube) {
	self.cia.store(gc.cpu.cia, Ordering::Relaxed);
	self.nia.store(gc.cpu.nia, Ordering::Relaxed);
	self.lr.store(gc.cpu.lr, Ordering::Relaxed);
	self.ctr.store(gc.cpu.ctr, Ordering::Relaxed);
	self.srr0.store(gc.cpu.srr0, Ordering::Relaxed);
	self.srr1.store(gc.cpu.srr1, Ordering::Relaxed);
	for (i, n) in gc.cpu.gprs.iter().enumerate() {
	    self.gprs[i].store(*n, Ordering::Relaxed);
	}
    }
}

fn update_instruction_buffer(gc: &mut Gamecube, buffer: &SharedInstructionBuffer) {
    let mut start = gc.cpu.cia.wrapping_sub(5 * 4);

    for (addr, instr) in buffer.iter() {
	addr.store(start, Ordering::Relaxed);
	let instruction = catch_unwind(AssertUnwindSafe(|| gc.read_u32(start, true))).unwrap_or(0);
	instr.store(instruction, Ordering::Relaxed);
	start = start.wrapping_add(4);
    }
}

pub enum Command {
    Run,
    Step,
    Stop,
    Breakpoint(u32),
}

pub enum Message {

}
