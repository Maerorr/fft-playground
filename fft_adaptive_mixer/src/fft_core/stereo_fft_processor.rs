use std::sync::{atomic::AtomicBool, Arc};

use nih_plug::{nih_log, util::{self, gain_to_db}};
use rand::Rng;
use realfft::{num_complex::{Complex, Complex32}, num_traits::Zero};

use crate::{analyzer_data::{AnalyzerChannel, AnalyzerData}, utils::{self, fft_size_to_bins}, WINDOW_CORRECTION};

use super::{adaptive_mixer::AdaptiveMixer, fft_data::FFTData, fft_size::FFTSize};

pub struct StereoFFTProcessor {
    input_buffer: [Vec<f32>; 2],
    aux_buffer: [Vec<f32>; 2],
    output_buffer: [Vec<f32>; 2],

    pub window: Vec<f32>,

    pub pos: usize,
    pub count_to_next_hop: usize,

    sample_rate: usize,

    data: [FFTData; 2],
    aux_data: [FFTData; 2],
    ifft_in: [Vec<Complex<f32>>; 2],

    fft_size: usize,
    analyzer_input_data:  triple_buffer::Input<AnalyzerData>,
    analyzer_channel: AnalyzerChannel,

    size_changed: Arc<AtomicBool>,

    smooth: f32,
    peakiness: f32,
    eq: Vec<f32>,

    pub fft_effect: AdaptiveMixer,
}

unsafe impl Send for StereoFFTProcessor {}
unsafe impl Sync for StereoFFTProcessor {}

impl StereoFFTProcessor {
    pub fn new(sample_rate: usize, fft_size: usize, size_changed: Arc<AtomicBool>, analyzer_buffer: triple_buffer::Input<AnalyzerData>) -> Self {
        let window = apodize::hanning_iter(fft_size).map(|x| x as f32).collect::<Vec<f32>>();
        let data1 = FFTData::new(fft_size);
        let data2 = FFTData::new(fft_size);
        let aux_data1 = FFTData::new(fft_size);
        let aux_data2 = FFTData::new(fft_size);
        let ifft_in = data1.c2r.make_input_vec();

        Self {
            input_buffer: [vec![0f32; fft_size], vec![0f32; fft_size]],
            aux_buffer: [vec![0f32; fft_size], vec![0f32; fft_size]],
            output_buffer: [vec![0f32; fft_size], vec![0f32; fft_size]],

            window,

            pos: 0,
            count_to_next_hop: 0,

            sample_rate,

            data: [data1, data2],
            aux_data: [aux_data1, aux_data2],
            ifft_in: [ifft_in.to_vec(), ifft_in.to_vec()],

            fft_size,
            analyzer_input_data: analyzer_buffer,
            analyzer_channel: AnalyzerChannel::Merged,

            size_changed,
            smooth: 0.0,
            peakiness: 1.0f32,
            eq: vec![0.0f32; 8],

            fft_effect: AdaptiveMixer::new(fft_size_to_bins(fft_size), sample_rate as f32),
        }
    }

    pub fn set_params(&mut self, 
        reduction_amount: f32, 
        low: f32, 
        high: f32, 
        gate: f32, 
        smooth: f32, 
        peakiness: f32,
        eq1: f32,
        eq2: f32,
        eq3: f32,
        eq4: f32,
        eq5: f32,
        eq6: f32,
        eq7: f32,
        eq8: f32,
        an_chan: AnalyzerChannel) 
    {
        self.analyzer_channel = an_chan;
        self.smooth = smooth;
        self.peakiness = peakiness;
        self.fft_effect.set_params(
            reduction_amount, 
            low, 
            high,
            gate, 
            smooth,
            peakiness,
            eq1,
            eq2,
            eq3,
            eq4,
            eq5,
            eq6,
            eq7,
            eq8,
        );
        self.eq[0] = eq1;
        self.eq[1] = eq2;
        self.eq[2] = eq3;
        self.eq[3] = eq4;
        self.eq[4] = eq5;
        self.eq[5] = eq6;
        self.eq[6] = eq7;
        self.eq[7] = eq8;
    }

