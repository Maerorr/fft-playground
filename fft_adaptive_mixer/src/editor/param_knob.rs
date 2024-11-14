use nih_plug::prelude::Param;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

#[derive(Debug)]
pub enum ParamEvent {
    BeginSetParam,
    SetParam(f32),
    EndSetParam,
}

#[derive(Lens)]
pub struct ParamKnob {
    param_base: ParamWidgetBase,
}

impl ParamKnob {
    pub fn new<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
        centered: bool,
    ) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone + Copy,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {
            param_base: ParamWidgetBase::new(cx, params.clone(), params_to_param),
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, move |cx, param_data| {
                VStack::new(cx, |cx| {
                    // Label::new(
                    //     cx,
                    //     params.map(move |params| {
                    //         params_to_param(params)
                    //             .normalized_value_to_string(
                    //                 params_to_param(params)
                    //                     .modulated_normalized_value()
                    //                     .to_owned(),
                    //                 true,
                    //             )
                    //             .to_owned()
                    //     }),
                    // )
                    // .class("param-label")
                    // .space(Stretch(1.0));

                    Knob::custom(
                        cx,
                        param_data.param().default_normalized_value(),
                        params.map(move |params| {
                            params_to_param(params).unmodulated_normalized_value()
                        }),
                        move |cx, lens| {
                            TickKnob::new(
                                cx,
                                Percentage(40.0),
                                Pixels(2.),
                                Percentage(55.0),
                                300.0,
                                KnobMode::Continuous,
                            )
                            .value(lens.clone())
                            .class("tick");
                            ArcTrack::new(
                                cx,
                                centered,
                                Percentage(60.0),
                                Percentage(30.),
                                -150.,
                                150.,
                                KnobMode::Continuous,
                            )
                            .value(lens)
                            .class("track")
                        },
                    )
                    .space(Stretch(1.0))
                    .on_mouse_down(move |cx, _button| {
                        cx.emit(ParamEvent::BeginSetParam);
                    })
                    .on_changing(move |cx, val| {
                        cx.emit(ParamEvent::SetParam(val));
                    })
                    .on_mouse_up(move |cx, _button| {
                        cx.emit(ParamEvent::EndSetParam);
                    })
                    .class("param-knob");

                    Label::new(
                        cx,
                        params.map(move |params| params_to_param(params).name().to_owned()),
                    )
                    .class("param-name")
                    .left(Stretch(1.0))
                    .right(Stretch(1.0));
                })
                .class("param-knob-whole");
            }),
        )
    }
}

impl View for ParamKnob {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|param_change_event, _| match param_change_event {
            ParamEvent::BeginSetParam => {
                self.param_base.begin_set_parameter(cx);
            }
            ParamEvent::SetParam(val) => {
                self.param_base.set_normalized_value(cx, *val);
            }
            ParamEvent::EndSetParam => {
                self.param_base.end_set_parameter(cx);
            }
        });
    }
}
