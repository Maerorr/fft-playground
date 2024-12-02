use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;

use crate::{analyzer_data::AnalyzerChannel, editor, fft_core::fft_size::FFTSize, utils};

#[derive(Params)]
pub struct PluginParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[id = "fft-size"]
    pub fft_size: EnumParam<FFTSize>,

    #[id = "analyzer-channel"]
    pub analyzer_channel: EnumParam<AnalyzerChannel>,
}

impl PluginParams {
    pub fn new(size_callback: Arc<AtomicBool>) -> Self {
        Self {
            editor_state: editor::default_state(),
            fft_size: EnumParam::new("FFT Size", FFTSize::_1024).with_callback(Arc::new(
                move |_| size_callback.store(true, Ordering::Release),
            )),
            analyzer_channel: EnumParam::new("Analyzer Channel", AnalyzerChannel::Merged),
        }
    }
}