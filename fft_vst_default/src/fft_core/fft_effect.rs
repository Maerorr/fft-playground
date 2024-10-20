use realfft::num_complex::Complex32;

pub trait FFTEffect: Send {
    fn process(&mut self, mag: [&Vec<f32>; 2], phase: [&Vec<f32>; 2], db: [&Vec<f32>; 2], freq: [&Vec<f32>; 2], output_buffer: [&Vec<Complex32>; 2]);
}