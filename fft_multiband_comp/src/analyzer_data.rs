use nih_plug::prelude::Enum;

#[derive(Enum, Debug, Clone, Copy, PartialEq)]
pub enum AnalyzerChannel {
    Left,
    Right,
    Merged
}

#[derive(Debug, Clone)]
pub struct AnalyzerData {
    pub num_bins: usize,
    pub magnitudes: Vec<f32>,
    pub frequencies: Vec<f32>,
    pub bands_freqs: [f32; 2],
    pub delta: Vec<f32>,
    pub comp_curve_low: Vec<f32>,
    pub comp_curve_mid: Vec<f32>,
    pub comp_curve_high: Vec<f32>,
}

impl AnalyzerData {
    pub fn new(num_bins: usize) -> Self {
        Self {
            num_bins,
            magnitudes: vec![0.0f32; num_bins],
            frequencies: vec![0.0f32; num_bins],
            bands_freqs: [0.0f32; 2],
            delta: vec![0.0f32; num_bins],
            comp_curve_low: vec![-100.0f32; 50],
            comp_curve_mid: vec![-100.0f32; 50],
            comp_curve_high: vec![-100.0f32; 50],
        }
    }
}