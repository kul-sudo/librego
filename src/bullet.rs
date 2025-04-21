use macroquad::prelude::*;
use std::time::Instant;

pub struct Bullet {
    pub position: Vec3,
    pub front: Vec3,
    pub born: Instant,
}
