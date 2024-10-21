use nih_plug::util;

#[derive(Clone)]
pub struct Gate {
    threshold: f32,
}

impl Gate {
    pub fn new() -> Self {
        Self {
            threshold: 0.0f32,
        }
    }

    pub fn new_with_threshold(threshold: f32) -> Self {
        Self {
            threshold,
        }
    }

    pub fn set_gate_params(&mut self, th: f32) {
        self.threshold = th;
    }

    /// takes in a db value of a signal and returns db value of the signal after gating.
    pub fn process(&mut self, val_db: f32) -> f32 {
        // WE ARE WORKING WITH dB REMEMBER THAT 0 = LOUDEST
        let mut out: f32 = val_db;
        if val_db < self.threshold {
            out = -100.0f32;
        }
    
        util::db_to_gain(out)
    }
}
