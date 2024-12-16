use std::sync::{Arc, Mutex};
use nih_plug::nih_log;

use nih_plug_vizia::vizia::{prelude::*, vg};
use crate::{analyzer_data::AnalyzerData, utils};

#[derive(Debug, Clone)]
pub enum Band {
    Low,
    Mid,
    High
}

#[derive(Debug, Clone)]
pub struct PeakCurve {
    analyzer_data: Arc<Mutex<triple_buffer::Output<AnalyzerData>>>,
    band: Band,
}

impl PeakCurve {
    pub fn new<LAnalyzerData>(cx: &mut Context, analyzer_data: LAnalyzerData, band: Band) -> Handle<Self> 
    where
        LAnalyzerData: Lens<Target = Arc<Mutex<triple_buffer::Output<AnalyzerData>>>>,
    {
        Self {
            analyzer_data: analyzer_data.get(cx),
            band,
        }.build(cx, |_cx| ())
    }
}

#[inline]
pub fn db_to_01(db: f32) -> f32 {
    ((db + 100.0) / 100.0).clamp(0.0, 1.0)
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
        let data;
        match self.band {
            // i dont like copying this here but w/e
            Band::Low => data = self.analyzer_data.lock().unwrap().read().comp_curve_low.to_vec(),
            Band::Mid => data = self.analyzer_data.lock().unwrap().read().comp_curve_mid.to_vec(),
            Band::High => data = self.analyzer_data.lock().unwrap().read().comp_curve_high.to_vec(),
        }
        let mut path = vg::Path::new();
        path.move_to(bounds.x, bounds.y + bounds.w * (1.0 - db_to_01(data[0])));
        for (i, el) in data.iter().skip(1).enumerate() { 
            let x = (i + 1) as f32 / 49.0f32;
            let y = bounds.y + bounds.w * (1.0 - db_to_01(*el));
            path.line_to(utils::lerp(bounds.x, bounds.x + bounds.w, x), y);
        }
        let paint = vg::Paint::color(vg::Color::white()).with_line_width(1.0);
        canvas.stroke_path(&path, &paint);

        let mut path = vg::Path::new();
        path.move_to(bounds.x, bounds.y);
        path.line_to(bounds.x + bounds.w, bounds.y);
        path.line_to(bounds.x + bounds.w, bounds.y + bounds.h);
        path.line_to(bounds.x, bounds.y + bounds.h);
        path.line_to(bounds.x, bounds.y);

        let paint = vg::Paint::color(vg::Color::white()).with_line_width(1.0);
        canvas.stroke_path(&path, &paint);

    }
}