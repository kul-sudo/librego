use macroquad::prelude::*;
use parry3d_f64::math::Vector;
use std::{
    f64::consts::{FRAC_PI_2, PI},
    time::Duration,
};

pub const DEFAULT_SCREEN_SIZE: Vec2 = vec2(1920.0, 1080.0);
pub const FOV: f32 = std::f32::consts::FRAC_PI_2;

pub const PITCH_BOUND: f64 = FRAC_PI_2 * 0.999;

pub const MOVE_SPEED: f64 = 0.05;
pub const LOOK_SPEED: f64 = 0.05;

pub const JUMP_VELOCITY: f64 = 0.06;
pub const GRAVITY: f64 = 0.0035;

pub const CROUCH_SPEED_CONST: f64 = 0.3;
pub const WALKING_SPEED_CONST: f64 = 0.5;

pub const CAMERA_Y: f64 = 1.0;
pub const PLAYER_SIZE: Vector<f64> = Vector::new(0.1 / 2.0, CAMERA_Y / 2.0, 0.1 / 2.0);
pub const CROUCH_LEVEL_CONST: f64 = 0.3 * CAMERA_Y;

pub const CROSSHAIR_LINE_LENGTH: f32 = 8.0;
pub const CROSSHAIR_THICKNESS: f32 = 3.0;
pub const CROSSHAIR_COLOR: Color = DARKGREEN;

pub const BULLET_RADIUS: f32 = 0.01;
pub const BULLET_COLOR: Color = YELLOW;
pub static BULLET_INTERVAL: Duration = Duration::from_millis(100);
pub static BULLET_SPREAD: f64 = PI / 10.0;
pub static BULLET_SPREAD_PERIOD: Duration = Duration::from_secs(10);
pub const BULLETS_BEFORE_RELOAD: u8 = 30;
pub const RELOAD_DURATION: Duration = Duration::from_secs(2);
pub const BULLETS_FONT_SIZE: u16 = 35;

pub const SIZE: f32 = 5.0;
pub const COLUMNS: usize = 10;
pub const HALF: f64 = COLUMNS as f64 * SIZE as f64 / 2.0;
