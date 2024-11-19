use realfft::{num_complex::Complex, num_traits::Zero};

use crate::utils::{self, SimpleLPF};
use nih_plug::nih_log;

pub struct AdaptiveMixer {
    reduction_amount: f32,
    pub lowcut: f32,
    pub highcut: f32,
    pub gate: f32,
    pub exp_mags: Vec<f32>,
    pub reduction: Vec<f32>,
    pub smoothed: Vec<f32>,
    pub lpf: utils::SimpleLPF,
    pub smoothness: f32,
}

impl AdaptiveMixer {
    pub fn new(num_bins: usize) -> Self {
        Self {
            reduction_amount: 0.0,
            lowcut: 20.0,
            highcut: 20_000.0,
            exp_mags: vec![0.0f32; num_bins],
            reduction: vec![0.0f32; num_bins],
            smoothed: vec![0.0f32; num_bins],
            gate: -120.0,
            lpf: SimpleLPF::new(0.001f32),
            smoothness: 0.0f32,
        }
    }

    pub fn resize(&mut self, new_bin_size: usize) {
        self.reduction.resize(new_bin_size, 0.0f32);
        self.exp_mags.resize(new_bin_size, 0.0f32);
        self.smoothed.resize(new_bin_size, 0.0f32);
    }

    pub fn set_params(&mut self, side_gain: f32, low: f32, high: f32, gate: f32, smooth: f32) {
        self.reduction_amount = side_gain;
        self.lowcut = low;
        self.highcut = high;
        self.gate = gate;
        self.smoothness = smooth;
        self.lpf.set_a(smooth);
    }   

    pub fn process_spectrum(&mut self, 
        mag: [&Vec<f32>; 2], 
        phase: [&Vec<f32>; 2], 
        db: [&Vec<f32>; 2], 
        freq: [&Vec<f32>; 2], 
        aux_db: [&Vec<f32>; 2],
        aux_mag: [&Vec<f32>; 2],
        output_buffer: &mut [Vec<Complex<f32>>; 2]) 
    {
        for channel in 0..2 {
            let max = aux_mag[channel].iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().max(utils::db_to_gain(-60.0));

            for (smoothed, (aux_db, aux_mag)) in self.smoothed.iter_mut().zip(aux_db[channel].iter().zip(aux_mag[channel].iter())) {
                *smoothed = if *aux_db > utils::gain_to_db(self.gate) {
                    calc_exp(*aux_mag / max) * self.reduction_amount
                } else {
                    0.0f32
                };
            }
            let len = self.smoothed.len();
            for smoothed in self.smoothed.iter_mut().skip(1).take(len - 2) {
                *smoothed = self.lpf.process(*smoothed);
            }
            for smoothed in self.smoothed.iter_mut().rev().skip(1).take(len - 2) {
                *smoothed = self.lpf.process(*smoothed);
            }

            for (i, ((aux_mag, aux_db), (mag, db))) in aux_mag[channel].iter().zip(aux_db[channel].iter()).zip(mag[channel].iter().zip(db[channel])).enumerate() {
                if freq[channel][i] < self.lowcut || freq[channel][i] > self.highcut {
                    output_buffer[channel][i] = Complex::from_polar(utils::db_to_gain(*db), phase[channel][i]);
                    self.reduction[i] = -120f32;
                    continue;
                } 
                let out_mag: f32;
                let reduction = self.smoothed[i];
                // if *aux_db > utils::gain_to_db(self.gate)  {
                //     calc_exp(*aux_mag / max) * self.reduction_amount
                // } else {
                //     0.0f32
                // };
                //nih_log!("aux_db {}, gate {:.3}db, max: {:.3}, red {:.3}. amount {}db", aux_db, utils::gain_to_db(self.gate), max, reduction, utils::gain_to_db(self.reduction_amount));
                self.reduction[i] = utils::gain_to_db(reduction);

                out_mag = *mag * (1.0 - reduction);

                output_buffer[channel][i] = Complex::from_polar(out_mag, phase[channel][i]);

                // let side_gained: f32 = utils::gain_to_db(aux_mag * self.reduction_amount);
                // // gate - side_gain -> negative value, distance between the values.
                // // abs'ing this will give us the amount to reduce the signal by
                // let side_gain = if side_gained > self.gate {self.gate - side_gained } else {0.0};
                // let out_db = db - side_gain.abs();
                // output_buffer[channel][i] = Complex::from_polar(utils::db_to_gain(out_db), phase[channel][i]);
                
                // self.reduction[i] = side_gain;
            }
            output_buffer[channel][0] = Complex::zero();
            output_buffer[channel][aux_db[0].len() - 1] = Complex::zero();
        }
    }
}

#[inline]
pub fn calc_exp(x: f32) -> f32 {
    x.clamp(0.0, 1.0)//.powi(2)
}