use std::sync::{atomic::AtomicBool, Arc};

use nih_plug::{
    nih_log,
    util::{self, gain_to_db},
};
use rand::Rng;
use realfft::{
    num_complex::{Complex, Complex32},
    num_traits::Zero,
};

use crate::{
    analyzer_data::{AnalyzerChannel, AnalyzerData},
    utils::{self, fft_size_to_bins},
    WINDOW_CORRECTION,
};

use super::{compressor::Compressor, fft_data::FFTData, fft_size::FFTSize, spectral_multiband_compressor::SpectralMultibandCompressor};

pub struct StereoFFTProcessor {
    input_buffer: [Vec<f32>; 2],
    output_buffer: [Vec<f32>; 2],

    pub window: Vec<f32>,

    pub pos: usize,
    pub count_to_next_hop: usize,
    pub hop_size: usize,

    pub in_gain: f32,
    pub out_gain: f32,

    sample_rate: usize,

    data: [FFTData; 2],
    ifft_in: [Vec<Complex<f32>>; 2],

    fft_size: usize,
    analyzer_input_data: triple_buffer::Input<AnalyzerData>,
    analyzer_channel: AnalyzerChannel,

    size_changed: Arc<AtomicBool>,

    pub fft_effect: SpectralMultibandCompressor,
}

unsafe impl Send for StereoFFTProcessor {}
unsafe impl Sync for StereoFFTProcessor {}

impl StereoFFTProcessor {
    pub fn new(
        sample_rate: usize,
        fft_size: usize,
        size_changed: Arc<AtomicBool>,
        analyzer_buffer: triple_buffer::Input<AnalyzerData>,
    ) -> Self {
        let window = apodize::hanning_iter(fft_size)
            .map(|x| x as f32)
            .collect::<Vec<f32>>();
        let data1 = FFTData::new(fft_size);
        let data2 = FFTData::new(fft_size);
        let ifft_in = data1.c2r.make_input_vec();

        Self {
            input_buffer: [vec![0f32; fft_size], vec![0f32; fft_size]],
            output_buffer: [vec![0f32; fft_size], vec![0f32; fft_size]],

            window,

            pos: 0,
            count_to_next_hop: 0,
            hop_size: fft_size / 4,

            in_gain: 1.0,
            out_gain: 1.0,

            sample_rate,

            data: [data1, data2],
            ifft_in: [ifft_in.to_vec(), ifft_in.to_vec()],

            fft_size,
            analyzer_input_data: analyzer_buffer,
            analyzer_channel: AnalyzerChannel::Merged,

            size_changed,

            fft_effect: SpectralMultibandCompressor::new(
                -20.0,
                0.0,
                -20.0,
                0.0,
                -20.0,
                0.0,
                10.0,
                100.0,
                10.0,
                fft_size,
                300.0,
                3500.0,
                sample_rate as f32,
            ),
        }
    }

    pub fn set_params(
        &mut self, 
        an_chan: AnalyzerChannel, 
        low_th: f32, 
        low_r: f32,
        low_up_r: f32,
        low_g: f32, 
        mid_th: f32,
        mid_r: f32,
        mid_up_r: f32, 
        mid_g: f32, 
        high_th: f32, 
        high_r: f32,
        high_up_r: f32,
        high_g: f32,
        attack_ms: f32,
        release_ms: f32,
        mix: f32,
        in_gain: f32,
        out_gain: f32,
    ) {
        self.analyzer_channel = an_chan;
        self.in_gain = in_gain;
        self.out_gain = out_gain;
        
        self.fft_effect.set_params(
            low_th, 
            low_r,
            low_up_r,
            low_g, 
            mid_th, 
            mid_r,
            mid_up_r,
            mid_g, 
            high_th, 
            high_r,
            high_up_r,
            high_g, 
            attack_ms, 
            release_ms, 
            self.sample_rate as f32 / self.hop_size as f32,
            mix,
        );
    }

