use core::f32;

use realfft::num_complex::Complex;
use realfft::{ComplexToReal, RealToComplex};
use realfft::{num_complex::ComplexFloat, RealFftPlanner};

use crate::circ_buf::CircBuf;
use crate::colorizer::Colorizer;
use crate::{utils, FFT_SIZE, HOP_SIZE, NUM_BINS, WINDOW_CORRECTION};
//                                  C     C#     D      D#    E      F      F#     G     G#     A      A#     B
const SELECTED_NOTES: [bool; 12] = [true, false, true, true, false, true, false, true, true, false, true, false];

pub struct FFTProcessor {
    fft_in: CircBuf,
    count_to_next_hop: usize,
    window: Vec<f32>,
    planner: RealFftPlanner<f32>,
    r2c: std::sync::Arc<dyn RealToComplex<f32>>,
    c2r: std::sync::Arc<dyn ComplexToReal<f32>>,

    input_buffer: Vec<f32>,
    output_buffer: Vec<Complex<f32>>,
    inverse_input: Vec<Complex<f32>>,
    inverse_output: Vec<f32>,

    spectrum_mag: Vec<f32>,
    spectrum_phase: Vec<f32>,
    spectrum_freq: Vec<f32>, 
    spectrum_db: Vec<f32>,

    post_process_buffer: Vec<f32>,

    sample_rate: u32,

    pub audio_in: Vec<Vec<f32>>,

    audio_out: Vec<f32>,
    audio_pos: usize,

    hop_no: usize,
}

impl FFTProcessor {
    pub fn new(sample_rate: u32) -> Self {
        let buf = CircBuf::new(FFT_SIZE);
        let window = apodize::hanning_iter(FFT_SIZE).map(|x| x as f32).collect::<Vec<f32>>();

        // prepare all the FFT stuff
        let mut planner = RealFftPlanner::<f32>::new();
        let r2c: std::sync::Arc<dyn RealToComplex<f32>> = planner.plan_fft_forward(FFT_SIZE);
        let c2r: std::sync::Arc<dyn ComplexToReal<f32>> = planner.plan_fft_inverse(FFT_SIZE);
        let input_buffer: Vec<f32> = r2c.make_input_vec();
        let output_buffer: Vec<Complex<f32>> = r2c.make_output_vec();
        let inverse_input: Vec<Complex<f32>> = c2r.make_input_vec();
        println!("inverse input len: {}", inverse_input.len());
        let inverse_output: Vec<f32> = c2r.make_output_vec();

        let bin_width = sample_rate as f32 / FFT_SIZE as f32;
        Self {
            fft_in: buf,
            audio_out: Vec::new(),
            audio_in: Vec::new(),
            count_to_next_hop: 0,
            window: window,
            planner: planner,
            r2c: r2c,
            c2r: c2r,
            input_buffer: input_buffer,
            output_buffer: output_buffer,
            inverse_input: inverse_input,
            inverse_output: inverse_output,
            spectrum_mag: vec![0f32; NUM_BINS],
            spectrum_phase: vec![0f32; NUM_BINS],
            spectrum_freq: vec![0f32; NUM_BINS],
            post_process_buffer: vec![0f32; NUM_BINS],
            spectrum_db: vec![0f32; NUM_BINS],
            sample_rate: sample_rate,
            audio_pos: 0,
            hop_no: 0,
        }
    }

    pub fn process_sample(&mut self, sample: f32) {
        self.fft_in.add_sample(sample);
        self.count_to_next_hop += 1;
        self.audio_pos += 1;

        if self.count_to_next_hop == HOP_SIZE {
            if !self.fft_in.was_filled_at_least_once() {
                self.count_to_next_hop = 0;
                return;
            }
            if self.hop_no == 0 {
                self.audio_pos = 0;
            }
            self.hop_no += 1;
            self.count_to_next_hop = 0;
            self.process_window();
        }
    }

    fn process_window(&mut self) {
        self.input_buffer.copy_from_slice(&self.fft_in.get_slice_as_vec().as_slice());
        //multiply by the chosen window
        self.input_buffer = utils::multiply_vectors(&self.input_buffer, &self.window);
        //self.input_buffer.iter().zip(self.window.iter()).map(|(x, y)| x * y).collect::<Vec<f32>>();
        self.audio_in.push(self.input_buffer.to_vec());
        // do fft
        self.r2c.process(&mut self.input_buffer, &mut self.output_buffer).unwrap();

        self.inverse_input = vec![Complex::new(0.0, 0.0); self.inverse_input.len()];

        let norm = FFT_SIZE as f32;

        for i in self.output_buffer.iter_mut() {
            // * 2.0 (window correction) * 2.0 (one sided fft correction)
            *i = *i * 4.0 / norm;
        }

        for (i, bin) in self.output_buffer.iter().enumerate() {
            self.inverse_input[i] = *bin;
        }

        // process your spectrum here
        self.process_spectrum();
        
        // do ifft to get output audio
        
        self.c2r.process(&mut self.inverse_input, &mut self.inverse_output).unwrap();

        self.inverse_output = utils::multiply_vectors(&self.inverse_output, &self.window);

        for i in self.inverse_output.iter_mut() {
            *i *= WINDOW_CORRECTION / 4.0;
        }

        //let mut vec: Vec<f32> = Vec::new();
        // for i in 0..FFT_SIZE {
        //     vec.push(self.inverse_output[i]);
        // }
        // self.audio_out.push(vec);

        for i in 0..FFT_SIZE {
            if self.audio_pos + i >= self.audio_out.len() {
                self.audio_out.push(self.inverse_output[i]);
            } else {
                self.audio_out[self.audio_pos + i] += self.inverse_output[i];
            }
        }
    }

    fn process_spectrum(&mut self) {
        //self.inverse_input[0] = Complex::from_polar(0.0, 0.0);
        for i in 0..NUM_BINS {
            let mut mag = self.output_buffer[i].abs();
            let mut phase = self.output_buffer[i].im.arg();
            let frequency = (i as u32 * self.sample_rate) as f32 / FFT_SIZE as f32;
            
            self.spectrum_mag[i] = mag;
            self.spectrum_phase[i] = phase;
            self.spectrum_freq[i] = frequency;
            self.spectrum_db[i] = utils::f32_to_db(mag);

            if self.spectrum_db[i] > -26.0 {
                self.post_process_buffer[i] = mag;
            } else {
                self.post_process_buffer[i] = mag;
            }

            // mag = mag * self.freq_gain(frequency);
            
            //self.inverse_input[i] = Complex::from_polar(mag, phase);
            
            // if frequency > 200f32 && frequency < 600f32 {
            //     println!("{}Hz: {}dB", frequency, self.spectrum_db[i]);
            // }
            
        }
        //self.post_process_buffer = self.colorizer.process_spectrum(&self.spectrum_mag, &self.spectrum_freq, &self.spectrum_db);

        for (i, (mag, phase)) in self.post_process_buffer.iter().zip(self.spectrum_phase.iter()).enumerate() {
            self.inverse_input[i] = Complex::from_polar(*mag, *phase);
        }
    }

    fn freq_gain(&self, x: f32) -> f32 {
        //(-(x - 20_000f32)).clamp(0f32, f32::MAX).powf(1.0 / 3.0) / 27.2f32
        //( (-x + 20_000f32) / 20_000f32 ).powi(3).max(0f32)
        if x % 440.0 < 40f32 {
            1f32
        } else {
            0f32
        }
    }

    pub fn get_audio(&self) -> Vec<f32> {
        self.audio_out.clone()
    }
}