    pub fn set_sample_rate(&mut self, sr: usize) {
        self.sample_rate = sr;
    }

    pub fn change_fft_size(&mut self, new_size: usize) {
        self.window = apodize::hanning_iter(new_size).map(|x| x as f32).collect::<Vec<f32>>();

        self.data[0].fft_size_change(new_size);
        self.data[1].fft_size_change(new_size);

        self.aux_data[0].fft_size_change(new_size);
        self.aux_data[1].fft_size_change(new_size);
    
        self.fft_size = new_size;

        self.input_buffer[0].resize(new_size, 0f32);
        self.output_buffer[0].resize(new_size, 0f32);
        self.aux_buffer[0].resize(new_size, 0f32);

        self.input_buffer[1].resize(new_size, 0f32);
        self.output_buffer[1].resize(new_size, 0f32);
        self.aux_buffer[1].resize(new_size, 0f32);

        self.ifft_in[0].resize(fft_size_to_bins(new_size), Complex::zero());
        self.ifft_in[1].resize(fft_size_to_bins(new_size), Complex::zero());

        self.ifft_in[0].fill(Complex::zero());
        self.ifft_in[1].fill(Complex::zero());

        self.pos = 0;
        self.count_to_next_hop = 0;

        self.fft_effect.resize(fft_size_to_bins(new_size));
    }

        pub fn process_sample(&mut self, samples_lr: [f32; 2], aux_samples_lr: [f32; 2]) -> [f32; 2] {
        let mut output = [0f32, 0f32];

        // copy each sample into l/r buffers
        for (channel, sample) in samples_lr.iter().enumerate() {
            self.input_buffer[channel][self.pos] = *sample;
            output[channel] = self.output_buffer[channel][self.pos];
            self.output_buffer[channel][self.pos] = 0f32;
        }
        
        for (channel, sample) in aux_samples_lr.iter().enumerate() {
            self.aux_buffer[channel][self.pos] = *sample;
        }

        //increment the buffer position. Both buffers use the same position
        self.pos += 1;
        if self.pos == self.fft_size {
            self.pos = 0;
        }

        self.count_to_next_hop += 1;
        if self.count_to_next_hop == self.fft_size / 4 {
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
                self.aux_data[channel].fft_in[i] = self.aux_buffer[channel][i + self.pos];
            }

            if self.pos > 0 {
                for i in 0..self.pos {
                    self.data[channel].fft_in[self.fft_size - self.pos + i] = self.input_buffer[channel][i];
                    self.aux_data[channel].fft_in[self.fft_size - self.pos + i] = self.aux_buffer[channel][i];
                }
            }

            // multiply the input vector by a window to prevent spectral leakage
            utils::multiply_vectors_in_place(&mut self.data[channel].fft_in, &self.window);
            utils::multiply_vectors_in_place(&mut self.aux_data[channel].fft_in, &self.window);

            // do forward FFT
            self.data[channel].r2c.process(&mut self.data[channel].fft_in, &mut self.data[channel].fft_out).unwrap();
            self.aux_data[channel].r2c.process(&mut self.aux_data[channel].fft_in, &mut self.aux_data[channel].fft_out).unwrap();
            let fft_sizef32 = self.fft_size as f32;
            // window and one-sided fft correction
            for (i, o) in self.data[channel].fft_out.iter_mut().zip(self.aux_data[channel].fft_out.iter_mut()) {
                // * 2.0 (window correction) * 2.0 (one sided fft correction)
                *i = *i * 4.0 / fft_sizef32;
                *o = *o * 4.0 / fft_sizef32;
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
            self.data[channel].c2r.process(&mut self.ifft_in[channel], &mut self.data[channel].ifft_out).unwrap();
            utils::multiply_vectors_in_place(&mut self.data[channel].ifft_out, &self.window);

            for i in self.data[channel].ifft_out.iter_mut() {
                *i *= WINDOW_CORRECTION / 4.0;
            }

            for i in 0..self.pos {
                self.output_buffer[channel][i] += self.data[channel].ifft_out[i + self.fft_size - self.pos];
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
                self.data[channel].spectrum_db[i] = util::gain_to_db(self.data[channel].spectrum_mag[i]);
                self.data[channel].spectrum_freq[i] = (i * self.sample_rate) as f32 / self.fft_size as f32;
            }
            for i in 1..(self.aux_data[channel].fft_out.len() - 1) {
                self.aux_data[channel].spectrum_mag[i] = self.aux_data[channel].fft_out[i].norm();
                self.aux_data[channel].spectrum_db[i] = util::gain_to_db(self.aux_data[channel].spectrum_mag[i]);
            }

            let db1 = -5.0;
            let db2 = -10.0;
            let db3 = -40.0;
            let db4 = -20.0;

            self.aux_data[channel].spectrum_mag[10] = utils::db_to_gain(db1);
            self.aux_data[channel].spectrum_mag[11] = utils::db_to_gain(db2);
            self.aux_data[channel].spectrum_mag[12] = utils::db_to_gain(db3);
            self.aux_data[channel].spectrum_mag[50] = utils::db_to_gain(db4);
            
            self.aux_data[channel].spectrum_db[10] = db1;
            self.aux_data[channel].spectrum_db[11] = db2;
            self.aux_data[channel].spectrum_db[12] = db3;
            self.aux_data[channel].spectrum_db[50] = db4;
        }
    }

