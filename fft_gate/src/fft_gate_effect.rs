use realfft::num_complex::Complex;

use crate::{gate::Gate, utils};

pub struct FFTGateEffect {
    gates: Vec<Gate>,
    threshold: f32,
}

impl FFTGateEffect {
    pub fn new(size: usize) -> Self {
        Self {
            gates: vec![Gate::new(); size],
            threshold: -100f32,
        }
    }

    pub fn resize(&mut self, size: usize) {
        self.gates.resize(size, Gate::new_with_threshold(self.threshold));
    }

    pub fn set_threshold(&mut self, th: f32) {
        self.threshold = th;
        for gate in self.gates.iter_mut() {
            gate.set_gate_params(th);
        }
    }

    pub fn process_spectrum(&mut self, 
        mag: [&Vec<f32>; 2], 
        phase: [&Vec<f32>; 2], 
        db: [&Vec<f32>; 2], 
        freq: [&Vec<f32>; 2], 
        output_buffer: &mut [Vec<Complex<f32>>; 2])
    {
        let len = output_buffer.len() - 1;
        output_buffer[0][0] = Complex::from_polar(0f32, 0f32);
        output_buffer[1][0] = Complex::from_polar(0f32, 0f32);
        output_buffer[0][len] = Complex::from_polar(0f32, 0f32);
        output_buffer[1][len] = Complex::from_polar(0f32, 0f32);
        for channel in 0..2 {
            for i in 1..(output_buffer[0].len() - 1) {
                let mut out: f32 = db[channel][i];
                if db[channel][i] < self.threshold {
                    out = -100.0f32;
                }
                output_buffer[channel][i] = Complex::from_polar(
                    utils::db_to_gain(out),
                     phase[channel][i]
                    );
            }
        }
    }
}