    pub fn set_sample_rate(&mut self, sr: usize) {
        self.sample_rate = sr;
        self.fft_effect.set_hops_per_second(sr as f32 / self.hop_size as f32);
    }

    pub fn change_fft_size(&mut self, new_size: usize) {
        self.window = apodize::hanning_iter(new_size)
            .map(|x| x as f32)
            .collect::<Vec<f32>>();

        self.data[0].fft_size_change(new_size);
        self.data[1].fft_size_change(new_size);

        self.fft_size = new_size;

        self.input_buffer[0].resize(new_size, 0f32);
        self.output_buffer[0].resize(new_size, 0f32);

        self.input_buffer[1].resize(new_size, 0f32);
        self.output_buffer[1].resize(new_size, 0f32);

        self.ifft_in[0].resize(fft_size_to_bins(new_size), Complex::zero());
        self.ifft_in[1].resize(fft_size_to_bins(new_size), Complex::zero());

        self.ifft_in[0].fill(Complex::zero());
        self.ifft_in[1].fill(Complex::zero());

        self.pos = 0;
        self.count_to_next_hop = 0;
        self.hop_size = new_size / 4;

        self.fft_effect.resize(new_size);
    }

    pub fn process_sample(&mut self, samples_lr: [f32; 2]) -> [f32; 2] {
        let mut output = [0f32, 0f32];

        // copy each sample into l/r buffers
        for (channel, sample) in samples_lr.iter().enumerate() {
            self.input_buffer[channel][self.pos] = *sample * self.in_gain;
            output[channel] = self.output_buffer[channel][self.pos] * self.out_gain;
            self.output_buffer[channel][self.pos] = 0f32;
        }

        //increment the buffer position. Both buffers use the same position
        self.pos += 1;
        if self.pos == self.fft_size {
            self.pos = 0;
        }

        self.count_to_next_hop += 1;
        if self.count_to_next_hop == self.hop_size {
            self.count_to_next_hop = 0;
            self.process_windows();
        }

        output 
    }

    pub fn process_windows(&mut self) {
        // stereo
        for channel in 0..2 {
            // properly copy the input buffer and make it continous
            let len = self.fft_size - self.pos;
            for i in 0..len {
                self.data[channel].fft_in[i] = self.input_buffer[channel][i + self.pos];
            }

            if self.pos > 0 {
                for i in 0..self.pos {
                    self.data[channel].fft_in[self.fft_size - self.pos + i] =
                        self.input_buffer[channel][i];
                }
            }

            // multiply the input vector by a window to prevent spectral leakage
            utils::multiply_vectors_in_place(&mut self.data[channel].fft_in, &self.window);

            // do forward FFT
            self.data[channel]
                .r2c
                .process(
                    &mut self.data[channel].fft_in,
                    &mut self.data[channel].fft_out,
                )
                .unwrap();

            let fft_sizef32 = self.fft_size as f32;
            // window and one-sided fft correction
            for i in self.data[channel]
                .fft_out
                .iter_mut()
            {
                // * 2.0 (window correction) * 2.0 (one sided fft correction)
                *i = *i * 4.0 / fft_sizef32;
            }
        }

        // calculate values for processing (magnitude, phase, magnitude in dB and bin frequencies)
        self.calculate_fft_values();
        // MAIN FFT-BASED PROCESSING
        self.process_spectrum();

        self.calculate_analyzer_db();

        self.handle_analyzer();

        // inverse FFT from processed bins
        for channel in 0..2 {
            self.data[channel]
                .c2r
                .process(&mut self.ifft_in[channel], &mut self.data[channel].ifft_out)
                .unwrap();
            utils::multiply_vectors_in_place(&mut self.data[channel].ifft_out, &self.window);

            for i in self.data[channel].ifft_out.iter_mut() {
                *i *= WINDOW_CORRECTION / 4.0;
            }

            for i in 0..self.pos {
                self.output_buffer[channel][i] +=
                    self.data[channel].ifft_out[i + self.fft_size - self.pos];
            }
            for i in 0..(self.fft_size - self.pos) {
                self.output_buffer[channel][i + self.pos] += self.data[channel].ifft_out[i];
            }
        }
    }

