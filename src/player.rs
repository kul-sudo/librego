use crate::bullet::Bullet;
use macroquad::prelude::*;
use std::{collections::HashMap, time::Instant};

#[derive(Default)]
pub struct Player {
    pub crouched: bool,
    pub walking: bool,
    pub jump: Option<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub front: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub position: Vec3,
    pub bullets: HashMap<Instant, Bullet>,
    pub last_bullet_timestamp: Option<Instant>,
}
