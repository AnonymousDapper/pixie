// MIT License

// Copyright (c) 2022 AnonmousDapper

#![deny(rust_2018_idioms)]
#![allow(mixed_script_confusables)]

pub const FB_HEIGHT: i32 = 256;
pub const FB_WIDTH: i32 = 256;

pub const WINDOW_WIDTH: u32 = 700;
pub const WINDOW_HEIGHT: u32 = 700;

pub const PIPELINE_TEXTURE_FORMAT: pixels::wgpu::TextureFormat =
    pixels::wgpu::TextureFormat::Rgba16Float;

pub mod canvas;

pub mod window;

pub mod pipeline;

pub mod simulation;