    fn calculate_fft_values(&mut self) {
        for channel in 0..2 {
            for i in 1..(self.data[channel].fft_out.len() - 1) {
                self.data[channel].spectrum_mag[i] = self.data[channel].fft_out[i].norm();
                self.data[channel].spectrum_phase[i] = self.data[channel].fft_out[i].arg();
                self.data[channel].spectrum_db[i] =
                    util::gain_to_db(self.data[channel].spectrum_mag[i]);
                self.data[channel].spectrum_freq[i] =
                    (i * self.sample_rate) as f32 / self.fft_size as f32;
            }

            // let db1 = -5.0;
            // let db2 = -10.0;
            // let db3 = -40.0;
            // let db4 = -20.0;

            // self.data[channel].spectrum_mag[10] = utils::db_to_gain(db1);
            // self.data[channel].spectrum_mag[11] = utils::db_to_gain(db2);
            // self.data[channel].spectrum_mag[12] = utils::db_to_gain(db3);
            // self.data[channel].spectrum_mag[50] = utils::db_to_gain(db4);

            // self.data[channel].spectrum_db[10] = db1;
            // self.data[channel].spectrum_db[11] = db2;
            // self.data[channel].spectrum_db[12] = db3;
            // self.data[channel].spectrum_db[50] = db4;
        }
    }

    fn process_spectrum(&mut self) {
        self.fft_effect.process(
            [&self.data[0].spectrum_mag, &self.data[1].spectrum_mag],
            [&self.data[0].spectrum_phase, &self.data[1].spectrum_phase],
            [&self.data[0].spectrum_db, &self.data[1].spectrum_db],
            [&self.data[0].spectrum_freq, &self.data[1].spectrum_freq],
            &mut self.ifft_in,
        );

        // for channel in 0..2 {
        //     for (i, (mag, phase)) in self.data[channel].spectrum_mag.iter().zip(self.data[channel].spectrum_phase.iter()).enumerate() {
        //         self.ifft_in[channel][i] = Complex::from_polar(*mag, *phase);
        //     }
        // }
    }

    fn calculate_analyzer_db(&mut self) {
        for channel in 0..2 {
            for i in 1..(utils::fft_size_to_bins(self.fft_size) - 1) {
                self.data[channel].spectrum_mag[i] = self.ifft_in[channel][i].norm();
                self.data[channel].spectrum_db[i] = util::gain_to_db(self.data[channel].spectrum_mag[i]);
            }
        }
    }

    fn handle_analyzer(&mut self) {
        let analyzer_input = self.analyzer_input_data.input_buffer();
        analyzer_input.magnitudes.fill(0.0f32);
        analyzer_input.num_bins = utils::fft_size_to_bins(self.fft_size);

        for (i, mag) in analyzer_input.magnitudes[0..utils::fft_size_to_bins(self.fft_size)]
            .iter_mut()
            .enumerate()
        {
            *mag = (self.data[0].spectrum_db[i] + self.data[1].spectrum_db[i]) / 2f32;
        }

        self.fft_effect.get_curve(&mut analyzer_input.comp_curve_low, 0);
        self.fft_effect.get_curve(&mut analyzer_input.comp_curve_mid, 1);
        self.fft_effect.get_curve(&mut analyzer_input.comp_curve_high, 2);

        for (i, delta) in analyzer_input.delta[0..utils::fft_size_to_bins(self.fft_size)].iter_mut().enumerate() {
            *delta = self.fft_effect.delta[i];
        }

        for (i, f) in analyzer_input.frequencies[0..utils::fft_size_to_bins(self.fft_size)]
            .iter_mut()
            .enumerate()
        {
            *f = self.data[0].spectrum_freq[i];
        }

        self.analyzer_input_data.publish();
    }
}
