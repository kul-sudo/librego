use crate::bullet::Bullet;
use macroquad::prelude::*;
use std::{collections::HashMap, time::Instant};

#[derive(Default)]
pub struct Player {
    pub crouched: bool,
    pub walking: bool,
    pub jump: Option<f64>,
    pub yaw: f64,
    pub pitch: f64,
    pub front: DVec3,
    pub right: DVec3,
    pub up: DVec3,
    pub position: DVec3,
    pub bullets: HashMap<Instant, Bullet>,
    pub last_bullet_timestamp: Option<Instant>,
    pub last_move_timestamp: Option<Instant>,
    pub bullets_since_last_reload: u8,
    pub last_reload_timestamp: Option<Instant>,
}
