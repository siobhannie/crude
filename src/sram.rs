use std::mem::ManuallyDrop;

use zerocopy::*;

#[repr(C)]
pub union Sram {
    byte_array: [u8; 0x44],
    actual: ManuallyDrop<SramImpl>,
}

impl Sram {
    pub fn new() -> Self {
	assert_eq!(size_of::<SramImpl>(), 0x44);
	Self {
	    actual: ManuallyDrop::new(SramImpl {
		rtc: big_endian::U32::ZERO,
		checksum: big_endian::U16::from(0x2c),
		checksum_inv: big_endian::U16::from(0x2c),
		ead0: 0,
		ead1: 0,
		rtc_bias: 0,
		si_horizontal_offset: 0,
		ntd: 0,
		language: 0,
		flags: 0x20,
		card_flash_id: [[b'D', b'O', b'L', b'P', b'H', b'I', b'N', b'S', b'L', b'O', b'T', b'A'], [b'D', b'O', b'L', b'P', b'H', b'I', b'N', b'S', b'L', b'O', b'T', b'B']],
		wireless_kbd_id: 0,
		wireless_pad_id: [0; 4],
		di_error_code: 0,
		field_25: 0,
		flash_id_checksum: [0x6E, 0x6D],
		gbs_mode: 0,
		field_3e: [0; 2],
	    })
	}
    }

    pub fn as_byte_array(&self) -> &[u8; 0x44] {
	unsafe {
	    &self.byte_array
	}
    }

    pub fn as_byte_array_mut(&mut self) -> &mut [u8; 0x44] {
	unsafe {
	    &mut self.byte_array
	}
    }

    pub fn as_struct(&self) -> &SramImpl {
	unsafe {
	    &self.actual
	}
    }

    pub fn as_struct_mut(&mut self) -> &mut SramImpl {
	unsafe {
	    &mut self.actual
	}
    }
}

impl Drop for Sram {
    fn drop(&mut self) {
        unsafe {
	    //i think this is right?????? i've never used ManuallyDrop<T> before...
	    ManuallyDrop::drop(&mut self.actual);
	}
    }
}

#[repr(C)]
pub struct SramImpl {
    pub rtc: big_endian::U32,
    pub checksum: big_endian::U16,
    pub checksum_inv: big_endian::U16,
    pub ead0: u32,
    pub ead1: u32,
    pub rtc_bias: u32,
    pub si_horizontal_offset: i8,
    pub ntd: u8,
    pub language: u8,
    pub flags: u8,
    pub card_flash_id: [[u8; 12]; 2],
    pub wireless_kbd_id: u32,
    pub wireless_pad_id: [u16; 4],
    pub di_error_code: u8,
    pub field_25: u8,
    pub flash_id_checksum: [u8; 2],
    pub gbs_mode: u16,
    pub field_3e: [u8; 2],
}
