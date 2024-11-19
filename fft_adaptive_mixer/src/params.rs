use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;

use crate::{analyzer_data::AnalyzerChannel, editor, fft_core::fft_size::FFTSize, utils};

#[derive(Params)]
pub struct PluginParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[id = "lowcut"]
    pub lowcut: FloatParam,

    #[id = "highcut"]
    pub highcut: FloatParam,

    #[id = "amount"]
    pub amount: FloatParam,

    #[id = "gate"]
    pub gate: FloatParam,

    #[id = "smooth"]
    pub smooth: FloatParam,

    #[id = "peakiness"]
    pub peakiness: FloatParam,

    #[id = "eq1"]
    pub eq1: FloatParam,

    #[id = "eq2"]
    pub eq2: FloatParam,

    #[id = "eq3"]
    pub eq3: FloatParam,

    #[id = "eq4"]
    pub eq4: FloatParam,

    #[id = "eq5"]
    pub eq5: FloatParam,

    #[id = "eq6"]
    pub eq6: FloatParam,

    #[id = "eq7"]
    pub eq7: FloatParam,

    #[id = "eq8"]
    pub eq8: FloatParam,

    #[id = "stereo-link"]
    pub stereo_link: BoolParam,

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

            lowcut: FloatParam::new(
                "LowCut",
                50.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: 0.3,
                },
            )
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            highcut: FloatParam::new(
                "HighCut",
                20_000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: 0.3,
                },
            )
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            stereo_link: BoolParam::new("Stereo Link", false),

            eq1: FloatParam::new(
                    "Eq1",
                    utils::db_to_gain(0.0),
                    FloatRange::SymmetricalSkewed {
                        min: utils::db_to_gain(-36.0),
                        max: utils::db_to_gain(36.0),
                        factor: 0.3,
                        center: utils::db_to_gain(0.0),
                    },
                )
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
                .with_unit("dB"),
            eq2: FloatParam::new(
                    "Eq2",
                    utils::db_to_gain(0.0),
                    FloatRange::SymmetricalSkewed {
                        min: utils::db_to_gain(-36.0),
                        max: utils::db_to_gain(36.0),
                        factor: 0.3,
                        center: utils::db_to_gain(0.0),
                    },
                )
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
                .with_unit("dB"),
            eq3: FloatParam::new(
                    "Eq3",
                    utils::db_to_gain(0.0),
                    FloatRange::SymmetricalSkewed {
                        min: utils::db_to_gain(-36.0),
                        max: utils::db_to_gain(36.0),
                        factor: 0.3,
                        center: utils::db_to_gain(0.0),
                    },
                )
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
                .with_unit("dB"),
            eq4: FloatParam::new(
                    "Eq4",
                    utils::db_to_gain(0.0),
                    FloatRange::SymmetricalSkewed {
                        min: utils::db_to_gain(-36.0),
                        max: utils::db_to_gain(36.0),
                        factor: 0.3,
                        center: utils::db_to_gain(0.0),
                    },
                )
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
                .with_unit("dB"),
            eq5: FloatParam::new(
                    "Eq5",
                    utils::db_to_gain(0.0),
                    FloatRange::SymmetricalSkewed {
                        min: utils::db_to_gain(-36.0),
                        max: utils::db_to_gain(36.0),
                        factor: 0.3,
                        center: utils::db_to_gain(0.0),
                    },
                )
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
                .with_unit("dB"),
            eq6: FloatParam::new(
                    "Eq6",
                    utils::db_to_gain(0.0),
                    FloatRange::SymmetricalSkewed {
                        min: utils::db_to_gain(-36.0),
                        max: utils::db_to_gain(36.0),
                        factor: 0.3,
                        center: utils::db_to_gain(0.0),
                    },
                )
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
                .with_unit("dB"),
            eq7: FloatParam::new(
                    "Eq7",
                    utils::db_to_gain(0.0),
                    FloatRange::SymmetricalSkewed {
                        min: utils::db_to_gain(-36.0),
                        max: utils::db_to_gain(36.0),
                        factor: 0.3,
                        center: utils::db_to_gain(0.0),
                    },
                )
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
                .with_unit("dB"),
            eq8: FloatParam::new(
                    "Eq8",
                    utils::db_to_gain(0.0),
                    FloatRange::SymmetricalSkewed {
                        min: utils::db_to_gain(-36.0),
                        max: utils::db_to_gain(36.0),
                        factor: 0.3,
                        center: utils::db_to_gain(0.0),
                    },
                )
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
                .with_unit("dB"),
            amount: FloatParam::new(
                    "Amount",
                    utils::db_to_gain(-40.0),
                    FloatRange::Skewed {
                        min: utils::db_to_gain(-80.0),
                        max: utils::db_to_gain(0.0),
                        factor: 0.6,
                    },
                )
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
                .with_unit("dB"),
            gate: FloatParam::new(
                    "Gate",
                    utils::db_to_gain(-40.0),
                    FloatRange::Skewed { 
                        min: utils::db_to_gain(-80.0), 
                        max: utils::db_to_gain(0.0), 
                        factor: 0.3 
                    }
                ).with_value_to_string(formatters::v2s_f32_gain_to_db(2))
                .with_string_to_value(formatters::s2v_f32_gain_to_db())
                .with_unit("dB"),
            smooth: FloatParam::new(
                "Smooth",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ),
            peakiness: FloatParam::new(
                "Peakiness",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            )
        }
    }
}