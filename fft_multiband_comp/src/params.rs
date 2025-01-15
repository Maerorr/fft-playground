use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use formatters::{s2v_compression_ratio, s2v_f32_percentage, v2s_compression_ratio, v2s_f32_percentage};
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

    #[id = "low-ratio"]
    pub low_ratio: FloatParam,

    #[id = "low-up-ratio"]
    pub low_up_ratio: FloatParam,

    #[id = "low-gain"]
    pub low_gain: FloatParam,

    //MID
    #[id = "mid-threshold"]
    pub mid_threshold: FloatParam,

    #[id = "mid-ratio"]
    pub mid_ratio: FloatParam,

    #[id = "mid-up-ratio"]
    pub mid_up_ratio: FloatParam,

    #[id = "mid-gain"]
    pub mid_gain: FloatParam,

    //HIGH
    #[id = "high-threshold"]
    pub high_threshold: FloatParam,

    #[id = "high-ratio"]
    pub high_ratio: FloatParam,

    #[id = "high-up-ratio"]
    pub high_up_ratio: FloatParam,

    #[id = "high-gain"]
    pub high_gain: FloatParam,

    #[id = "attack-ms"]
    pub attack_ms: FloatParam,

    #[id = "release-ms"]
    pub release_ms: FloatParam,

    #[id = "in-gain"]
    pub in_gain: FloatParam,

    #[id = "out-gain"]
    pub out_gain: FloatParam,

    #[id = "mix"]
    pub mix: FloatParam,

    #[id = "smooth"]
    pub smooth: FloatParam,
}

impl PluginParams {
    pub fn new(size_callback: Arc<AtomicBool>) -> Self {
        Self {
            editor_state: editor::default_state(),
            fft_size: EnumParam::new("FFT Size", FFTSize::_1024).with_callback(Arc::new(
                move |_| size_callback.store(true, Ordering::Release),
            )),
            analyzer_channel: EnumParam::new("Analyzer Channel", AnalyzerChannel::Merged),
            
            low_mid_frequency: FloatParam::new(
                "Low/Mid Frequency", 
                300.0f32, 
                FloatRange::Skewed 
                { 
                    min: 100.0f32, 
                    max: 8_000f32, 
                    factor: 0.6 
                })
                .with_value_to_string(formatters::v2s_f32_hz_then_khz(1)),
            mid_high_frequency: FloatParam::new(
                "Mid/High Frequency", 
                3500.0f32, 
                FloatRange::Skewed 
                { 
                    min: 300.0f32, 
                    max: 10_000f32, 
                    factor: 0.6 
                })
                .with_value_to_string(formatters::v2s_f32_hz_then_khz(1)),
            
            low_threshold: FloatParam::new(
                "Low Threshold",
                utils::db_to_gain(-10f32),
                FloatRange::Skewed {
                    min: utils::db_to_gain(-80f32),
                    max: utils::db_to_gain(0f32),
                    factor: 0.3,
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
                utils::db_to_gain(-20f32),
                FloatRange::Skewed {
                    min: utils::db_to_gain(-80f32),
                    max: utils::db_to_gain(0f32),
                    factor: 0.3,
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
                utils::db_to_gain(-30f32),
                FloatRange::Skewed {
                    min: utils::db_to_gain(-80f32),
                    max: utils::db_to_gain(0f32),
                    factor: 0.3,
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
            attack_ms: FloatParam::new(
                "Attack",
                10.0f32,
                FloatRange::Linear {
                    min: 0.1f32,
                    max: 100.0f32,
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(1))
            .with_unit("ms"),
            release_ms: FloatParam::new(
                "Release",
                50.0f32,
                FloatRange::Linear {
                    min: 1.0f32,
                    max: 200.0f32,
                },
            ).with_value_to_string(formatters::v2s_f32_rounded(1))
            .with_unit("ms"),
            low_ratio: FloatParam::new("Low Ratio", 2.0, FloatRange::Linear { min: 1.0, max: 20.0 })
            .with_string_to_value(s2v_compression_ratio())
            .with_value_to_string(v2s_compression_ratio(2)),
            mid_ratio: FloatParam::new("Mid Ratio", 2.0,  FloatRange::Linear { min: 1.0, max: 20.0 })
            .with_string_to_value(s2v_compression_ratio())
            .with_value_to_string(v2s_compression_ratio(2)),
            high_ratio: FloatParam::new("High Ratio", 2.0, FloatRange::Linear { min: 1.0, max: 20.0 })
            .with_string_to_value(s2v_compression_ratio())
            .with_value_to_string(v2s_compression_ratio(2)),
            in_gain: FloatParam::new(
                "In Gain",
                utils::db_to_gain(0f32),
                FloatRange::Skewed {
                    min: utils::db_to_gain(-40f32),
                    max: utils::db_to_gain(20f32),
                    factor: 0.4
                },
            )
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            out_gain: FloatParam::new(
                "Out Gain",
                utils::db_to_gain(0f32),
                FloatRange::Skewed {
                    min: utils::db_to_gain(-40f32),
                    max: utils::db_to_gain(20f32),
                    factor: 0.4
                },
            )
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            mix: FloatParam::new("Mix", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_string_to_value(s2v_f32_percentage())
            .with_value_to_string(v2s_f32_percentage(2))
            .with_unit("%"),
            low_up_ratio: FloatParam::new("Low Up Ratio", 1.0,  FloatRange::Linear { min: 1.0, max: 5.0 })
            .with_string_to_value(s2v_compression_ratio())
            .with_value_to_string(v2s_compression_ratio(2)),
            mid_up_ratio: FloatParam::new("Mid Up Ratio", 1.0,  FloatRange::Linear { min: 1.0, max: 5.0 })
            .with_string_to_value(s2v_compression_ratio())
            .with_value_to_string(v2s_compression_ratio(2)),
            high_up_ratio: FloatParam::new("High Up Ratio", 1.0,  FloatRange::Linear { min: 1.0, max: 5.0 })
            .with_string_to_value(s2v_compression_ratio())
            .with_value_to_string(v2s_compression_ratio(2)),

            smooth: FloatParam::new(
                "Smooth",
                0.8,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ),
        }
    }
}
