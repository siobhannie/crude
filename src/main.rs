use std::{env, fs::File, io::{stdout, Read}, time::SystemTime};

use crude::Gamecube;
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
    let mut gamecube = Gamecube::new(bios_data);
    crude::run(&mut gamecube);
}
