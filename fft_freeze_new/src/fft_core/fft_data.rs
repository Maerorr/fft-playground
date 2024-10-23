use realfft::{num_complex::Complex, num_traits::Zero, ComplexToReal, RealFftPlanner, RealToComplex};

use crate::utils::fft_size_to_bins;

pub struct FFTData {
    pub planner: RealFftPlanner<f32>,
    pub r2c: std::sync::Arc<dyn RealToComplex<f32>>,
    pub c2r: std::sync::Arc<dyn ComplexToReal<f32>>,

    pub fft_in: Vec<f32>,
    pub fft_out: Vec<Complex<f32>>,
    pub ifft_out: Vec<f32>, 

    pub spectrum_mag: Vec<f32>,
    pub spectrum_phase: Vec<f32>,
    pub spectrum_freq: Vec<f32>, 
    pub spectrum_db: Vec<f32>,
}

impl FFTData {
    pub fn new(fft_size: usize) -> Self {
        // all fft stuff
        let mut planner = RealFftPlanner::<f32>::new();
        let r2c = planner.plan_fft_forward(fft_size);
        let c2r = planner.plan_fft_inverse(fft_size);
        let mut fft_in = r2c.make_input_vec();
        let mut fft_out = r2c.make_output_vec();
        let mut ifft_out = c2r.make_output_vec();
        let num_bins = fft_out.len();

        fft_in.reserve(2048);
        fft_out.reserve(2048);
        ifft_out.reserve(2048);

        let mut spectrum_mag = vec![0f32; num_bins];
        let mut spectrum_phase = vec![0f32; num_bins];
        let mut spectrum_freq = vec![0f32; num_bins];
        let mut spectrum_db = vec![0f32; num_bins];
        spectrum_mag.reserve(2048);
        spectrum_db.reserve(2048);
        spectrum_phase.reserve(2048);
        spectrum_freq.reserve(2048);

        Self {
            planner, 
            r2c,
            c2r,
            fft_in,
            fft_out,
            ifft_out,
            spectrum_mag: vec![0f32; num_bins],
            spectrum_phase: vec![0f32; num_bins],
            spectrum_freq: vec![0f32; num_bins],
            spectrum_db: vec![0f32; num_bins],
        }
    }

    pub fn fft_size_change(&mut self, new_fft_size: usize) {
        self.r2c = self.planner.plan_fft_forward(new_fft_size);
        self.c2r = self.planner.plan_fft_inverse(new_fft_size);
        self.fft_in.resize(new_fft_size, 0.0f32);// = self.r2c.make_input_vec();
        self.fft_out.resize(fft_size_to_bins(new_fft_size), Complex::zero());// = self.r2c.make_output_vec();
        self.ifft_out.resize(new_fft_size, 0.0f32);//= self.c2r.make_output_vec();
        
        let num_bins: usize = self.fft_out.len();
        self.spectrum_mag.resize(num_bins, 0.0f32);
        self.spectrum_mag.fill(0f32);
        self.spectrum_phase.resize(num_bins, 0.0f32);
        self.spectrum_phase.fill(0f32);
        self.spectrum_db.resize(num_bins, 0.0f32);
        self.spectrum_db.fill(0f32);
        self.spectrum_freq.resize(num_bins, 0.0f32);
        self.spectrum_freq.fill(0f32);
    }
}