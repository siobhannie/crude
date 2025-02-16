use std::sync::{atomic::AtomicU16, Arc};

use super::DSPControlRegister;

//this is what's given to the gc in order to communicate with the DSP and vice versa
#[derive(Clone)]
pub struct DSPClient {
    pub control_reg: Arc<DSPControlRegister>,
    pub cpu_mbox_h: Arc<AtomicU16>,
    pub cpu_mbox_l: Arc<AtomicU16>,
    pub dsp_mbox_h: Arc<AtomicU16>,
    pub dsp_mbox_l: Arc<AtomicU16>,
}

impl DSPClient {
    pub fn new() -> Self {
	Self {
	    control_reg: Arc::new(DSPControlRegister(AtomicU16::new(0))),
	    cpu_mbox_h: Arc::new(AtomicU16::new(0)),
	    cpu_mbox_l: Arc::new(AtomicU16::new(0)),
	    dsp_mbox_h: Arc::new(AtomicU16::new(0)),
	    dsp_mbox_l: Arc::new(AtomicU16::new(0)),
	}
    }
}
