use fft_processor::FFTProcessor;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::{env, sync::Arc};

mod editor;
mod fft_processor;
mod utils;
mod gate;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

const FFT_SIZE: usize = 1024;
const FFT_SIZE_F32: f32 = FFT_SIZE as f32;
const NUM_BINS: usize = FFT_SIZE / 2 + 1;
const OVERLAP: usize = 4;
const HOP_SIZE: usize = FFT_SIZE / OVERLAP;
const WINDOW_CORRECTION: f32 = 2.0 / 3.0;

pub struct FFTGate {
    fft_processors: [FFTProcessor; 2],
    params: Arc<FFTGateParams>,
}

#[derive(Params)]
struct FFTGateParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
}

impl Default for FFTGate {
    fn default() -> Self {
        let fft_proc1 = FFTProcessor::new(44100u32);
        let fft_proc2 = FFTProcessor::new(44100u32);
        Self {
            fft_processors: [fft_proc1, fft_proc2],
            params: Arc::new(FFTGateParams::default()),
        }
    }
}

impl Default for FFTGateParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
        }
    }
}

impl Plugin for FFTGate{
    const NAME: &'static str = "fftgate";
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
        env::set_var("NIH_LOG", "C:\\Users\\7hube\\Desktop\\nih_log.txt");
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            for (i, sample) in channel_samples.into_iter().enumerate() {
                *sample = self.fft_processors[i].process_sample(*sample);
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.editor_state.clone(),
        )
    }
}

impl ClapPlugin for FFTGate {
    const CLAP_ID: &'static str = "fftgate";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("fftgate");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for FFTGate {
    const VST3_CLASS_ID: [u8; 16] = *b"fft_gate________";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

//nih_export_clap!(FFTGate);
nih_export_vst3!(FFTGate);
