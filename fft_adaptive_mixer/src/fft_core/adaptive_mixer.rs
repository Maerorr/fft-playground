use realfft::{num_complex::Complex, num_traits::Zero};

use crate::utils::{self, SimpleLPF};
use nih_plug::nih_log;

const EQ_FREQS: [f32; 9] = [20.0, 189.32, 368.40, 716.87, 1394.95, 2714.41, 5281.95, 10278.08, 20000.0]; 

pub struct AdaptiveMixer {
    reduction_amount: f32,
    pub lowcut: f32,
    pub highcut: f32,
    pub gate: f32,
    pub exp_mags: Vec<f32>,
    pub reduction: Vec<f32>,
    pub peaked: Vec<f32>,
    pub lpf: utils::SimpleLPF,
    pub smoothness: f32,
    pub peakiness: f32,
    pub eq: Vec<f32>,

    pub mags_eq: [f32; 4096],
    pub db_eq: [f32; 4096],

    pub fft_size: usize,
    pub sample_rate: f32,

    pub time_lpf: [SimpleLPF; 4096],
}

impl AdaptiveMixer {
    pub fn new(num_bins: usize, sr: f32) -> Self {
        Self {
            reduction_amount: 0.0,
            lowcut: 20.0,
            highcut: 20_000.0,
            exp_mags: vec![0.0f32; num_bins],
            reduction: vec![0.0f32; num_bins],
            peaked: vec![0.0f32; num_bins],
            gate: -120.0,
            lpf: SimpleLPF::new(0.001f32),
            smoothness: 0.0f32,
            peakiness: 1.0f32,
            eq: vec![0.0f32; 8],

            mags_eq: [0.0f32; 4096],
            db_eq: [0.0f32; 4096],

            fft_size: num_bins * 2,
            sample_rate: sr,
            time_lpf: [SimpleLPF::new(0.0); 4096],
        }
    }

    pub fn resize(&mut self, new_bin_size: usize) {
        self.fft_size = new_bin_size * 2;
        self.reduction.resize(new_bin_size, 0.0f32);
        self.exp_mags.resize(new_bin_size, 0.0f32);
        self.peaked.resize(new_bin_size, 0.0f32);
    }

    pub fn set_params(&mut self, 
        side_gain: f32, 
        low: f32, 
        high: f32, 
        gate: f32, 
        smooth: f32, 
        peakiness: f32,
        time: f32,
        eq1: f32,
        eq2: f32,
        eq3: f32,
        eq4: f32,
        eq5: f32,
        eq6: f32,
        eq7: f32,
        eq8: f32,
    ) {
        self.reduction_amount = side_gain;
        self.lowcut = low;
        self.highcut = high;
        self.gate = gate;
        self.smoothness = smooth;
        self.peakiness = peakiness;
        self.lpf.set_a(smooth);
        if self.time_lpf[0].a != time {
            for time_smooth in self.time_lpf.iter_mut() {
                time_smooth.set_a(time);
            }
        }
        self.eq[0] = eq1;
        self.eq[1] = eq2;
        self.eq[2] = eq3;
        self.eq[3] = eq4;
        self.eq[4] = eq5;
        self.eq[5] = eq6;
        self.eq[6] = eq7;
        self.eq[7] = eq8;
    }   

    pub fn get_max_within_cutoffs(&self, arr: &Vec<f32>, freq: &Vec<f32>) -> f32 {
        let mut max = std::f32::MIN;

        for (el, f) in arr.iter().zip(freq.iter()) {
            if *f < self.lowcut || *f > self.highcut {
                continue;
            }
            if *el > max {
                max = *el;
            }
        }
        max
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
        let one_over_p = 1.0f32 / self.peakiness;
        for channel in 0..2 {
            // FIRST EQ THE AUX SIGNAL WITH OUT SIMPLE 8-BAND STEP EQ
            //nih_log!("db[10] at entry {}", aux_db[channel][10]);
            for (i, eq) in self.eq.iter().enumerate() {
                let bin_min = utils::freq_to_bin(EQ_FREQS[i], self.fft_size, self.sample_rate);
                let bin_max = utils::freq_to_bin(EQ_FREQS[i+1], self.fft_size, self.sample_rate);
                for o in bin_min..bin_max {
                    //self.mags_eq[o] = aux_mag[channel][o] * *eq;
                    // if o == 50  {
                    //     nih_log!("db {} + eq {} = {}", aux_db[channel][o], *eq, aux_db[channel][o] + *eq);
                    // }
                    self.db_eq[o] = aux_db[channel][o] + *eq;
                }
            }
            // Fill the rest of the buffer with bands eq'd with the last high shelf
            for i in utils::freq_to_bin(EQ_FREQS[8], self.fft_size, self.sample_rate)..(aux_mag[channel].len() - 1) {
                //self.mags_eq[i] = aux_mag[channel][i] * self.eq[7];
                self.db_eq[i] = aux_db[channel][i] + self.eq[7];
            }

            // rescale into 0-1
            for (i, db) in self.peaked.iter_mut().enumerate() {
                *db = self.db_eq[i] / 120.0 + 1.0
            }
            //nih_log!("db[100] after 0-1 scaling {}", self.peaked[100]);
            let max = self.get_max_within_cutoffs(&self.peaked, &freq[channel]).max(-90.0);
            // normalize the 0-1 so that the highest peak is equal to 1.0
            for (i, db) in self.peaked.iter_mut().enumerate() {
                *db = *db / max;
            }

            // rescale back to db values -> highest peak is now 0.0, lowest possible is -100
            for (i, db) in self.peaked.iter_mut().enumerate() {
                *db = *db * 100.0 - 100.0;
            }

            // calculate diff between nonpeaked and peaked
            // smoothed now becomes dB DIFFERENCE of peaked and non-peaked
            for (i, peaked) in self.peaked.iter_mut().enumerate() {
                *peaked = utils::peakiness_scaled(*peaked, self.peakiness, one_over_p, -100.0, 100.0, -100.0, 100.0) - *peaked;
            }
            
            for (i, peaked) in self.peaked.iter_mut().enumerate() {
                *peaked = self.db_eq[i] + *peaked;
                if *peaked < utils::gain_to_db(self.gate) {
                    *peaked = -120.0; // mute if below gate
                } else {
                    *peaked += self.reduction_amount; // if above gate, apply db of gain parameter
                } 
                self.reduction[i] = if *peaked > -20.0 {
                    *peaked + 20.0
                } else {
                    0.0f32
                };
            }

            let len = self.reduction.len();
            for red in self.reduction.iter_mut().skip(1).take(len - 2) {
                *red = self.lpf.process(*red);
            }
            for red in self.reduction.iter_mut().rev().skip(1).take(len - 2) {
                *red = self.lpf.process(*red);
            }

            for (i, db) in db[channel].iter().enumerate() {
                if freq[channel][i] < self.lowcut || freq[channel][i] > self.highcut {
                    output_buffer[channel][i] = Complex::from_polar(utils::db_to_gain(*db), phase[channel][i]);
                    self.reduction[i] = self.time_lpf[i].process(0.0f32);
                    continue;
                } 
                self.reduction[i] = self.time_lpf[i].process(self.reduction[i]);
                output_buffer[channel][i] = Complex::from_polar(utils::db_to_gain(db - self.reduction[i]), phase[channel][i]);
            }
            output_buffer[channel][0] = Complex::zero();
            output_buffer[channel][aux_db[0].len() - 1] = Complex::zero();
        }
    }
}

