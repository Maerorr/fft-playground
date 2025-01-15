use std::sync::{Arc, Mutex};

use analyzer::Analyzer;
use nih_plug::prelude::{util, AtomicF32, Editor, Vst3Plugin};
use nih_plug_vizia::vizia::image::{Pixel, Pixels};
use nih_plug_vizia::vizia::style::Color;
use nih_plug_vizia::vizia::vg::Canvas;
use nih_plug_vizia::vizia::{prelude::*, vg};
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use param_knob::ParamKnob;
use peak_curve::{Band, PeakCurve};

use crate::analyzer_data::AnalyzerData;
use crate::PluginParams;

mod analyzer;
mod param_knob;
mod peak_curve;

pub const COMFORTAA_LIGHT_TTF: &[u8] = include_bytes!("../res/Comfortaa-Light.ttf");
pub const COMFORTAA: &str = "Comfortaa";

const WIDTH: u32 = 850;
const HEIGHT: u32 = 700; 

const ANALYZER_WIDTH: f32 = 800.0;
const ANALYZER_HEIGHT: f32 = 225.0;

const TOP_KNOB_SPACE_WIDTH: f32 = 120.0f32;

const PANEL_COLOR: Color = Color::rgb(36, 35, 43);
const SPECTRUM_BORDER_COLOR: Color = Color::rgb(72, 71, 93);

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (WIDTH, HEIGHT))
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
            Label::new(cx, "SPECTRAL MULTIBAND COMPRESSOR")
            .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
            .font_size(28.0)
            .height(Pixels(50.0))
            .child_top(Stretch(1.0))
            .child_bottom(Stretch(1.0))
            .class("header-label");

            HStack::new(cx, |cx| {
                ParamKnob::new(cx, 
                    EditorData::plugin_data, |params| &params.attack_ms, false, 
                    String::from("top"), 
                    true)
                    .width(Pixels(TOP_KNOB_SPACE_WIDTH))
                    .height(Pixels(80.0))
                    .left(Pixels(94.0));
                ParamKnob::new(cx, 
                    EditorData::plugin_data, |params| &params.release_ms, false, 
                    String::from("top"), 
                    true)
                    .width(Pixels(TOP_KNOB_SPACE_WIDTH))
                    .height(Pixels(80.0));
                ParamKnob::new(cx, 
                    EditorData::plugin_data, |params| &params.smooth, false, 
                    String::from("top"), 
                    true)
                    .width(Pixels(TOP_KNOB_SPACE_WIDTH))
                    .height(Pixels(80.0));
                ParamKnob::new(cx, 
                    EditorData::plugin_data, |params| &params.mix, false, 
                    String::from("top"), 
                    true)
                    .width(Pixels(TOP_KNOB_SPACE_WIDTH))
                    .height(Pixels(80.0));
                ParamKnob::new(cx, 
                    EditorData::plugin_data, |params| &params.in_gain, false, 
                    String::from("top"), 
                    true)
                    .width(Pixels(TOP_KNOB_SPACE_WIDTH))
                    .height(Pixels(80.0));
                ParamKnob::new(cx, 
                    EditorData::plugin_data, |params| &params.out_gain, false, 
                    String::from("top"), 
                    true)
                    .width(Pixels(TOP_KNOB_SPACE_WIDTH))
                    .height(Pixels(80.0));
            })
            .height(Pixels(80.0))
            .width(Pixels(WIDTH as f32))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0));

            Analyzer::new(cx, EditorData::analyzer_data, EditorData::sample_rate)
                .left(Stretch(1.0))
                .right(Stretch(1.0))
                .width(Pixels(ANALYZER_WIDTH))
                .height(Pixels(ANALYZER_HEIGHT))
                .border_color(SPECTRUM_BORDER_COLOR)
                .border_width(Pixels(4.0));
            HStack::new(cx, |cx| {
                ParamSlider::new(cx, EditorData::plugin_data, |params| &params.low_mid_frequency)
                .set_style(ParamSliderStyle::FromLeft)
                .left(Pixels(225.0))
                .right(Pixels(50.0));
                ParamSlider::new(cx, EditorData::plugin_data, |params| &params.mid_high_frequency)
                .set_style(ParamSliderStyle::FromLeft)
                .left(Pixels(55.0))
                .right(Pixels(50.0));
            })
            .top(Pixels(10.0))
            .height(Pixels(35.0))
            .width(Pixels(ANALYZER_WIDTH));
            HStack::new(cx, |cx|{
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.low_gain, false, 
                            String::from("low"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.low_ratio, false, 
                            String::from("low"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                    })
                    .child_left(Pixels(30.0))
                    .child_right(Stretch(1.0))
                    .height(Pixels(80.0))
                    .width(Pixels(250.0));
                    
                    HStack::new(cx, |cx| {
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.low_threshold, false, 
                            String::from("low"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.low_up_ratio, false, 
                            String::from("low"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                    })
                    .child_left(Pixels(30.0))
                    .child_right(Stretch(1.0))
                    .height(Pixels(80.0))
                    .width(Pixels(250.0));

                    PeakCurve::new(cx, EditorData::analyzer_data, Band::Low)
                    .width(Pixels(100.0))
                    .height(Pixels(100.0))
                    .top(Pixels(15.0))
                    .left(Pixels(70.0));
                })
                .height(Pixels(300.0))
                .width(Pixels(250.0));

                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.mid_gain, false, 
                            String::from("mid"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.mid_ratio, false, 
                            String::from("mid"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                    })
                    .child_left(Pixels(30.0))
                    .child_right(Stretch(1.0))
                    .height(Pixels(80.0))
                    .width(Pixels(250.0));
                    
                    HStack::new(cx, |cx| {
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.mid_threshold, false, 
                            String::from("mid"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.mid_up_ratio, false, 
                            String::from("mid"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                    })
                    .child_left(Pixels(30.0))
                    .child_right(Stretch(1.0))
                    .height(Pixels(80.0))
                    .width(Pixels(250.0));

                    PeakCurve::new(cx, EditorData::analyzer_data, Band::Mid)
                    .width(Pixels(100.0))
                    .height(Pixels(100.0))
                    .top(Pixels(15.0))
                    .left(Pixels(70.0));
                })
                .height(Pixels(300.0))
                .width(Pixels(250.0));

                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.high_gain, false, 
                            String::from("high"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.high_ratio, false, 
                            String::from("high"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                    })
                    .child_left(Pixels(30.0))
                    .child_right(Stretch(1.0))
                    .height(Pixels(80.0))
                    .width(Pixels(250.0));
                    
                    HStack::new(cx, |cx| {
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.high_threshold, false, 
                            String::from("high"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, |params| &params.high_up_ratio, false, 
                            String::from("high"), 
                            true)
                            .width(Pixels(120.0))
                            .height(Pixels(80.0));
                    })
                    .child_left(Pixels(30.0))
                    .child_right(Stretch(1.0))
                    .height(Pixels(80.0))
                    .width(Pixels(250.0));

                    PeakCurve::new(cx, EditorData::analyzer_data, Band::High)
                    .width(Pixels(100.0))
                    .height(Pixels(100.0))
                    .top(Pixels(15.0))
                    .left(Pixels(70.0));
                })
                .height(Pixels(300.0))
                .width(Pixels(250.0));
            })
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0))
            .height(Pixels(300.0))
            .width(Pixels(ANALYZER_WIDTH));
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0))
        .class("main-gui");
    })
}
