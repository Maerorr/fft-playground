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
use peak_curve::PeakCurve;

use crate::analyzer_data::AnalyzerData;
use crate::PluginParams;

mod analyzer;
mod param_knob;
mod peak_curve;

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
                    HStack::new(cx, |cx| {
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, 
                            |params| &params.lowcut, 
                            false, 
                            String::from("cut"),
                            true
                        ).width(Pixels(100.0))
                        .left(Pixels(15.0));
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, 
                            |params| &params.highcut, 
                            false, 
                            String::from("cut"),
                            true
                        ).width(Pixels(100.0));
                    })
                    .height(Pixels(100.0))
                    .width(Pixels(210.0));

                    ParamKnob::new(cx, 
                        EditorData::plugin_data, 
                        |params| &params.amount, 
                        false, 
                        String::from("main"),
                        true
                    )
                    .width(Pixels(140.0))
                    .left(Stretch(1.0))
                    .right(Stretch(1.0))
                    .bottom(Pixels(-10.0))
                    .height(Pixels(150.0));

                    HStack::new(cx, |cx| {
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, 
                            |params| &params.gate, 
                            false, 
                            String::from("mid"),
                            true
                        ).width(Pixels(65.0));
                        ParamKnob::new(cx, 
                            EditorData::plugin_data, 
                            |params| &params.smooth, 
                            false, 
                            String::from("mid"),
                            false
                        ).width(Pixels(65.0));

                        VStack::new(cx, |cx| {
                            PeakCurve::new(cx, EditorData::analyzer_data)
                            .width(Pixels(20.0))
                            .height(Pixels(20.0))
                            .left(Pixels(20.0))
                            .bottom(Pixels(-20.0));

                            ParamKnob::new(cx, 
                                EditorData::plugin_data, 
                                |params| &params.peakiness, 
                                false, 
                                String::from("mid"),
                                false
                                )
                                .width(Pixels(65.0))
                                .height(Pixels(100.0))
                                .bottom(Pixels(30.0));
                        }).height(Pixels(100.0));
                    })
                    .left(Pixels(5.0))
                    .height(Pixels(100.0))
                    .width(Pixels(210.0));

                    
                    
                    VStack::new(cx, |cx| {
                        ParamSlider::new(cx, EditorData::plugin_data, |params| &params.fft_size)
                            .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                            .font_size(13.0)
                            .top(Pixels(8.0))
                            .height(Pixels(20.0))
                            .width(Pixels(100.0));
                        Label::new(cx, "FFT Size:")
                            .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                            .font_size(13.0)
                            .height(Pixels(22.0))
                            .top(Pixels(2.0))
                            .space(Stretch(1.0));
                    })
                    .row_between(Pixels(0.0))
                    .height(Pixels(42.0))
                    .child_left(Stretch(1.0))
                    .child_right(Stretch(1.0))
                    .bottom(Pixels(10.0));

                    VStack::new(cx, |cx| {
                        ParamSlider::new(cx, EditorData::plugin_data, |params| {
                            &params.analyzer_channel
                        })
                            .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                            .font_size(13.0)
                            .top(Pixels(8.0))
                            .height(Pixels(20.0))
                            .width(Pixels(100.0));
                        Label::new(cx, "Analyzer Channel:")
                            .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                            .font_size(13.0)
                            .height(Pixels(22.0))
                            .top(Pixels(2.0))
                            .space(Stretch(1.0));
                    })
                    .row_between(Pixels(0.0))
                    .height(Pixels(42.0))
                    .child_left(Stretch(1.0))
                    .child_right(Stretch(1.0))
                    .bottom(Pixels(10.0));
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

                    let mut i = 0;
                    // Spectrum params go here:
                    HStack::new(cx, |cx| {
                        VStack::new(cx, |cx| {
                            Label::new(cx, format!("({}, {})", EQ_FREQS[i].floor(), EQ_FREQS[i+1].floor()).as_str())
                            .top(Pixels(5.0))
                            .height(Pixels(25.0))
                            .left(Pixels(37.0))
                            .right(Stretch(1.0))
                            .font_size(10.0);
                            ParamKnob::new(
                                cx,
                                EditorData::plugin_data,
                                |params| &params.eq1,
                                false,
                                String::from("eq"),
                                true
                            ).left(Pixels(20.0));
                        })
                        .left(Pixels(105.0))
                        .width(Pixels(70.0))
                        .height(Pixels(120.0));
                        i+=1;
                        VStack::new(cx, |cx| {
                            Label::new(cx, format!("({}, {})", EQ_FREQS[i].floor(), EQ_FREQS[i+1].floor()).as_str())
                            .top(Pixels(5.0))
                            .height(Pixels(25.0))
                            .left(Pixels(35.0))
                            .right(Stretch(1.0))
                            .font_size(10.0);
                            ParamKnob::new(
                                cx,
                                EditorData::plugin_data,
                                |params| &params.eq2,
                                false,
                                String::from("eq"),
                                true
                            ).left(Pixels(20.0));
                        }).width(Pixels(70.0))
                        .height(Pixels(120.0));
                        i+=1;
                        VStack::new(cx, |cx| {
                            Label::new(cx, format!("({}, {})", EQ_FREQS[i].floor(), EQ_FREQS[i+1].floor()).as_str())
                            .top(Pixels(5.0))
                            .height(Pixels(25.0))
                            .left(Pixels(32.0))
                            .right(Stretch(1.0))
                            .font_size(10.0);
                            ParamKnob::new(
                                cx,
                                EditorData::plugin_data,
                                |params| &params.eq3,
                                false,
                                String::from("eq"),
                                true
                            ).left(Pixels(20.0));
                        }).width(Pixels(70.0))
                        .height(Pixels(120.0));
                        i+=1;
                        VStack::new(cx, |cx| {
                            Label::new(cx, format!("({}, {})", EQ_FREQS[i].floor(), EQ_FREQS[i+1].floor()).as_str())
                            .top(Pixels(5.0))
                            .height(Pixels(25.0))
                            .left(Pixels(32.0))
                            .right(Stretch(1.0))
                            .font_size(10.0);
                            ParamKnob::new(
                                cx,
                                EditorData::plugin_data,
                                |params| &params.eq4,
                                false,
                                String::from("eq"),
                                true
                            ).left(Pixels(20.0));
                        }).width(Pixels(70.0))
                        .height(Pixels(120.0));
                        i+=1;
                        VStack::new(cx, |cx| {
                            Label::new(cx, format!("({}, {})", EQ_FREQS[i].floor(), EQ_FREQS[i+1].floor()).as_str())
                            .top(Pixels(5.0))
                            .height(Pixels(25.0))
                            .left(Pixels(28.0))
                            .right(Stretch(1.0))
                            .font_size(10.0);
                            ParamKnob::new(
                                cx,
                                EditorData::plugin_data,
                                |params| &params.eq5,
                                false,
                                String::from("eq"),
                                true
                            ).left(Pixels(20.0));
                        }).width(Pixels(70.0))
                        .height(Pixels(120.0));
                        i+=1;
                        VStack::new(cx, |cx| {
                            Label::new(cx, format!("({}, {})", EQ_FREQS[i].floor(), EQ_FREQS[i+1].floor()).as_str())
                            .top(Pixels(5.0))
                            .height(Pixels(25.0))
                            .left(Pixels(27.0))
                            .right(Stretch(1.0))
                            .font_size(10.0);
                            ParamKnob::new(
                                cx,
                                EditorData::plugin_data,
                                |params| &params.eq6,
                                false,
                                String::from("eq"),
                                true
                            ).left(Pixels(20.0));
                        }).width(Pixels(70.0))
                        .height(Pixels(120.0));
                        i+=1;
                        VStack::new(cx, |cx| {
                            Label::new(cx, format!("({}, {})", EQ_FREQS[i].floor(), EQ_FREQS[i+1].floor()).as_str())
                            .top(Pixels(5.0))
                            .height(Pixels(25.0))
                            .left(Pixels(26.0))
                            .right(Stretch(1.0))
                            .font_size(10.0);
                            ParamKnob::new(
                                cx,
                                EditorData::plugin_data,
                                |params| &params.eq7,
                                false,
                                String::from("eq"),
                                true
                            ).left(Pixels(20.0));
                        }).width(Pixels(70.0))
                        .height(Pixels(120.0));
                        i+=1;
                        VStack::new(cx, |cx| {
                            Label::new(cx, format!("({}, {})", EQ_FREQS[i].floor(), EQ_FREQS[i+1].floor()).as_str())
                            .top(Pixels(5.0))
                            .height(Pixels(25.0))
                            .left(Pixels(24.0))
                            .right(Stretch(1.0))
                            .font_size(10.0);
                            ParamKnob::new(
                                cx,
                                EditorData::plugin_data,
                                |params| &params.eq8,
                                false,
                                String::from("eq"),
                                true
                            ).left(Pixels(20.0));
                        }).width(Pixels(70.0))
                        .height(Pixels(120.0));

                    })
                    .width(Pixels(850.0))
                    .height(Pixels(HEIGHT as f32 - 400.0))
                    .background_color(PANEL_COLOR);
                })
                .row_between(Pixels(0.0))
                .child_left(Stretch(1.0))
                .child_right(Stretch(1.0))
                .width(Pixels(850.0));
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
