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
    pub reduction: Vec<f32>,
    pub lowcut: f32,
    pub highcut: f32,
    pub p: f32,
    pub eq: Vec<f32>,
}

impl AnalyzerData {
    pub fn new(num_bins: usize, sr: usize) -> Self {
        Self {
            num_bins,
            magnitudes: vec![0.0f32; num_bins],
            frequencies: vec![0.0f32; num_bins],
            reduction: vec![0.0f32; num_bins],
            lowcut: 50.0f32,
            highcut: 18_000.0f32,
            p: 1.0f32,
            eq: vec![0.0f32; 8],
        }
    }
}