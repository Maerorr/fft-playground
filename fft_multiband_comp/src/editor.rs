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

use crate::analyzer_data::AnalyzerData;
use crate::PluginParams;

mod analyzer;
mod param_knob;

pub const COMFORTAA_LIGHT_TTF: &[u8] = include_bytes!("../res/Comfortaa-Light.ttf");
pub const COMFORTAA: &str = "Comfortaa";
pub const EQ_FREQS: [f32; 9] = [20.0, 189.32, 368.40, 716.87, 1394.95, 2714.41, 5281.95, 10278.08, 20000.0]; 

const WIDTH: u32 = 1050;
const HEIGHT: u32 = 545; 

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
            HStack::new(cx, |cx| {
                //params go here \/
                VStack::new(cx, |cx| {
                    
                })
                .row_between(Pixels(0.0))
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0))
                .width(Pixels(200.0))
                .height(Pixels(HEIGHT as f32))
                .background_color(PANEL_COLOR);

                // ANALYZER + EQ
                VStack::new(cx, |cx| {
                    Analyzer::new(cx, EditorData::analyzer_data, EditorData::sample_rate)
                        .left(Stretch(1.0))
                        .right(Stretch(1.0))
                        .width(Pixels(850.0))
                        .height(Pixels(400.0))
                        .border_color(SPECTRUM_BORDER_COLOR)
                        .border_width(Pixels(4.0));

                    VStack::new(cx, |cx| {
                        HStack::new(cx, |cx| {
                            // LOW BAND PARAMS
                            HStack::new(cx, |cx| {
                                ParamKnob::new(cx, 
                                    EditorData::plugin_data, |params| &params.low_threshold, false, 
                                    String::from("mid"), 
                                    true);
                                ParamKnob::new(cx, 
                                    EditorData::plugin_data, |params| &params.low_gain, false, 
                                    String::from("mid"), 
                                    true);
                            })
                            .width(Pixels(200.0))
                            .height(Pixels(80.0))
                            .child_left(Pixels(10.0))
                            .child_right(Pixels(10.0))
                            .right(Stretch(1.0))
                            .left(Stretch(1.0));;
    
                            // MID BAND PARAMS
                            HStack::new(cx, |cx| {
                                ParamKnob::new(cx, 
                                    EditorData::plugin_data, |params| &params.mid_threshold, false, 
                                    String::from("mid"), 
                                    true);
                                ParamKnob::new(cx, 
                                    EditorData::plugin_data, |params| &params.mid_gain, false, 
                                    String::from("mid"), 
                                    true);
                            })
                            .width(Pixels(200.0))
                            .height(Pixels(80.0))
                            .child_left(Pixels(10.0))
                            .child_right(Pixels(10.0))
                            .right(Stretch(1.0))
                            .left(Stretch(1.0));
    
                            // HIGH BAND PARAMS
                            HStack::new(cx, |cx| {
                                ParamKnob::new(cx, 
                                    EditorData::plugin_data, |params| &params.high_threshold, false, 
                                    String::from("mid"), 
                                    true);
                                ParamKnob::new(cx, 
                                    EditorData::plugin_data, |params| &params.high_gain, false, 
                                    String::from("mid"), 
                                    true);
                            })
                            .width(Pixels(200.0))
                            .height(Pixels(80.0))
                            .child_left(Pixels(10.0))
                            .child_right(Pixels(10.0))
                            .right(Stretch(1.0))
                            .left(Stretch(1.0));
                        })
                        .height(Pixels(100.0))
                        .width(Pixels(850.0));

                        HStack::new(cx, |cx| {
                            ParamSlider::new(cx, EditorData::plugin_data, |params| &params.low_mid_frequency)
                            .left(Pixels(233.0))
                            .right(Pixels(50.0));
                            ParamSlider::new(cx, EditorData::plugin_data, |params| &params.mid_high_frequency)
                            .left(Pixels(50.0))
                            .right(Pixels(50.0));
                        })
                        //.top(Pixels(10.0))
                        .height(Pixels(65.0))
                        .width(Pixels(850.0));
                    })
                    .height(Pixels(145.0))
                    .width(Pixels(850.0));
                    
                })
                .row_between(Pixels(0.0))
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0))
                .width(Pixels(850.0))
                .background_color(PANEL_COLOR);
            })
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0))
            .max_height(Pixels(HEIGHT as f32));
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0))
        .class("main-gui");
    })
}
