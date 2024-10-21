use analyzer_data::{AnalyzerChannel, AnalyzerData};
use fft_core::{fft_size::FFTSize, stereo_fft_processor::StereoFFTProcessor};
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use triple_buffer::TripleBuffer;
use util::db_to_gain;
use std::{env, f32::consts::PI, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}};

mod editor;
mod fft_core;
mod utils;
mod analyzer_data;

// const FFT_SIZE: usize = 1024;
// const FFT_SIZE_F32: f32 = FFT_SIZE as f32;
// const NUM_BINS: usize = FFT_SIZE / 2 + 1;
// const OVERLAP: usize = 4;
// const HOP_SIZE: usize = FFT_SIZE / OVERLAP;
const WINDOW_CORRECTION: f32 = 2.0 / 3.0;

pub struct PluginData {
    stereo_fft_processor: StereoFFTProcessor,
    params: Arc<PluginParams>,
    analyzer_output_data: Arc<Mutex<triple_buffer::Output<AnalyzerData>>>,
    sample_rate: Arc<AtomicF32>,
    size_changed: Arc<AtomicBool>,
}

#[derive(Params)]
pub struct PluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "fft-size"]
    fft_size: EnumParam<FFTSize>,

    #[id = "analyzer-channel"]
    analyzer_channel: EnumParam<AnalyzerChannel>,
}

impl Default for PluginData {
    fn default() -> Self {
        let (analyzer_input_data, analyzer_output_data) = TripleBuffer::new(&AnalyzerData::new(utils::fft_size_to_bins(FFTSize::_4096 as usize), 44100)).split();
        let size_changed = Arc::new(AtomicBool::new(false));
        
        Self {
            stereo_fft_processor: StereoFFTProcessor::new(44100, FFTSize::_1024 as usize,size_changed.clone(), analyzer_input_data),
            params: Arc::new(PluginParams::new(size_changed.clone())),
            analyzer_output_data: Arc::new(Mutex::new(analyzer_output_data)),
            sample_rate: Arc::new(AtomicF32::new(1.0)),
            size_changed: size_changed.clone(),
        }
    }
}

impl PluginParams {
    fn new(size_callback: Arc<AtomicBool>) -> Self {
        Self {
            editor_state: editor::default_state(),
            fft_size: EnumParam::new("FFT Size", FFTSize::_1024)
            .with_callback(Arc::new(move |_| {
                    size_callback.store(true, Ordering::Release)
                })),
            analyzer_channel: EnumParam::new("Analyzer Channel", AnalyzerChannel::Merged),
        }
    }
}

impl Plugin for PluginData {
    const NAME: &'static str = "default_spectral_template";
    const VENDOR: &'static str = "";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
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
        self.stereo_fft_processor.set_sample_rate(_buffer_config.sample_rate as usize);
        self.sample_rate.store(_buffer_config.sample_rate, std::sync::atomic::Ordering::Relaxed);
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

        if self.size_changed.load(Ordering::Relaxed) {
            self.stereo_fft_processor.change_fft_size(fft_size as usize);
            self.size_changed.store(false, Ordering::Relaxed);
        }

        self.stereo_fft_processor.set_params(an_chan);

        for mut channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let output_samples = self.stereo_fft_processor.process_sample(
                [*channel_samples.get_mut(0).unwrap(), 
                *channel_samples.get_mut(1).unwrap()]
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
            }
        )
    }
}

impl ClapPlugin for PluginData {
    const CLAP_ID: &'static str = "default_spectral_template";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("default_spectral_template");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for PluginData {
    const VST3_CLASS_ID: [u8; 16] = *b"deflt_spec_templ";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

//nih_export_clap!(FFTGate);
nih_export_vst3!(PluginData);
