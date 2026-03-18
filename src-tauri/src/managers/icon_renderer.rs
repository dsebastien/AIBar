use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Rect, Transform};

const ICON_SIZE: u32 = 32;
const BAR_WIDTH: f32 = 10.0;
const BAR_GAP: f32 = 4.0;
const BAR_MARGIN_X: f32 = 4.0;
const BAR_MARGIN_Y: f32 = 2.0;

/// Determine the bar color based on a usage percentage (0.0 to 1.0).
fn bar_color(pct: f64) -> Color {
    if pct >= 0.8 {
        // Red
        Color::from_rgba8(239, 68, 68, 255)
    } else if pct >= 0.5 {
        // Yellow
        Color::from_rgba8(234, 179, 8, 255)
    } else {
        // Green
        Color::from_rgba8(34, 197, 94, 255)
    }
}

/// Render a 32x32 RGBA tray icon with two vertical bars indicating usage percentages.
///
/// - `primary_pct`: usage percentage for the left bar (0.0..=1.0)
/// - `secondary_pct`: usage percentage for the right bar (0.0..=1.0)
///
/// Returns RGBA pixel data suitable for `tauri::image::Image::from_rgba()`.
pub fn render_tray_icon(primary_pct: f64, secondary_pct: f64) -> Vec<u8> {
    let primary_pct = primary_pct.clamp(0.0, 1.0);
    let secondary_pct = secondary_pct.clamp(0.0, 1.0);

    let mut pixmap = Pixmap::new(ICON_SIZE, ICON_SIZE).expect("failed to create pixmap");

    // Draw background (transparent)
    // Pixmap is already zeroed (transparent), nothing to do.

    let max_bar_height = ICON_SIZE as f32 - 2.0 * BAR_MARGIN_Y;

    // Draw left bar (primary)
    draw_bar(
        &mut pixmap,
        BAR_MARGIN_X,
        BAR_MARGIN_Y,
        BAR_WIDTH,
        max_bar_height,
        primary_pct,
    );

    // Draw right bar (secondary)
    draw_bar(
        &mut pixmap,
        BAR_MARGIN_X + BAR_WIDTH + BAR_GAP,
        BAR_MARGIN_Y,
        BAR_WIDTH,
        max_bar_height,
        secondary_pct,
    );

    pixmap.data().to_vec()
}

fn draw_bar(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    max_height: f32,
    pct: f64,
) {
    // Draw track (dark background)
    let mut track_paint = Paint::default();
    track_paint.set_color(Color::from_rgba8(60, 60, 60, 180));
    track_paint.anti_alias = true;

    if let Some(track_rect) = Rect::from_xywh(x, y, width, max_height) {
        if let Some(path) = {
            let mut pb = PathBuilder::new();
            pb.push_rect(track_rect);
            pb.finish()
        } {
            pixmap.fill_path(&path, &track_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }

    // Draw filled portion (from bottom up)
    let fill_height = (max_height * pct as f32).max(1.0);
    let fill_y = y + max_height - fill_height;

    let mut fill_paint = Paint::default();
    fill_paint.set_color(bar_color(pct));
    fill_paint.anti_alias = true;

    if let Some(fill_rect) = Rect::from_xywh(x, fill_y, width, fill_height) {
        if let Some(path) = {
            let mut pb = PathBuilder::new();
            pb.push_rect(fill_rect);
            pb.finish()
        } {
            pixmap.fill_path(&path, &fill_paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        }
    }
}

/// Render a default (idle) tray icon with both bars at 0%.
pub fn render_default_icon() -> Vec<u8> {
    render_tray_icon(0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_has_correct_size() {
        let data = render_tray_icon(0.5, 0.75);
        assert_eq!(data.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
    }

    #[test]
    fn extreme_values_are_clamped() {
        let data = render_tray_icon(-0.5, 1.5);
        assert_eq!(data.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
    }
}
