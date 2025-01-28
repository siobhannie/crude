use super::EXIDevice;

pub struct NullDevice;


impl EXIDevice for NullDevice {
    fn imm_write(&mut self) {
	panic!("imm_write to null device!!");
    }

    fn imm_data_write(&mut self, val: u32) {
	panic!("imm_data_write to null device!!");
    }

    fn imm_read(&mut self) -> u32 {
	panic!("imm_read to null device!!");
    }
}
