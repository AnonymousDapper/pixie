// MIT License

// Copyright (c) 2022 AnonmousDapper

#![allow(non_upper_case_globals)]

use serde::Deserialize;

use crate::{
    canvas::{self, Canvas, Render, RgbaF16},
    FB_HEIGHT, FB_WIDTH,
};

// Physics constants

/// Gravitational constant (m³/s²/kg)
const G: f32 = 6.67384e-11;

/// Speed of light in vacuum (m/s)
const c: f32 = 2.997_924_5e8;

/// Charge on electron (C)
const ε: f32 = 1.602_176_6e-19;

/// Planck's constant (J/s)
const h: f32 = 6.626_07e-34;

/// Pi
const π: f32 = std::f32::consts::PI;

/// Boltzmann constant (J·K¯¹)
const k: f32 = 1.380_649e-23;

/// Stefan-Boltzmann constant (W/m²/K⁴)
const σ: f32 = 5.670367e-8;

// Simulator types

pub type Point = glam::Vec2;

pub type Velocity = glam::Vec2;

pub type Color = (u8, u8, u8);

fn default_color() -> Color {
    (255, 255, 255)
}

fn default_temp() -> f32 {
    288.0 // 188K ≈ 60°F
}

#[derive(Debug, Deserialize)]
pub struct Particle {
    #[serde(default)]
    name: String,

    #[serde(alias = "pos")]
    position: Point,

    #[serde(alias = "vel")]
    #[serde(default)]
    velocity: Velocity,

    #[serde(default = "default_color")]
    #[serde(alias = "colour")]
    color: Color,

    #[serde(default = "default_temp")]
    #[serde(alias = "temp")]
    temperature: f32,

    // these 3 aren't allowed to be 0, so we'll use that as the sentinel
    #[serde(default)]
    size: f32,

    #[serde(default)]
    mass: f32,

    #[serde(default)]
    density: f32,
}

impl Particle {
    pub fn new(
        position: Point,
        velocity: Velocity,
        color: Color,
        temperature: f32,
        size: f32,
        mass: f32,
        density: f32,
    ) -> Self {
        let mut part = Self {
            name: String::new(),
            position,
            velocity,
            color,
            temperature,
            size,
            mass,
            density,
        };

        part.resolve_properties();

        part
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn resolve_properties(&mut self) {
        self.temperature = if self.temperature == 0.0 {
            default_temp()
        } else {
            self.temperature
        };

        let size = if self.size == 0.0 { 3.0 } else { self.size };
        let density = if self.density == 0.0 {
            50.0
        } else {
            self.mass = size * self.density; // overwrite mass value if density is defined
            self.density
        };

        let mass = if self.mass == 0.0 {
            size * density
        } else {
            self.mass
        };

        self.size = size;
        self.density = density;
        self.mass = mass;
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn position_mut(&mut self) -> &mut Point {
        &mut self.position
    }

    pub fn wrap_position(&mut self) {
        self.position.x = self.position.x.rem_euclid(FB_WIDTH as f32);
        self.position.y = self.position.y.rem_euclid(FB_HEIGHT as f32);
    }

    pub fn normalize_position(&self) -> canvas::Point {
        let x = self.position.x.round() as canvas::Size;
        let y = self.position.y.round() as canvas::Size;

        (x.clamp(0, FB_WIDTH - 1), y.clamp(0, FB_HEIGHT - 1))
    }

    pub fn velocity(&self) -> Velocity {
        self.velocity
    }

    pub fn velocity_mut(&mut self) -> &mut Velocity {
        &mut self.velocity
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn render_color(&self) -> RgbaF16 {
        let l = (((4.0 * π * σ) * (self.size * 50.0).powi(2) * self.temperature.powi(4))
            / 1000000.0)
            .log(300.0);

        RgbaF16::new(
            l * self.color.0 as f32 / 255.0,
            l * self.color.1 as f32 / 255.0,
            l * self.color.2 as f32 / 255.0,
        )
    }

    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn normalize_size(&self) -> canvas::Size {
        self.size.round() as canvas::Size
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn density(&self) -> f32 {
        self.density
    }
}

// this is the actual particle interaction physics
fn handle_interaction(a: &mut Particle, b: &mut Particle, scale: f32) {
    let mass_a = a.mass();
    let pos_a = a.position();

    let mass_b = b.mass();
    let pos_b = b.position();

    let distance = pos_a.distance(pos_b);

    if distance < 1. {
        println!("Collision! {} - {}", a.name(), b.name());
    }

    let a_to_b = (pos_b - pos_a) / distance;

    let distance2 = pos_a.distance_squared(pos_b);

    let force = (scale * G * mass_a * mass_b) / distance2;

    let force_a = force / mass_a;
    let _force_b_rel = force_a * (mass_a / mass_b);

    let accel_a = a_to_b * force_a;

    *a.velocity_mut() += accel_a;

    let vel_a = a.velocity();

    *a.position_mut() += vel_a;

    a.wrap_position();
}

pub struct Simulator {
    environment: Vec<Particle>,
    physics_scale: f32,
}

impl Simulator {
    pub fn new() -> Self {
        Self::new_with_scale(1.0)
    }

    pub fn new_with_scale(physics_scale: f32) -> Self {
        Self {
            environment: Vec::new(),
            physics_scale,
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.physics_scale = scale;
    }

    pub fn physics_scale(&self) -> f32 {
        self.physics_scale
    }

    pub fn add_particle(&mut self, particle: Particle) -> usize {
        let idx = self.environment.len();
        self.environment.push(particle);

        idx
    }

    pub fn remove_particle(&mut self, idx: usize) {
        self.environment.remove(idx);
    }

    pub fn step_physics(&mut self) {
        let len = self.environment.len();
        for idx in 0..len {
            let (head, tail) = self.environment.split_at_mut(idx);

            let (current, rest) = tail.split_first_mut().unwrap();

            for other in head {
                handle_interaction(current, other, self.physics_scale);
            }

            for other in rest {
                handle_interaction(current, other, self.physics_scale);
            }
        }
    }

    pub fn render(&self, canvas: &mut Canvas) {
        for particle in &self.environment {
            let coords = particle.normalize_position();
            let color = particle.render_color();
            canvas.particle(coords, particle.normalize_size(), color);
        }
    }
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new()
    }
}
