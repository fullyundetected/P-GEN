use egui::{Color32, Painter, Pos2, Rect};

pub fn draw_vertical_line(painter: &Painter, x: f32, y_start: f32, y_end: f32, width: f32, color: Color32) {
    let rect = painter.round_rect_to_pixels(Rect::from_min_max(Pos2::new(x, y_start), Pos2::new(x + width, y_end)));
    painter.rect_filled(rect, 0.0, color);
}

pub fn draw_horizontal_line(painter: &Painter, x_start: f32, x_end: f32, y: f32, height: f32, color: Color32) {
    let rect = painter.round_rect_to_pixels(Rect::from_min_max(Pos2::new(x_start, y), Pos2::new(x_end, y + height)));
    painter.rect_filled(rect, 0.0, color);
}

pub fn draw_rect_stroke(painter: &Painter, rect: Rect, color: Color32) {
    draw_horizontal_line(painter, rect.min.x, rect.max.x, rect.min.y, 1.0, color);
    draw_horizontal_line(painter, rect.min.x, rect.max.x, rect.max.y, 1.0, color);
    draw_vertical_line(painter, rect.min.x, rect.min.y, rect.max.y, 1.0, color);
    draw_vertical_line(painter, rect.max.x, rect.min.y, rect.max.y + 1.0, 1.0, color);
}