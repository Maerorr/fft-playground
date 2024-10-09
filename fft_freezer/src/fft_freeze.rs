use crate::NUM_BINS;

pub struct FFTFreeze {
    pub mag_freeze: [f32; NUM_BINS],
    pub phase_freeze: [f32; NUM_BINS]
}

impl FFTFreeze {
    pub fn new() -> Self {
        Self {
            mag_freeze: [0.0f32; NUM_BINS],
            phase_freeze: [0.0f32; NUM_BINS],
        }
    }
}