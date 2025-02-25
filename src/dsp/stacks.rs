pub struct DSPStacks {
    pub stacks: [[u16; 0x20]; 4],
    pub pointers: [usize; 4],
}

impl DSPStacks {
    pub fn new() -> Self {
	Self {
	    stacks: [[0; 0x20]; 4],
	    pointers: [0; 4],
	}
    }
}
