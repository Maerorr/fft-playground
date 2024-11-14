use realfft::{num_complex::Complex, ComplexToReal, RealFftPlanner, RealToComplex};

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
        let fft_in = r2c.make_input_vec();
        let fft_out = r2c.make_output_vec();
        let ifft_out = c2r.make_output_vec();
        let num_bins = fft_out.len();

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
        self.fft_in = self.r2c.make_input_vec();
        self.fft_out = self.r2c.make_output_vec();
        self.ifft_out = self.c2r.make_output_vec();
        
        let num_bins = self.fft_out.len();
        self.spectrum_mag.resize(num_bins, 0.0f32);
        self.spectrum_phase.resize(num_bins, 0.0f32);
        self.spectrum_db.resize(num_bins, 0.0f32);
        self.spectrum_freq.resize(num_bins, 0.0f32);
    }
}