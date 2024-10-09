use crate::NUM_BINS;

#[derive(Debug, Clone)]
pub struct AnalyzerData {
    pub num_bins: usize,
    pub magnitudes: [f32; NUM_BINS],
}

impl Default for AnalyzerData {
    fn default() -> Self {
        Self { num_bins: 0, magnitudes: [0.0; NUM_BINS] }
    }
}