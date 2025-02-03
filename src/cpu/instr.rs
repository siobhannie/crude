pub struct Instruction(pub u32);

impl Instruction {
    pub fn opcd(&self) -> usize {
	((self.0 >> 26) & 0x3F) as usize
    }

    
    pub fn d(&self) -> usize {
	((self.0 >> 21) & 0x1F) as usize
    }

    pub fn a(&self) -> usize {
	((self.0 >> 16) & 0x1F) as usize
    }

    pub fn simm(&self) -> i16 {
	(self.0 & 0xFFFF) as i16
    }
    
    pub fn uimm(&self) -> u16 {
	(self.0 & 0xFFFF) as u16
    }

    pub fn sec_opcd(&self) -> usize {
	((self.0 >> 1) & 0x3FF) as usize
    }

    pub fn spr(&self) -> usize {
	let spr = ((self.0 >> 11) & 0x3FF) as usize;

	(((spr & 0x1F) << 5) | ((spr >> 5) & 0x1F)) as usize
    }

    pub fn s(&self) -> usize {
	((self.0 >> 21) & 0x1F) as usize
    }

    pub fn sr(&self) -> usize {
	((self.0 >> 16) & 0xF) as usize
    }

    pub fn rc(&self) -> bool {
	(self.0 & 1) != 0
    }

    pub fn me(&self) -> usize {
	((self.0 >> 1) & 0x1F) as usize
    }

    pub fn mb(&self) -> usize {
	((self.0 >> 6) & 0x1F) as usize
    }

    pub fn sh(&self) -> usize {
	((self.0 >> 11) & 0x1F) as usize
    }

    pub fn tbr(&self) -> usize {
	(((self.0 >> 6) & 0x3E0) | ((self.0 >> 16) & 0x1F)) as usize
    }

    pub fn oe(&self) -> bool {
	((self.0 >> 10) & 1) != 0
    }

    pub fn b(&self) -> usize {
	((self.0 >> 11) & 0x1F) as usize
    }

    pub fn crd(&self) -> usize {
	((self.0 >> 23) & 7) as usize
    }

   
    pub fn lk(&self) -> bool {
	(self.0 & 1) != 0
    }

    pub fn aa(&self) -> bool {
	((self.0 >> 1) & 1) != 0
    }

    pub fn bd(&self) -> u16 {
	((self.0 >> 2) & 0x3FFF) as u16
    }

    pub fn bi(&self) -> usize {
	((self.0 >> 16) & 0x1F) as usize
    }

    pub fn bo(&self) -> usize {
	((self.0 >> 21) & 0x1F) as usize
    }

    pub fn li(&self) -> u32 {
	(self.0 >> 2) & 0xFF_FFFF
    }

    pub fn uimm_d(&self) -> u16 {
	(self.0 & 0xFFF) as u16
    }

    pub fn i(&self) -> usize {
	((self.0 >> 12) & 0x7) as usize
    }

    pub fn w(&self) -> bool {
	((self.0 >> 15) & 1) != 0
    }
}
