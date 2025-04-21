use macroquad::prelude::*;
use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI},
    time::Duration,
};

pub const DEFAULT_FOV: f32 = FRAC_PI_8;
pub const FOV: f32 = FRAC_PI_4;

pub const PITCH_BOUND: f32 = FRAC_PI_2 * 0.999;

pub const GRAVITY: f32 = 0.005;

pub const MOVE_SPEED: f32 = 0.05;
pub const LOOK_SPEED: f32 = 0.05;

pub const JUMP_VELOCITY: f32 = 0.1;

pub const CAMERA_Y: f32 = 1.0;
pub const CROUCH_SPEED_CONST: f32 = 0.3;
pub const WALKING_SPEED_CONST: f32 = 0.5;

pub const CROUCH_LEVEL_CONST: f32 = 0.3;

pub const COLLISION_GAP: f32 = 0.1;

pub const CROSSHAIR_LINE_LENGTH: f32 = 8.0;
pub const CROSSHAIR_THICKNESS: f32 = 3.0;
pub const CROSSHAIR_COLOR: Color = DARKGREEN;

pub const BULLET_RADIUS: f32 = 0.05;
pub const BULLET_COLOR: Color = YELLOW;
pub const BULLET_STEP: f32 = 1.0 * FOV / DEFAULT_FOV;
pub static BULLET_INTERVAL: Duration = Duration::from_millis(100);
pub static BULLET_SPREAD: f32 = PI / 10.0;
pub static BULLET_SPREAD_PERIOD: Duration = Duration::from_secs(6);
pub static BULLET_LIFETIME: Duration =
    Duration::from_secs((COLUMNS as f32 * SIZE / BULLET_STEP) as u64);

pub const SIZE: f32 = 5.0;
pub const COLUMNS: usize = 10;
pub const HALF: f32 = COLUMNS as f32 * SIZE / 2.0;
