use macroquad::prelude::*;
use std::time::Instant;

pub struct Bullet {
    pub position: DVec3,
    pub front: DVec3,
    pub born: Instant,
}
