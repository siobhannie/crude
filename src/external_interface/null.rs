use super::EXIDevice;

pub struct NullDevice;


impl EXIDevice for NullDevice {
    fn transfer_byte(&mut self, byte: &mut u8) {
	panic!("transfer_byte called for null device!");
    }

    fn select(&mut self) {
	panic!("select called for null device!");
    }
}