    fn process_spectrum(&mut self) {
        self.fft_effect.process_spectrum(
            [&self.data[0].spectrum_mag, &self.data[1].spectrum_mag], 
            [&self.data[0].spectrum_phase, &self.data[1].spectrum_phase],
            [&self.data[0].spectrum_db, &self.data[1].spectrum_db],
            [&self.data[0].spectrum_freq, &self.data[1].spectrum_freq],
            [&self.aux_data[0].spectrum_db, &self.aux_data[1].spectrum_db],
            [&self.aux_data[0].spectrum_mag, &self.aux_data[1].spectrum_mag],
            &mut self.ifft_in
        );

        // for channel in 0..2 {
        //     for (i, mag) in self.data[channel].spectrum_mag.iter().enumerate() {
        //         self.ifft_in[channel][i] = *mag;
        //     }
        // }
    }

    fn calculate_analyzer_db(&mut self) {
        for channel in 0..2 {
            for i in 1..(utils::fft_size_to_bins(self.fft_size) - 1) {
                self.data[channel].spectrum_mag[i] = self.ifft_in[channel][i].norm();
                self.data[channel].spectrum_db[i] = util::gain_to_db(self.data[channel].spectrum_mag[i]);
                //self.data[channel].spectrum_mag[i] = self.aux_data[channel].spectrum_mag[i];
                //self.data[channel].spectrum_db[i] = util::gain_to_db(self.aux_data[channel].spectrum_mag[i]);
            }
        }
    }

    fn handle_analyzer(&mut self) {

        let analyzer_input = self.analyzer_input_data.input_buffer();
        analyzer_input.magnitudes.fill(0.0f32);
        analyzer_input.reduction.fill(0.0f32);
        analyzer_input.num_bins = utils::fft_size_to_bins(self.fft_size);
        analyzer_input.p = self.peakiness;

        for (i, eq) in self.eq.iter().enumerate() {
            analyzer_input.eq[i] = *eq;
        }

        for (i, mag) in analyzer_input.magnitudes[0..utils::fft_size_to_bins(self.fft_size)].iter_mut().enumerate() {
            *mag = (self.data[0].spectrum_db[i] + self.data[1].spectrum_db[i]) / 2f32;
        }

        for (i, f) in analyzer_input.frequencies[0..utils::fft_size_to_bins(self.fft_size)].iter_mut().enumerate() {
            *f = self.data[0].spectrum_freq[i];
        }
        
        analyzer_input.lowcut = self.fft_effect.lowcut;
        analyzer_input.highcut = self.fft_effect.highcut;

        for (i, reduction) in analyzer_input.reduction[0..utils::fft_size_to_bins(self.fft_size)].iter_mut().enumerate() {
            *reduction = self.fft_effect.reduction[i];
        }
        self.analyzer_input_data.publish();
    }
}