pub enum GateState {
    Closed,
    Opening,
    Opened,
    Closing,
}

pub struct Gate {
    state: GateState,
    gain: f32,
    attack_ms: f32,
    release_ms: f32,
    time_step: f32,
    threshold: f32,
    sample_rate: usize,
    current_time: f32,
}

impl Gate {
    pub fn new(sample_rate: usize, times_step: f32) -> Self {
        Self {
            state: GateState::Closed,
            gain: 0.0f32,
            attack_ms: 0.0f32,
            release_ms: 0.0f32,
            threshold: 0.0f32,
            sample_rate: sample_rate,
            time_step: times_step,
            current_time: 0.0f32,
        }
    }

    pub fn set_gate_params(&mut self, th: f32, att: f32, rel: f32) {
        self.threshold = th;
        self.attack_ms = att;
        self.release_ms = rel;
    }

    /// takes in a db value of a signal and returns db value of the signal after gating.
    pub fn process(&mut self, val_db: f32) -> f32 {
        let out: f32;
        match self.state {
            GateState::Closed => {
                if val_db > self.threshold {
                    self.state = GateState::Opening;
                }
                out = val_db * 0.0f32; // Muted when closed
            },
            GateState::Opening => {
                let attack_time_sec = self.attack_ms / 1000.0;
                if attack_time_sec > 0.0 {
                    self.gain += self.time_step / attack_time_sec;
                    if self.gain >= 1.0 {
                        self.gain = 1.0;
                        self.state = GateState::Opened;
                    }
                } else {
                    // Instant attack
                    self.gain = 1.0;
                    self.state = GateState::Opened;
                }
                out = val_db * self.gain;
            },
            GateState::Opened => {
                if val_db < self.threshold {
                    self.state = GateState::Closing;
                }
                out = val_db * self.gain; // Pass through when opened
            },
            GateState::Closing => {
                let release_time_sec = self.release_ms / 1000.0;
                if release_time_sec > 0.0 {
                    self.gain -= self.time_step / release_time_sec;
                    if self.gain <= 0.0 {
                        self.gain = 0.0;
                        self.state = GateState::Closed;
                    }
                } else {
                    // Instant release
                    self.gain = 0.0;
                    self.state = GateState::Closed;
                }
                out = val_db * self.gain;
            },
        }
    
        out
    }
}
