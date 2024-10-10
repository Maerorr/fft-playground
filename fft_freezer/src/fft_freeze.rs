use nih_plug::nih_log;
use rand::{rngs::{StdRng, ThreadRng}, Rng, SeedableRng};

use crate::NUM_BINS;

#[derive(Clone)]
pub struct SpectralFrame {
    pub magnitudes: Vec<f32>,
    pub phases: Vec<f32>,
    
}

impl SpectralFrame {
    pub fn new() -> Self {
        Self {
            magnitudes: vec![0.0f32; NUM_BINS],
            phases: vec![0.0f32; NUM_BINS],
        }
    }
}

pub struct FFTFreeze {
    pub frames: Vec<SpectralFrame>,
    idx: usize,
    pub rand: StdRng,
    current_rand_read_idx: usize,
}

impl FFTFreeze {
    pub fn new(frames_num: usize) -> Self {
        Self {
            frames: vec![SpectralFrame::new(); frames_num],
            idx: 0,
            rand: StdRng::from_entropy(),
            current_rand_read_idx: 0,
        }
    }

    pub fn record(&mut self, mags: &Vec<f32>, phases: &Vec<f32>) {
        //nih_log!("recording frame num {}", self.idx);
        for (i, (mag, phase)) in mags.iter().zip(phases.iter()).enumerate() {
            self.frames[self.idx].magnitudes[i] = *mag;
            self.frames[self.idx].phases[i] = *phase;
        }
        self.idx += 1;
        if (self.idx == self.frames.len()) {
            self.idx = 0;
        }
    }

    pub fn get_rand_frame_num(&mut self) -> usize {
        //rand::thread_rng().gen::<usize>() % self.frames.len()
        let mut out_idx = 0usize;
        if self.current_rand_read_idx == 0 {
            self.current_rand_read_idx += 1;
        }

        if self.current_rand_read_idx == self.frames.len() - 1 {
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
        if out_idx >= self.frames.len() {
            out_idx -= self.frames.len();
        }

        out_idx
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
