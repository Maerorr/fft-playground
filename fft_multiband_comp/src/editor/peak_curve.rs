use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{prelude::*, vg};
use crate::{analyzer_data::AnalyzerData, utils};

#[derive(Debug, Clone)]
pub struct PeakCurve {
    analyzer_data: Arc<Mutex<triple_buffer::Output<AnalyzerData>>>,
}

impl PeakCurve {
    pub fn new<LAnalyzerData>(cx: &mut Context, analyzer_data: LAnalyzerData) -> Handle<Self> 
    where
        LAnalyzerData: Lens<Target = Arc<Mutex<triple_buffer::Output<AnalyzerData>>>>,
    {
        Self {
            analyzer_data: analyzer_data.get(cx),
        }.build(cx, |_cx| ())
    }
}

impl View for PeakCurve {
    fn element(&self) -> Option<&'static str> {
        Some("PeakCurve")
    }

    fn draw(
        &self,
        cx: &mut nih_plug_vizia::vizia::context::DrawContext,
        canvas: &mut nih_plug_vizia::vizia::view::Canvas,
    ) {
        let bounds = cx.bounds();
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }
        let mut path = vg::Path::new();
        path.move_to(bounds.x, bounds.y + bounds.h);
        let p = self.analyzer_data.lock().unwrap().read().p;
        let one_over_p = 1.0f32 / p;
        for i in 0..50 {
            let x = i as f32 / 49.0f32;
            let y = bounds.y + (1.0 - utils::calculate_peakness(x, p, one_over_p)) * bounds.h;
            path.line_to(utils::lerp(bounds.x, bounds.x + bounds.w, x), y);
        }
        let paint = vg::Paint::color(vg::Color::white()).with_line_width(1.0);
        canvas.stroke_path(&path, &paint);
    }
}