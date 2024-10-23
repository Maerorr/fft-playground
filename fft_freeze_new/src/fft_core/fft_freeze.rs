use nih_plug::nih_log;
use rand::{rngs::{StdRng, ThreadRng}, Rng, SeedableRng};
use realfft::{num_complex::{Complex, Complex32}, num_traits::Zero};

use crate::utils::fft_size_to_bins;

#[derive(Clone)]
pub struct SpectralFrame {
    pub magnitudes: Vec<f32>,
    pub phases: Vec<f32>,
    pub size: usize,
}

impl SpectralFrame {
    pub fn new(bins_size: usize) -> Self {
        let mut mags = vec![0.0f32; 4096];
        let mut phases = vec![0.0f32; 4096];
        Self {
            magnitudes: mags,
            phases: phases,
            size: bins_size,
        }
    }
}

pub struct FFTFreeze {
    pub frames: [Vec<SpectralFrame>; 2],
    idx: usize,
    pub rand: StdRng,
    current_rand_read_idx: usize,
    smooth_size: usize,
    stereo_link: bool,

    frozen: bool,
}

impl FFTFreeze {
    pub fn new(bins_size: usize, frames_num: usize) -> Self {
        Self {
            frames: [vec![SpectralFrame::new(bins_size); frames_num], vec![SpectralFrame::new(bins_size); frames_num]],
            idx: 0,
            rand: StdRng::from_entropy(),
            current_rand_read_idx: 0,
            smooth_size: 4,
            stereo_link: true,
            frozen: false,
        }
    }

    pub fn resize(&mut self, size: usize) {
        // for channel in 0..2 {
        //     for frame in self.frames[channel].iter_mut() {
        //         frame.magnitudes.resize(fft_size_to_bins(size), 0.0f32);
        //         frame.phases.resize(fft_size_to_bins(size), 0.0f32);
        //     }
        // }
        for channel in 0..2 {
            for frame in self.frames[channel].iter_mut() {
                frame.size = fft_size_to_bins(size);
            }
        }
    }

    pub fn set_params(&mut self, frozen: bool, stereo_link: bool) {
        self.frozen = frozen;
        self.stereo_link = stereo_link;
    }

    /// Stores a stereo spectral frame to a ring buffer of N previous frames
    pub fn record(&mut self, mags: [&Vec<f32>; 2], phases: [&Vec<f32>; 2]) {
        for channel in 0..2 {
            for (i, (mag, phase)) in mags[channel].iter().zip(phases[channel].iter()).enumerate() {
                self.frames[channel][self.idx].magnitudes[i] = *mag;
                self.frames[channel][self.idx].phases[i] = *phase;
            }
            self.idx += 1;
            if self.idx == self.frames[channel].len() {
                self.idx = 0;
            }
        }
    }

    pub fn get_random_walk_next_frame_idx(&mut self) -> usize {
        let mut out_idx = 0usize;
        if self.current_rand_read_idx == 0 {
            self.current_rand_read_idx += 1;
        }
        if self.current_rand_read_idx >= (self.frames[0].len() - self.smooth_size - 4) {
            self.current_rand_read_idx -= 1;
        } else {
            let step = self.rand.gen_range(-1..=1);
            if step < 0 {
                self.current_rand_read_idx -= 1;
            }
            if step > 0 {
                self.current_rand_read_idx += 1;
            }
        }

        out_idx = self.current_rand_read_idx + self.idx;
        if out_idx >= self.frames[0].len() {
            out_idx -= self.frames[0].len();
        }

        out_idx
    }

    pub fn wrap_index(&self, idx: isize) -> usize {
        (((idx % self.frames[0].len() as isize) + self.frames[0].len() as isize) % self.frames[0].len() as isize) as usize
    }

    pub fn process_spectrum(&mut self, 
        mag: [&Vec<f32>; 2], 
        phase: [&Vec<f32>; 2], 
        db: [&Vec<f32>; 2], 
        freq: [&Vec<f32>; 2], 
        output_buffer: &mut [Vec<Complex<f32>>; 2]) 
    {
        // if we're not frozen, record the frame and move on
        if !self.frozen {
            self.record(mag, phase);
            
            // do nothing, simply forward the data to output buffer
            for channel in 0..2 {
                for (i, bin) in output_buffer[channel].iter_mut().enumerate() {
                    *bin = Complex::from_polar(mag[channel][i], phase[channel][i])
                }
            }

            return;
        }

        let mut idxs: [usize; 2] = [0, 0];
        idxs[0] = self.get_random_walk_next_frame_idx();
        // if stereo-link is on, that means both left and right channels use the same random idx to preserve timing of frames
        // if it's off, then both channels get separate indices for stereo randomness
        idxs[1] = if self.stereo_link {
            idxs[0]
        } else {
            self.get_random_walk_next_frame_idx()
        };

        for (channel, idx) in (0..2).into_iter().zip(idxs) {
            
            for (i, out_complex) in output_buffer[channel].iter_mut().enumerate() {
                let mut mag = 0.0f32;
                let mut phase = 0.0f32;
                let mut sum = 0.0f32;

                for o in 0..self.smooth_size {
                    let a = (self.smooth_size as f32 - 1f32) / 2f32;
                    let x = o as f32;
                    let weight = (-((x - a) / a).abs()) + 1.3f32;
                    //nih_log!("idx {} + o {} = {}", idx, o, idx+o);
                    mag += self.frames[channel][self.wrap_index((idx + o) as isize)].magnitudes[i] * weight;
                    phase += self.frames[channel][self.wrap_index((idx + o) as isize)].phases[i] * weight;
                    sum += weight
                }
                *out_complex = Complex::from_polar(mag / sum, phase / sum)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let len = 10;
        let idx = 5;
        let read_idx = 5;   
        let mut new_idx = read_idx + idx;
        if new_idx >= len {
            new_idx -= len;
        }
        assert_eq!(new_idx, 0);
    }
}
