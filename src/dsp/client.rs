use std::sync::{atomic::AtomicU16, Arc};

use super::DSPControlRegister;

//this is what's given to the gc in order to communicate with the DSP and vice versa
#[derive(Clone)]
pub struct DSPClient {
    pub control_reg: Arc<DSPControlRegister>,
}

impl DSPClient {
    pub fn new() -> Self {
	Self {
	    control_reg: Arc::new(DSPControlRegister(AtomicU16::new(0))),
	}
    }
}
