use log::debug;

const DEQUANTIZE_TABLE: [f32; 64] = [
    1.0 / (1 << 0) as f32,  1.0 / (1 << 1) as f32,  1.0 / (1 << 2) as f32, 1.0 / (1 << 3) as f32,
    1.0 / (1 << 4) as f32,  1.0 / (1 << 5) as f32,  1.0 / (1 << 6) as f32, 1.0 / (1 << 7) as f32,
    1.0 / (1 << 8) as f32,  1.0 / (1 << 9) as f32,  1.0 / (1 << 10) as f32, 1.0 / (1 << 11) as f32,
    1.0 / (1 << 12) as f32, 1.0 / (1 << 13) as f32, 1.0 / (1 << 14) as f32, 1.0 / (1 << 15) as f32,
    1.0 / (1 << 16) as f32, 1.0 / (1 << 17) as f32, 1.0 / (1 << 18) as f32, 1.0 / (1 << 19) as f32,
    1.0 / (1 << 20) as f32, 1.0 / (1 << 21) as f32, 1.0 / (1 << 22) as f32, 1.0 / (1 << 23) as f32,
    1.0 / (1 << 24) as f32, 1.0 / (1 << 25) as f32, 1.0 / (1 << 26) as f32, 1.0 / (1 << 27) as f32,
    1.0 / (1 << 28) as f32, 1.0 / (1 << 29) as f32, 1.0 / (1 << 30) as f32, 1.0 / (1 << 31) as f32,
    (1u64 << 32) as f32,    (1 << 31) as f32,       (1 << 30) as f32,       (1 << 29) as f32,
    (1 << 28) as f32,       (1 << 27) as f32,       (1 << 26) as f32,       (1 << 25) as f32,
    (1 << 24) as f32,       (1 << 23) as f32,       (1 << 22) as f32,       (1 << 21) as f32,
    (1 << 20) as f32,       (1 << 19) as f32,       (1 << 18) as f32,       (1 << 17) as f32,
    (1 << 16) as f32,       (1 << 15) as f32,       (1 << 14) as f32,       (1 << 13) as f32,
    (1 << 12) as f32,       (1 << 11) as f32,       (1 << 10) as f32,       (1 << 9) as f32,
    (1 << 8) as f32,        (1 << 7) as f32,        (1 << 6) as f32,        (1 << 5) as f32,
    (1 << 4) as f32,        (1 << 3) as f32,        (1 << 2) as f32,        (1 << 1) as f32,
    ];

pub fn dequantized(val: u32, ld_type: usize, ld_scale: usize) -> f32 {
    let result = match ld_type {
	0 => f32::from_bits(val),
	4 => (val as u8) as f32,
	5 => (val as u16) as f32,
	6 => (val as i8) as f32,
	7 => (val as i16) as f32,
	_ => {
	    f32::from_bits(val)
	},
    };

    result * DEQUANTIZE_TABLE[ld_scale]
}

pub fn sext_26(val: u32) -> i32 {
    if val & 0x0200_0000 != 0 {
	(val | 0xFC00_0000) as i32
    } else {
	val as i32
    }
}

pub fn sext_12(val: u16) -> i32 {
    if val & 0x800 != 0 {
	i32::from(val | 0xF000)
    } else {
	i32::from(val)
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
