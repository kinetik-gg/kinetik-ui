//! Windowed Kinetik UI showcase entry point.

mod live;

use std::fmt;

use kinetik_ui::{
    core::{PhysicalSize, ScaleFactor, Size, ViewportInfo},
    render::{RenderDiagnostic, RenderFrameInput},
    render_vello::VelloRenderer,
};
use kinetik_ui_showcase::{
    app::ShowcaseApp,
    raster::{rasterize, write_bmp},
};

const DEFAULT_WIDTH: usize = 1440;
const DEFAULT_HEIGHT: usize = 900;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--list") {
        for scenario in kinetik_ui_showcase::all_scenarios() {
            println!(
                "{}: {} primitives",
                scenario.name,
                scenario.primitives.len()
            );
        }
        return Ok(());
    }

    if let Some(path) = render_once_path(&args) {
        let width = usize_arg(&args, "--width").unwrap_or(DEFAULT_WIDTH);
        let height = usize_arg(&args, "--height").unwrap_or(DEFAULT_HEIGHT);
        let scale_factor = f64_arg(&args, "--scale").unwrap_or(1.0);
        let mut app = ShowcaseApp::new();
        app.set_viewport_size(logical_size_from_pixels(width, height, scale_factor));
        if let Some(page) = page_arg(&args).and_then(ShowcaseApp::page_from_name) {
            app.set_page(page);
        }

        submit_render_once_to_vello(&app, width, height, scale_factor)?;
        // The BMP preview writer is a lightweight diagnostic fallback; Vello
        // submission above keeps render-once aligned with the live text path.
        let frame = rasterize(&app.primitives(), width, height);
        write_bmp(&frame, path)?;
        return Ok(());
    }

    live::run(page_arg(&args).and_then(ShowcaseApp::page_from_name))?;
    Ok(())
}

fn render_once_path(args: &[String]) -> Option<&str> {
    args.windows(2)
        .find_map(|window| (window[0] == "--render-once").then_some(window[1].as_str()))
}

fn page_arg(args: &[String]) -> Option<&str> {
    args.windows(2)
        .find_map(|window| (window[0] == "--page").then_some(window[1].as_str()))
}

fn usize_arg(args: &[String], name: &str) -> Option<usize> {
    args.windows(2)
        .find_map(|window| (window[0] == name).then(|| window[1].parse().ok()))
        .flatten()
}

fn f64_arg(args: &[String], name: &str) -> Option<f64> {
    args.windows(2)
        .find_map(|window| (window[0] == name).then(|| window[1].parse().ok()))
        .flatten()
}

fn submit_render_once_to_vello(
    app: &ShowcaseApp,
    width: usize,
    height: usize,
    scale_factor: f64,
) -> Result<VelloRenderer, RenderOnceVelloError> {
    let viewport = render_once_viewport(app, width, height, scale_factor)?;
    let resources = app.render_resources();
    let mut renderer = VelloRenderer::new();
    let output = renderer.submit_frame(RenderFrameInput {
        viewport,
        primitives: &app.output().primitives,
        resources: &resources,
    });

    if output.diagnostics.is_empty() {
        Ok(renderer)
    } else {
        Err(RenderOnceVelloError::Diagnostics(output.diagnostics))
    }
}

fn render_once_viewport(
    app: &ShowcaseApp,
    width: usize,
    height: usize,
    scale_factor: f64,
) -> Result<ViewportInfo, RenderOnceVelloError> {
    let scale_factor = ScaleFactor::new(scale_factor);
    if !scale_factor.is_valid() {
        return Err(RenderOnceVelloError::InvalidScaleFactor);
    }

    Ok(ViewportInfo::new(
        app.viewport_size(),
        PhysicalSize::new(pixel_to_u32(width), pixel_to_u32(height)),
        scale_factor,
    ))
}

