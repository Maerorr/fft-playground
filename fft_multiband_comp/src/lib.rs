use analyzer_data::{AnalyzerChannel, AnalyzerData};
use fft_core::{compressor::Compressor, fft_size::FFTSize, stereo_fft_processor::StereoFFTProcessor};
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use params::PluginParams;
use std::{
    env,
    f32::consts::PI,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use triple_buffer::TripleBuffer;
use util::db_to_gain;

mod analyzer_data;
mod editor;
mod fft_core;
mod params;
mod utils;

// const FFT_SIZE: usize = 1024;
// const FFT_SIZE_F32: f32 = FFT_SIZE as f32;
// const NUM_BINS: usize = FFT_SIZE / 2 + 1;
// const OVERLAP: usize = 4;
// const HOP_SIZE: usize = FFT_SIZE / OVERLAP;
const WINDOW_CORRECTION: f32 = 2.0 / 3.0;

pub struct PluginData {
    stereo_fft_processor: StereoFFTProcessor,
    compressor: Compressor,
    params: Arc<PluginParams>,
    analyzer_output_data: Arc<Mutex<triple_buffer::Output<AnalyzerData>>>,
    sample_rate: Arc<AtomicF32>,
    size_changed: Arc<AtomicBool>,
}

impl Default for PluginData {
    fn default() -> Self {
        let (analyzer_input_data, analyzer_output_data) = TripleBuffer::new(&AnalyzerData::new(
            utils::fft_size_to_bins(FFTSize::_4096 as usize),
        ))
        .split();
        let size_changed = Arc::new(AtomicBool::new(false));

        let attack_coeff = (-1.0f32 / (10.0 * 44100.0 * 0.001)).exp();
        let release_coeff = (-1.0f32 / (50.0 * 44100.0 * 0.001)).exp();

        Self {
            stereo_fft_processor: StereoFFTProcessor::new(
                44100,
                FFTSize::_1024 as usize,
                size_changed.clone(),
                analyzer_input_data,
            ),
            compressor: Compressor::new(-30.0, 2.0, 5.0, attack_coeff, release_coeff),
            params: Arc::new(PluginParams::new(size_changed.clone())),
            analyzer_output_data: Arc::new(Mutex::new(analyzer_output_data)),
            sample_rate: Arc::new(AtomicF32::new(1.0)),
            size_changed: size_changed.clone(),
        }
    }
}

impl Plugin for PluginData {
    const NAME: &'static str = "fft_multiband_comp";
    const VENDOR: &'static str = "";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        //env::set_var("NIH_LOG", "C:\\Users\\7hube\\Desktop\\nih_log.txt");
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        let new_size = self.params.fft_size.value();
        _context.set_latency_samples(new_size as u32);
        self.stereo_fft_processor.change_fft_size(new_size as usize);
        self.stereo_fft_processor
            .set_sample_rate(_buffer_config.sample_rate as usize);
        self.sample_rate.store(
            _buffer_config.sample_rate,
            std::sync::atomic::Ordering::Relaxed,
        );

        let attack_coeff = (-1.0f32 / (10.0 * _buffer_config.sample_rate as f32 * 0.001)).exp();
        let release_coeff = (-1.0f32 / (50.0 * _buffer_config.sample_rate as f32 * 0.001)).exp();

        self.compressor.set_params(-30.0, 2.0, 5.0, attack_coeff, release_coeff);
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
        let new_size = self.params.fft_size.value();
        self.stereo_fft_processor.change_fft_size(new_size as usize);
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let an_chan = self.params.analyzer_channel.value();
        let fft_size = self.params.fft_size.value();

        let low_mid_freq = self.params.low_mid_frequency.value();
        let mid_high_freq = self.params.mid_high_frequency.value();

        let low_threshold =  utils::gain_to_db(self.params.low_threshold.value());
        let low_ratio = self.params.low_ratio.value();
        let low_up_ratio = self.params.low_up_ratio.value();
        let low_gain = self.params.low_gain.value();
        let mid_threshold =  utils::gain_to_db(self.params.mid_threshold.value());
        let mid_ratio = self.params.mid_ratio.value();
        let mid_up_ratio = self.params.mid_up_ratio.value();
        let mid_gain = self.params.mid_gain.value();
        let high_threshold = utils::gain_to_db(self.params.high_threshold.value());
        let high_ratio = self.params.high_ratio.value();
        let high_up_ratio = self.params.high_up_ratio.value();
        let high_gain = self.params.high_gain.value();
        let attack_ms = self.params.attack_ms.value();
        let release_ms = self.params.release_ms.value();
        let mix = self.params.mix.value();
        let in_gain = self.params.in_gain.value();
        let out_gain = self.params.out_gain.value();

        self.stereo_fft_processor.set_params(
            an_chan,
            low_threshold,
            low_ratio,
            low_up_ratio,
            low_gain,
            mid_threshold,
            mid_ratio,
            mid_up_ratio,
            mid_gain,
            high_threshold,
            high_ratio,
            high_up_ratio,
            high_gain,
            attack_ms,
            release_ms,
            mix,
            in_gain,
            out_gain,
        );

        if self.size_changed.load(Ordering::Relaxed) {
            _context.set_latency_samples(fft_size as u32);
            self.stereo_fft_processor.change_fft_size(fft_size as usize);
            self.size_changed.store(false, Ordering::Relaxed);
        }

        for mut channel_samples in buffer.iter_samples() {
            let output_samples = self.stereo_fft_processor.process_sample(
                [
                    *channel_samples.get_mut(0).unwrap(),
                    *channel_samples.get_mut(1).unwrap(),
                ],
            );

            *channel_samples.get_mut(0).unwrap() = output_samples[0];
            *channel_samples.get_mut(1).unwrap() = output_samples[1];
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.editor_state.clone(),
            editor::EditorData {
                plugin_data: self.params.clone(),
                analyzer_data: self.analyzer_output_data.clone(),
                sample_rate: self.sample_rate.clone(),
            },
        )
    }
}

impl ClapPlugin for PluginData {
    const CLAP_ID: &'static str = "fft_multiband_comp";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("fft multiband comp");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for PluginData {
    const VST3_CLASS_ID: [u8; 16] = *b"fftmultibandcomp";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_vst3!(PluginData);
