use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

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

    #[id = "low-mid-frequency"]
    pub low_mid_frequency: FloatParam,

    #[id = "mid-high-frequency"]
    pub mid_high_frequency: FloatParam,

    // LOW
    #[id = "low-threshold"]
    pub low_threshold: FloatParam,

    #[id = "low-gain"]
    pub low_gain: FloatParam,

    //MID
    #[id = "mid-threshold"]
    pub mid_threshold: FloatParam,

    #[id = "mid-gain"]
    pub mid_gain: FloatParam,

    //HIGH
    #[id = "high-threshold"]
    pub high_threshold: FloatParam,

    #[id = "high-gain"]
    pub high_gain: FloatParam,
}

impl PluginParams {
    pub fn new(size_callback: Arc<AtomicBool>) -> Self {
        Self {
            editor_state: editor::default_state(),
            fft_size: EnumParam::new("FFT Size", FFTSize::_1024).with_callback(Arc::new(
                move |_| size_callback.store(true, Ordering::Release),
            )),
            analyzer_channel: EnumParam::new("Analyzer Channel", AnalyzerChannel::Merged),
            
            low_mid_frequency: FloatParam::new("Low/Mid Frequency", 300.0f32, FloatRange::Skewed { min: 100.0f32, max: 8_000f32, factor: 0.6 })
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(1)),
            mid_high_frequency: FloatParam::new("Mid/High Frequency", 300.0f32, FloatRange::Skewed { min: 300.0f32, max: 10_000f32, factor: 0.6 })
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(1)),
            
            low_threshold: FloatParam::new(
                "Low Threshold",
                utils::db_to_gain(0f32),
                FloatRange::Linear {
                    min: utils::db_to_gain(-80f32),
                    max: utils::db_to_gain(20f32),
                },
            )
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            low_gain: FloatParam::new(
                "Low Gain",
                utils::db_to_gain(0f32),
                FloatRange::Linear {
                    min: utils::db_to_gain(-20f32),
                    max: utils::db_to_gain(40f32),
                },
            )
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            mid_threshold: FloatParam::new(
                "Mid Threshold",
                utils::db_to_gain(0f32),
                FloatRange::Linear {
                    min: utils::db_to_gain(-80f32),
                    max: utils::db_to_gain(20f32),
                },
            )
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            mid_gain: FloatParam::new(
                "Mid Gain",
                utils::db_to_gain(0f32),
                FloatRange::Linear {
                    min: utils::db_to_gain(-20f32),
                    max: utils::db_to_gain(40f32),
                },
            )
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            high_threshold: FloatParam::new(
                "High Threshold",
                utils::db_to_gain(0f32),
                FloatRange::Linear {
                    min: utils::db_to_gain(-80f32),
                    max: utils::db_to_gain(20f32),
                },
            )
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            high_gain: FloatParam::new(
                "High Gain",
                utils::db_to_gain(0f32),
                FloatRange::Linear {
                    min: utils::db_to_gain(-20f32),
                    max: utils::db_to_gain(40f32),
                },
            )
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}
