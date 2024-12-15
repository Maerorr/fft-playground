use realfft::{num_complex::Complex, num_traits::Zero};

use crate::utils;

use super::{compressor::Compressor, fft_size::FFTSize};

pub struct SpectralMultibandCompressor {
    compressors: [Vec<Compressor>; 2],
    pub low_mid_freq: f32,
    pub mid_high_freq: f32,
    pub low_mid_idx: usize,
    pub mid_high_idx: usize,

    // compressor params for 3 bands, low mid high
    pub low_threshold: f32,
    pub low_gain: f32,
    pub mid_threshold: f32,
    pub mid_gain: f32,
    pub high_threshold: f32,
    pub high_gain: f32,

    pub attack_ms: f32,
    pub release_ms: f32,

    pub hops_per_second: f32,
    pub fft_size: usize,

    pub delta: Vec<f32>,

    pub sample_rate: f32,
}

impl SpectralMultibandCompressor {
    pub fn new(
        low_threshold: f32,
        low_gain: f32,
        mid_threshold: f32,
        mid_gain: f32,
        high_threshold: f32,
        high_gain: f32,
        attack_ms: f32,
        release_ms: f32,
        hops_per_second: f32,
        fft_size: usize,
        low_mid_freq: f32,
        mid_high_freq: f32,
        sample_rate: f32,
    ) -> Self {
        let mut compressors = Vec::with_capacity(fft_size / 2 + 1);
        let attack_coeff = (-1.0 / (attack_ms * hops_per_second * 0.001)).exp();
        let release_coeff = (-1.0 / (release_ms * hops_per_second * 0.001)).exp();
        for _ in 0..(fft_size / 2 + 1) {
            let compressor = Compressor::new(
                low_threshold,
                2.0,
                5.0,
                attack_coeff,
                release_coeff,
            );
            compressors.push(compressor);
        }

        let low_mid_idx = utils::freq_to_bin(low_mid_freq, fft_size , sample_rate);
        let mid_high_idx = utils::freq_to_bin(mid_high_freq, fft_size , sample_rate);

        Self {
            compressors: [compressors.to_vec(), compressors.to_vec()],
            low_threshold,
            low_gain,
            mid_threshold,
            mid_gain,
            high_threshold,
            high_gain,
            attack_ms,
            release_ms,
            hops_per_second,
            fft_size,
            low_mid_freq,
            mid_high_freq,
            low_mid_idx,
            mid_high_idx,
            sample_rate,
            delta: vec![0.0f32; fft_size / 2 + 1]
        }
    }

    pub fn resize(&mut self, fft_size: usize) {
        self.fft_size = fft_size;
        let bin_num = fft_size / 2 + 1;
        
        let attack_coeff = (-1.0 / (self.attack_ms * self.hops_per_second * 0.001)).exp();
        let release_coeff = (-1.0 / (self.release_ms * self.hops_per_second * 0.001)).exp();
        self.delta.resize(bin_num, 0.0f32);
        self.compressors[0].resize(
            bin_num, 
            Compressor::new(
                self.low_threshold,
                0.2,
                5.0,
                attack_coeff,
                release_coeff,
            )
        );
        self.compressors[1].resize(
            bin_num, 
            Compressor::new(
                self.low_threshold,
                2.0,
                5.0,
                attack_coeff,
                release_coeff,
            )
        );
    }

    pub fn set_params(
        &mut self,
        low_threshold: f32,
        low_gain: f32,
        mid_threshold: f32,
        mid_gain: f32,
        high_threshold: f32,
        high_gain: f32,
        attack_ms: f32,
        release_ms: f32,
        hops_per_second: f32,
    ) {
        // check if any parameters changed, if so update only the compressors in said band
        if self.low_threshold != low_threshold {
            for i in 0..self.low_mid_idx {
                self.compressors[0][i].th = low_threshold;
                self.compressors[1][i].th = low_threshold;
            }
        }

        if self.mid_threshold != mid_threshold {
            for i in self.low_mid_idx..self.mid_high_idx {
                self.compressors[0][i].th = mid_threshold;
                self.compressors[1][i].th = mid_threshold;
            }
        }

        if self.high_threshold != high_threshold {
            for i in self.mid_high_idx..(self.fft_size / 2 + 1) {
                self.compressors[0][i].th = high_threshold;
                self.compressors[1][i].th = high_threshold;
            }
        }

        if self.attack_ms != attack_ms || self.release_ms != release_ms {
            let attack_coeff = (-1.0 / (attack_ms * hops_per_second * 0.001)).exp();
            let release_coeff = (-1.0 / (release_ms * hops_per_second * 0.001)).exp();
            for i in 0..(self.fft_size / 2 + 1) {
                self.compressors[0][i].att = attack_coeff;
                self.compressors[0][i].rel = release_coeff;
                self.compressors[1][i].att = attack_coeff;
                self.compressors[1][i].rel = release_coeff;
            }
        }
        self.low_threshold = low_threshold;
        self.low_gain = low_gain;
        self.mid_threshold = mid_threshold;
        self.mid_gain = mid_gain;
        self.high_threshold = high_threshold;
        self.high_gain = high_gain;
        self.attack_ms = attack_ms;
        self.release_ms = release_ms;
        self.hops_per_second = hops_per_second;
    }

    pub fn set_hops_per_second(&mut self, hops_per_second: f32) {
        self.hops_per_second = hops_per_second;
        let attack_coeff = (-1.0 / (self.attack_ms * hops_per_second * 0.001)).exp();
        let release_coeff = (-1.0 / (self.release_ms * hops_per_second * 0.001)).exp();
        for i in 0..(self.fft_size / 2 + 1) {
            self.compressors[0][i].att = attack_coeff;
            self.compressors[0][i].rel = release_coeff;
            self.compressors[1][i].att = attack_coeff;
            self.compressors[1][i].rel = release_coeff;
        }
    }

    pub fn process(
        &mut self,
        mag: [&Vec<f32>; 2], 
        phase: [&Vec<f32>; 2], 
        db: [&Vec<f32>; 2], 
        freq: [&Vec<f32>; 2], 
        output_buffer: &mut [Vec<Complex<f32>>; 2]) 
    {
        for d in self.delta.iter_mut() {
            *d = 0.0f32;
        }
        for channel in 0..2 {
            for (i, db) in db[channel].iter().enumerate() {
                let gain = match freq[channel][i] {
                    f if f < self.low_mid_freq => self.low_gain,
                    f if f < self.mid_high_freq => self.mid_gain,
                    _ => self.high_gain,
                };
                let delta = self.compressors[channel][i].process_db(*db + gain);
                output_buffer[channel][i] = Complex::from_polar(utils::db_to_gain(*db + gain + delta), phase[channel][i]);
                // this will average over both channels
                self.delta[i] += delta / 2f32;
            }
            self.delta[0] = 0.0;
            self.delta[db[0].len() - 1] = 0.0f32;
            output_buffer[channel][0] = Complex::zero();
            output_buffer[channel][db[0].len() - 1] = Complex::zero();
        }
    }
}