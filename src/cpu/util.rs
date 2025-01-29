use log::debug;

pub fn sext_26(val: u32) -> i32 {
    if val & 0x0200_0000 != 0 {
	(val | 0xFC00_0000) as i32
    } else {
	val as i32
    }
}

pub fn mask(mb: usize, me: usize) -> u32 {
    let mut mask = 0xFFFF_FFFF >> mb;

    if me >= 31 {
	mask ^= 0;
    } else {
	mask ^= 0xFFFF_FFFF >> (me + 1);
    };

    if me < mb {
	mask = !mask
    }
    debug!("mask: {mask:#034b}");
    mask
}
