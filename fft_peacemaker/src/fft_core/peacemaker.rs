use realfft::num_complex::Complex;

use crate::utils;
use nih_plug::nih_log;

pub struct Peacemaker {
    sidechain_gain: f32,
    pub lowcut: f32,
    pub highcut: f32,
    stereo_link: bool,
    pub reduction: Vec<f32>,
}

impl Peacemaker {
    pub fn new(num_bins: usize) -> Self {
        Self {
            sidechain_gain: 0.0,
            lowcut: 20.0,
            highcut: 20_000.0,
            stereo_link: false,
            reduction: vec![0.0f32; num_bins],
        }
    }

    pub fn resize(&mut self, new_bin_size: usize) {
        self.reduction.resize(new_bin_size, 0.0f32);
    }

    pub fn set_params(&mut self, side_gain: f32, low: f32, high: f32, s_link: bool) {
        self.sidechain_gain = side_gain;
        self.lowcut = low;
        self.highcut = high;
        self.stereo_link = s_link;
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
        if self.stereo_link {
            for (i, ((db_l, db_r), (aux_mag_l, aux_mag_r))) in db[0].iter().zip(db[1].iter()).zip(aux_mag[0].iter().zip(aux_mag[1].iter())).enumerate() {
                if freq[0][i] < self.lowcut || freq[0][i] > self.highcut {
                    output_buffer[0][i] = Complex::from_polar(utils::db_to_gain(*db_l), phase[0][i]);
                    output_buffer[1][i] = Complex::from_polar(utils::db_to_gain(*db_r), phase[1][i]);
                    continue;
                }
                // stereo-link meaning the sidechain input is averaged between left and right
                let x = utils::gain_to_db((aux_mag_l + aux_mag_r) / 2.0 * self.sidechain_gain);
                let side_gain = if x > -50.0 {50.0 + x } else {0.0};
                let out_db_l = db_l - side_gain;
                let out_db_r = db_r - side_gain;
                self.reduction[i] = side_gain;
                output_buffer[0][i] = Complex::from_polar(utils::db_to_gain(out_db_l), phase[0][i]);
                output_buffer[1][i] = Complex::from_polar(utils::db_to_gain(out_db_r), phase[1][i]);
            }
        } else {
            for channel in 0..2 {
                for (i, (db, aux_mag)) in db[channel].iter().zip(aux_mag[channel].iter()).enumerate() {
                    if freq[channel][i] < self.lowcut || freq[channel][i] > self.highcut {
                        output_buffer[channel][i] = Complex::from_polar(utils::db_to_gain(*db), phase[channel][i]);

                        if i == 0 {
                            self.reduction[i] = 0f32;
                        } else {
                            self.reduction[i] = (self.reduction[i]) / 2.0f32;
                        }

                        continue;
                    } 
                    let x = utils::gain_to_db(aux_mag * self.sidechain_gain);
                    let side_gain = if x > -50.0 {50.0 + x } else {0.0};
                    let out_db = db - side_gain;
                    output_buffer[channel][i] = Complex::from_polar(utils::db_to_gain(out_db), phase[channel][i]);

                    if i == 0 {
                        self.reduction[i] = side_gain;
                    } else {
                        self.reduction[i] = (self.reduction[i] + side_gain) / 2.0f32;
                    }
                }
            }
            
        }
        
    }
}

pub fn sidechain_gain_calc(x: f32) -> f32 {
    (- ((x + 120.0) / 24.0).powf(3.0)).clamp(-120.0, 0.0)
}