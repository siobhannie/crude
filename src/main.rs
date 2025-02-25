use std::{env, fs::File, io::{stdout, Read}, sync::{atomic::AtomicU8, Arc, RwLock}, time::SystemTime};

use crude::{dsp::DSP, Gamecube};
use fern::Dispatch;
use log::LevelFilter;

fn main() {
    Dispatch::new()
        .format(|out, message, record| {
	    out.finish(format_args!(
		"[{} {} {}] {}",
		humantime::format_rfc3339_seconds(SystemTime::now()),
		record.level(),
		record.target(),
		message
	    ))
	})
        .level(LevelFilter::Debug)
        .chain(stdout())
        .apply().unwrap();
    let bios_path = env::args().nth(1).unwrap();
    let mut bios_data = Vec::new();
    File::open(bios_path).unwrap().read_to_end(&mut bios_data).unwrap();
    let aram = Arc::new(std::iter::repeat_with(|| AtomicU8::new(0)).take(0x0100_0000).collect::<Vec<_>>());
    let (mut dsp, client) = DSP::new(aram.clone());
    let mut gamecube = Gamecube::new(bios_data, aram, client);
    loop {
	dsp.step();
	crude::step(&mut gamecube);
    }
}
