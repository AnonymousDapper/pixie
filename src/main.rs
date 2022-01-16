// MIT License

// Copyright (c) 2022 AnonmousDapper

#![deny(rust_2018_idioms)]

use pixels::{PixelsBuilder, SurfaceTexture};

use winit::{
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
};

use winit_input_helper::WinitInputHelper;

use log::error;

use pixie::canvas::{Canvas, Render};

use pixie::simulation::{Particle, Point, Simulator, Velocity};

// =======================================
//
// TODO:
//
//     Particle system
//      - particle object
//      - environment object (physics calcs, coordinate normalization, render details)
//      -
//
// =======================================

fn rgb(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    (r, g, b)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let (window, window_width, window_height, _hipdi) =
        pixie::window::create_window("P.I.X.I.E.", &event_loop)?;

    let mut pixels = {
        let surtex = SurfaceTexture::new(window_width, window_height, &window);
        PixelsBuilder::new(pixie::FB_WIDTH as u32, pixie::FB_HEIGHT as u32, surtex)
            //.wgpu_backend(pixels::wgpu::Backends::VULKAN)
            .texture_format(pixie::PIPELINE_TEXTURE_FORMAT)
            .render_texture_format(pixie::PIPELINE_TEXTURE_FORMAT)
            .enable_vsync(true)
            .build()?
    };

    let mut shader = pixie::pipeline::ShaderPipeline::new(
        &pixels,
        pixie::FB_WIDTH as u32,
        pixie::FB_HEIGHT as u32,
    );

    let mut sim = Simulator::new();
    let mut canvas = Canvas::new();

    sim.add_particle(
        Particle::new(
            Point::new(80., 80.),
            Velocity::new(0., 0.02),
            rgb(255, 202, 40),
            400.,
            11.,
            7e9,
            0.,
        )
        .with_name("Sol"),
    );

    sim.add_particle(
        Particle::new(
            Point::new(80., 90.),
            Velocity::new(-0.1, 0.0),
            rgb(105, 240, 174),
            1700.,
            7.,
            7e7,
            0.,
        )
        .with_name("green blob"),
    );

    sim.add_particle(
        Particle::new(
            Point::new(10., 28.),
            Velocity::new(0.2, 0.05),
            rgb(25, 118, 210),
            0.,
            0.,
            7e3,
            0.,
        )
        .with_name("blue"),
    );

    event_loop.run(move |evt, _, flow| {
        if let Event::RedrawRequested(_) = evt {
            canvas.clear();
            sim.render(&mut canvas);
            canvas.render_to(pixels.get_frame());

            //let result = pixels.render();

            let result = pixels.render_with(|encoder, target, ctx| {
                let texture_buffer = shader.get_texture_view();
                ctx.scaling_renderer.render(encoder, texture_buffer);

                shader.render(encoder, target, ctx.scaling_renderer.clip_rect());

                Ok(())
            });

            if result
                .map_err(|e| error!("pixels render failed: {}", e))
                .is_err()
            {
                *flow = ControlFlow::Exit;
            }
        }

        if input.update(&evt) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *flow = ControlFlow::Exit;
            }

            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                shader.resize(&pixels, size.width, size.height);
            }

            sim.step_physics();
            window.request_redraw();
        }
    });
}