fn logical_size_from_pixels(width: usize, height: usize, scale_factor: f64) -> Size {
    let scale_factor = ScaleFactor::new(scale_factor);
    if scale_factor.is_valid() {
        scale_factor
            .physical_size_to_logical(PhysicalSize::new(pixel_to_u32(width), pixel_to_u32(height)))
    } else {
        Size::new(pixel_to_f32(width), pixel_to_f32(height))
    }
}

fn pixel_to_f32(value: usize) -> f32 {
    let value = u16::try_from(value).unwrap_or(u16::MAX);
    f32::from(value)
}

fn pixel_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RenderOnceVelloError {
    InvalidScaleFactor,
    Diagnostics(Vec<RenderDiagnostic>),
}

impl fmt::Display for RenderOnceVelloError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScaleFactor => write!(formatter, "invalid render-once scale factor"),
            Self::Diagnostics(diagnostics) => {
                write!(formatter, "render-once Vello diagnostics: {diagnostics:?}")
            }
        }
    }
}

impl std::error::Error for RenderOnceVelloError {}

#[cfg(test)]
mod tests {
    use super::{
        f64_arg, logical_size_from_pixels, render_once_viewport, submit_render_once_to_vello,
        usize_arg,
    };
    use kinetik_ui_showcase::app::ShowcaseApp;

    #[test]
    fn render_once_cli_parses_scale_and_dimensions() {
        let args = [
            "showcase".to_owned(),
            "--render-once".to_owned(),
            "frame.bmp".to_owned(),
            "--width".to_owned(),
            "1440".to_owned(),
            "--height".to_owned(),
            "900".to_owned(),
            "--scale".to_owned(),
            "1.25".to_owned(),
        ];

        assert_eq!(usize_arg(&args, "--width"), Some(1440));
        assert_eq!(usize_arg(&args, "--height"), Some(900));
        assert_eq!(f64_arg(&args, "--scale"), Some(1.25));
    }

    #[test]
    fn render_once_viewport_uses_scaled_logical_size() {
        let mut app = ShowcaseApp::new();
        app.set_viewport_size(logical_size_from_pixels(1440, 900, 1.25));

        let viewport = render_once_viewport(&app, 1440, 900, 1.25).expect("viewport");

        assert_eq!(viewport.physical_size.width, 1440);
        assert_eq!(viewport.physical_size.height, 900);
        assert_eq!(viewport.logical_size, app.viewport_size());
        assert!((viewport.scale_factor.value() - 1.25).abs() < f64::EPSILON);
    }

    #[test]
    fn render_once_rejects_invalid_scale_factor() {
        let app = ShowcaseApp::new();

        assert!(render_once_viewport(&app, 1440, 900, 0.0).is_err());
        assert!(render_once_viewport(&app, 1440, 900, f64::NAN).is_err());
    }

    #[test]
    fn render_once_submits_fractional_dpi_text_through_vello() {
        let mut app = ShowcaseApp::new();
        app.set_viewport_size(logical_size_from_pixels(1440, 900, 1.25));

        let renderer =
            submit_render_once_to_vello(&app, 1440, 900, 1.25).expect("vello submission");
        let encoding = renderer.scene().encoding();
        let glyph_runs = &encoding.resources.glyph_runs;
        let glyphs = &encoding.resources.glyphs;

        assert!(!glyph_runs.is_empty());
        assert!(!glyphs.is_empty());
        assert!(
            glyph_runs.iter().all(|run| run.hint),
            "render-once should use hinted physical text for axis-aligned showcase glyphs"
        );
        assert!(
            glyphs
                .iter()
                .all(|glyph| (glyph.x - glyph.x.round()).abs() <= 0.001),
            "render-once should snap physical glyph x positions"
        );
        assert!(
            glyphs
                .iter()
                .all(|glyph| (glyph.y - glyph.y.round()).abs() <= 0.001),
            "render-once should snap physical glyph baselines"
        );
    }
}
