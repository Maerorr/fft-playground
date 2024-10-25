use std::sync::{Arc, Mutex};

use analyzer::Analyzer;
use nih_plug::prelude::{util, AtomicF32, Editor, Vst3Plugin};
use nih_plug_vizia::vizia::image::{Pixel, Pixels};
use nih_plug_vizia::vizia::{prelude::*, vg};
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};

use crate::analyzer_data::AnalyzerData;
use crate::PluginParams;

mod param_knob;
mod analyzer;

pub const COMFORTAA_LIGHT_TTF: &[u8] = include_bytes!("../res/Comfortaa-Light.ttf");
pub const COMFORTAA: &str = "Comfortaa";

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (500, 350))
}

#[derive(Clone, Lens)]
pub struct EditorData {
    pub plugin_data: Arc<PluginParams>,
    pub analyzer_data: Arc<Mutex<triple_buffer::Output<AnalyzerData>>>,
    pub sample_rate: Arc<AtomicF32>,
}

impl Model for EditorData {}

pub(crate) fn create(
    editor_state: Arc<ViziaState>,
    editor_data: EditorData,
    ) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        cx.add_font_mem(&COMFORTAA_LIGHT_TTF);
        cx.set_default_font(&[COMFORTAA]);

        editor_data.clone().build(cx);

        ResizeHandle::new(cx);
        VStack::new(cx, |cx| {
            Label::new(cx, "PEACEMAKER")
            .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
            .font_size(24.0)
            .height(Pixels(75.0))
            .child_top(Stretch(1.0))
            .child_bottom(Stretch(1.0))
            .class("header-label");

            HStack::new(cx, |cx| {
                //params go here \/
                Label::new(cx, "Sidechain Gain")
                    .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                    .font_size(16.0)
                    .left(Stretch(1.0))
                    .right(Pixels(7.0))
                    .top(Pixels(5.0));

                ParamSlider::new(cx, EditorData::plugin_data, |params| &params.sidechain_gain);

            }).child_left(Stretch(1.0))
            .child_right(Stretch(1.0))
            .max_height(Pixels(50.0));

            Analyzer::new(cx, EditorData::analyzer_data, EditorData::sample_rate)
            //.max_width(Pixels(450.0))
            .max_height(Pixels(200.0))
            .border_width(Pixels(2.0))
            .border_color(Color::black())
            .left(Pixels(1.0)).right(Pixels(1.0));

            // BOTTOM BAR FOR MISC INFO IN ALL PLUGINS (FFT SIZE AND ANALYZER CHANNEL)
            HStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Label::new(cx, "FFT Size:")
                    .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                    .font_size(13.0)
                    .left(Stretch(1.0))
                    .right(Pixels(7.0))
                    .top(Pixels(5.0));

                    ParamSlider::new(cx, EditorData::plugin_data, |params| &params.fft_size)
                    .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                    .font_size(13.0)
                    .top(Pixels(5.0))
                    .max_width(Pixels(100.0))
                    .max_height(Pixels(20.0));
                })
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0))
                .width(Pixels(230.0));
                HStack::new(cx, |cx| {
                    Label::new(cx, "Analyzer Channel:")
                    .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                    .font_size(13.0)
                    .left(Stretch(1.0))
                    .right(Pixels(7.0))
                    .top(Pixels(5.0));

                    ParamSlider::new(cx, EditorData::plugin_data, |params| &params.analyzer_channel).font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                    .font_size(13.0)
                    .top(Pixels(5.0))
                    .max_width(Pixels(100.0))
                    .max_height(Pixels(20.0));
                })
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0))
                .width(Pixels(230.0));
            }).child_left(Stretch(1.0))
            .child_right(Stretch(1.0))
            .max_height(Pixels(50.0));
            
        }).row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0))
        .class("main-gui");
    })
}