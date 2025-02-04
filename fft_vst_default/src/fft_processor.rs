use core::{f32, panic};
use std::any::Any;
use std::process::Output;
use std::sync::{Arc, Mutex};

use crate::analyzer_data::AnalyzerData;
use crate::nih_log;
use nih_plug::util;
use realfft::num_complex::Complex;
use realfft::{ComplexToReal, RealToComplex};
use realfft::{num_complex::ComplexFloat, RealFftPlanner};

use crate::{utils, FFT_SIZE, FFT_SIZE_F32, HOP_SIZE, NUM_BINS, WINDOW_CORRECTION};

pub struct FFTProcessor {
    sample_rate: u32,
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
    pos: usize,
    window: Vec<f32>,

    count_to_next_hop: usize,

    planner: RealFftPlanner<f32>,
    r2c: std::sync::Arc<dyn RealToComplex<f32>>,
    c2r: std::sync::Arc<dyn ComplexToReal<f32>>,

    fft_in: Vec<f32>,
    fft_out: Vec<Complex<f32>>,
    ifft_in: Vec<Complex<f32>>,
    ifft_out: Vec<f32>,

    spectrum_mag: Vec<f32>,
    spectrum_phase: Vec<f32>,
    spectrum_freq: Vec<f32>, 
    spectrum_db: Vec<f32>,
    bin_width: f32,

    post_process_buffer: Vec<f32>,

    analyzer_input_data:  Option<triple_buffer::Input<AnalyzerData>>,
}

impl FFTProcessor {
    pub fn new(sample_rate: u32, analyzer_buffer: Option<triple_buffer::Input<AnalyzerData>>) -> Self {
        let window = apodize::hanning_iter(FFT_SIZE).map(|x| x as f32).collect::<Vec<f32>>();

        // all fft stuff
        let mut planner = RealFftPlanner::<f32>::new();
        let r2c = planner.plan_fft_forward(FFT_SIZE);
        let c2r = planner.plan_fft_inverse(FFT_SIZE);
        let fft_in = r2c.make_input_vec();
        let fft_out = r2c.make_output_vec();
        let ifft_in = c2r.make_input_vec();
        let ifft_out = c2r.make_output_vec();

        let bin_width = sample_rate as f32 / FFT_SIZE as f32;
        let time_step = HOP_SIZE as f32 / sample_rate as f32;

        //nih_log!("{} {}", NUM_BINS, ifft_in.len());

        Self {
            input_buffer: vec![0.0f32; FFT_SIZE],
            output_buffer: vec![0.0f32; FFT_SIZE],
            pos: 0,
            count_to_next_hop: 0,
            window: window,
            planner: planner,
            r2c: r2c,
            c2r: c2r,
            fft_in: fft_in,
            fft_out: fft_out, 
            ifft_in: ifft_in, 
            ifft_out: ifft_out,
            sample_rate: sample_rate,
            spectrum_mag: vec![0f32; NUM_BINS],
            spectrum_phase: vec![0f32; NUM_BINS],
            spectrum_freq: vec![0f32; NUM_BINS],
            spectrum_db: vec![0f32; NUM_BINS],
            bin_width: bin_width,
            post_process_buffer: vec![0f32; NUM_BINS],
            analyzer_input_data: analyzer_buffer,
        }
    }

    pub fn process_sample(&mut self, sample: f32) -> f32 {
        self.input_buffer[self.pos] = sample;
        let output_sample = self.output_buffer[self.pos];
        self.output_buffer[self.pos] = 0.0f32;

        self.pos += 1;
        if (self.pos == FFT_SIZE) {
            self.pos = 0;
        }

        self.count_to_next_hop += 1;
        if self.count_to_next_hop == HOP_SIZE {
            self.count_to_next_hop = 0;
            self.process_window();
        }

        output_sample
    }

    pub fn process_window(&mut self) {
        
        let len = FFT_SIZE - self.pos;
        for i in 0..len {
            self.fft_in[i] = self.input_buffer[i + self.pos];
        }

        if self.pos > 0 {
            for i in 0..self.pos {
                self.fft_in[FFT_SIZE - self.pos + i] = self.input_buffer[i];
            }
        }

        utils::multiply_vectors_in_place(&mut self.fft_in, &self.window);

        self.r2c.process(&mut self.fft_in, &mut self.fft_out).unwrap();
        for i in self.fft_out.iter_mut() {
            // * 2.0 (window correction) * 2.0 (one sided fft correction)
            *i = *i * 4.0 / FFT_SIZE_F32;
        }

        // this calculates mag, phase, db and frequencies of each bin
        self.calculate_fft_values();
    
        self.process_spectrum();

        self.calculate_db_for_analyzer();
        let is_some = self.analyzer_input_data.is_some();

        if is_some {
            let analyzer_input = self.analyzer_input_data.as_mut().unwrap().input_buffer();
            analyzer_input.magnitudes[..NUM_BINS].fill(0.0f32);
            analyzer_input.num_bins = NUM_BINS;
            for (i, mag) in analyzer_input.magnitudes[..NUM_BINS].iter_mut().enumerate() {
                *mag = self.spectrum_db[i];
            }
            self.analyzer_input_data.as_mut().unwrap().publish();
        }

        // for (i, bin) in self.fft_out.iter().enumerate() {
        //     self.ifft_in[i] = *bin;
        // }

        self.c2r.process(&mut self.ifft_in, &mut self.ifft_out).unwrap();
        utils::multiply_vectors_in_place(&mut self.ifft_out, &self.window);

        for i in self.ifft_out.iter_mut() {
            *i *= WINDOW_CORRECTION / 4.0;
        }

        for i in 0..self.pos {
            self.output_buffer[i] += self.ifft_out[i + FFT_SIZE - self.pos];
        }
        for i in 0..(FFT_SIZE - self.pos) {
            self.output_buffer[i + self.pos] += self.ifft_out[i];
        }
    }

    fn process_spectrum(&mut self) {
        //self.ifft_in = self.fft_out.clone();
        for (i, x) in self.fft_out.iter().enumerate() {
            self.ifft_in[i] = *x;
        }
        // do yout processing here
    }

    fn calculate_fft_values(&mut self) {
        for i in 1..(self.fft_out.len() - 1) {
            //nih_log!("calculating fft_values: {}", i);
            self.spectrum_mag[i] = self.fft_out[i].norm();
            self.spectrum_phase[i] = self.fft_out[i].arg();
            self.spectrum_db[i] = util::gain_to_db(self.spectrum_mag[i]);
            self.spectrum_freq[i] = (i as u32 * self.sample_rate) as f32 / FFT_SIZE as f32;
        }
    }

    fn calculate_db_for_analyzer(&mut self) {
        for i in 1..(self.ifft_in.len() - 1) {
            self.spectrum_mag[i] = self.ifft_in[i].norm();
            self.spectrum_db[i] = util::gain_to_db(self.spectrum_mag[i]);
        }
    }
}