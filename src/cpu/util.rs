use log::debug;

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
