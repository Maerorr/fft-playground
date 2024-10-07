use std::sync::Arc;

use nih_plug::editor;
use nih_plug::prelude::{util, Editor, Vst3Plugin};
use nih_plug_vizia::vizia::image::Pixel;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::resource::ImageRetentionPolicy;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use param_knob::ParamKnob;

use crate::FFTGateParams;

mod param_knob;

pub const COMFORTAA_LIGHT_TTF: &[u8] = include_bytes!("../res/Comfortaa-Light.ttf");
pub const COMFORTAA: &str = "Comfortaa";

#[derive(Lens)]
struct Data {
    plugin_data: Arc<FFTGateParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (400, 350))
}

pub(crate) fn create(
    plugin_data: Arc<FFTGateParams>,
    editor_state: Arc<ViziaState>) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {

        cx.add_font_mem(&COMFORTAA_LIGHT_TTF);
        cx.set_default_font(&[COMFORTAA]);

        Data {
            plugin_data: plugin_data.clone(),
        }.build(cx);

        ResizeHandle::new(cx);
            VStack::new(cx, |cx| {
                Label::new(cx, "MAEROR'S SPECTRAL GATE")
                .font_family(vec![FamilyOwned::Name(String::from(COMFORTAA))])
                .font_size(24.0)
                .height(Pixels(75.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0));
                
                HStack::new(cx, |cx| {
                    ParamKnob::new(cx, Data::plugin_data, |params| &params.threshold, false);
                    //ParamKnob::new(cx, Data::plugin_data, |params| &params.attack, false);
                    //ParamKnob::new(cx, Data::plugin_data, |params| &params.release, false);
                }).col_between(Pixels(75.0));
                
            }).row_between(Pixels(0.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0));
    })
}