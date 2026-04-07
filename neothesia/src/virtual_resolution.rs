use macroquad::prelude::*;

pub const VIRTUAL_W: f32 = 1920.0;
pub const VIRTUAL_H: f32 = 1080.0;

pub fn vw() -> f32 {
    VIRTUAL_W
}

pub fn vh() -> f32 {
    VIRTUAL_H
}

fn scale_factor() -> f32 {
    let sw = screen_width();
    let sh = screen_height();
    (sw / VIRTUAL_W).min(sh / VIRTUAL_H)
}

fn letterbox_offset() -> (f32, f32) {
    let sw = screen_width();
    let sh = screen_height();
    let scale = scale_factor();
    let ox = (sw - VIRTUAL_W * scale) / 2.0;
    let oy = (sh - VIRTUAL_H * scale) / 2.0;
    (ox, oy)
}

pub fn screen_to_virtual(sx: f32, sy: f32) -> (f32, f32) {
    let scale = scale_factor();
    let (ox, oy) = letterbox_offset();
    let vx = (sx - ox) / scale;
    let vy = (sy - oy) / scale;
    (vx, vy)
}

pub fn vmouse() -> (f32, f32) {
    let (mx, my) = mouse_position();
    screen_to_virtual(mx, my)
}

pub fn setup_camera() -> Camera2D {
    let sw = screen_width();
    let sh = screen_height();
    let scale = scale_factor();

    let camera = Camera2D {
        target: Vec2::new(VIRTUAL_W / 2.0, VIRTUAL_H / 2.0),
        zoom: Vec2::new(2.0 * scale / sw, 2.0 * scale / sh),
        offset: Vec2::new(0.0, 0.0),
        rotation: 0.0,
        render_target: None,
        viewport: None,
    };

    set_camera(&camera);
    camera
}

pub fn clear_letterbox() {
    let sw = screen_width();
    let sh = screen_height();
    let scale = scale_factor();
    let (ox, oy) = letterbox_offset();

    let black = Color::new(0.0, 0.0, 0.0, 1.0);

    if ox > 0.0 {
        draw_rectangle(0.0, 0.0, ox, sh, black);
        draw_rectangle(sw - ox, 0.0, ox, sh, black);
    }
    if oy > 0.0 {
        draw_rectangle(0.0, 0.0, sw, oy, black);
        draw_rectangle(0.0, sh - oy, sw, oy, black);
    }
}
