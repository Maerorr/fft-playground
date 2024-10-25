use realfft::num_complex::Complex;

use crate::utils;
use nih_plug::nih_log;

pub struct Peacemaker {
    sidechain_gain: f32,
    lowcut: f32,
    highcut: f32,
}

impl Peacemaker {
    pub fn new() -> Self {
        Self {
            sidechain_gain: 0.0,
            lowcut: 20.0,
            highcut: 20_000.0,
        }
    }

    pub fn set_params(&mut self, side_gain: f32, low: f32, high: f32) {
        self.sidechain_gain = side_gain;
        self.lowcut = low;
        self.highcut = high;
        //nih_log!("{}, {}, {}", self.sidechain_gain, self.lowcut, self.highcut);
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
            for (i, (mag, aux_mag)) in mag[channel].iter().zip(aux_mag[channel].iter()).enumerate() {
                //let mag = utils::db_to_gain(db + (utils::gain_to_db(self.sidechain_gain * aux_mag) - 120.0).clamp(-120.0, 0.0));
                //nih_log!("{}", utils::db_to_gain(sidechain_gain_calc(utils::gain_to_db(self.sidechain_gain * 0.5))));
                //let out_db = db * (1.0 - (self.sidechain_gain * aux_db).clamp(0.0, 1.0));
                let side_gain = utils::db_to_gain(sidechain_gain_calc(utils::gain_to_db(self.sidechain_gain * aux_mag)));
                let out_mag = mag * side_gain;
                output_buffer[channel][i] = Complex::from_polar(out_mag, phase[channel][i]);
            }
        }
    }
}

pub fn sidechain_gain_calc(x: f32) -> f32 {
    (- ((x + 120.0) / 24.0).powf(3.0)).clamp(-120.0, 0.0)
}