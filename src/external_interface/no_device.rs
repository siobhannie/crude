use log::info;

use super::EXIDevice;

pub struct NoDevice;

impl EXIDevice for NoDevice {
    fn transfer_byte(&mut self, byte: &mut u8) {
	info!("transfered byte to no device!");
    }

    fn select(&mut self) {
        info!("selected no device!");
    }
}
