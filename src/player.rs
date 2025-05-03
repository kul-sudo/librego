use crate::bullet::Bullet;
use macroquad::prelude::*;
use std::{collections::HashMap, time::Instant};

#[derive(Default)]
pub enum JumpType {
    #[default]
    Full,
    Fall,
}

#[derive(Default)]
pub struct Player {
    pub crouched: bool,
    pub walking: bool,
    pub jump: Option<(f32, JumpType)>,
    pub yaw: f32,
    pub pitch: f32,
    pub front: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub position: Vec3,
    pub bullets: HashMap<Instant, Bullet>,
    pub last_bullet_timestamp: Option<Instant>,
    pub last_move_timestamp: Option<Instant>,
    pub bullets_since_last_reload: u8,
    pub last_reload_timestamp: Option<Instant>,
}
