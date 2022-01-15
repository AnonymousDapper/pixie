// MIT License

// Copyright (c) 2022 AnonmousDapper

use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

const WIDTH: f64 = crate::WINDOW_WIDTH as f64;
const HEIGHT: f64 = crate::WINDOW_HEIGHT as f64;

const MIN_WIDTH: f64 = crate::FB_WIDTH as f64;
const MIN_HEIGHT: f64 = crate::FB_HEIGHT as f64;

pub fn create_window(
    title: &str,
    event_loop: &EventLoop<()>,
) -> Result<(Window, u32, u32, f64), winit::error::OsError> {
    let window = WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(event_loop)?;

    let hidpi = window.scale_factor();

    let (_, display_height) = {
        if let Some(display) = window.current_monitor() {
            let size = display.size().to_logical(hidpi);
            (size.width, size.height)
        } else {
            (WIDTH, HEIGHT)
        }
    };

    let scale = (display_height / HEIGHT * 2.0 / 3.0).round().max(1.0);

    let min_size = PhysicalSize::new(WIDTH, HEIGHT).to_logical::<f64>(hidpi);
    let default = LogicalSize::new(MIN_WIDTH * scale, MIN_HEIGHT * scale);

    window.set_inner_size(default);
    window.set_min_inner_size(Some(min_size));
    window.set_visible(true);

    let size = default.to_physical::<f64>(hidpi);

    Ok((
        window,
        size.width.round() as u32,
        size.height.round() as u32,
        hidpi,
    ))
}
