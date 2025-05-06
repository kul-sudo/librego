use macroquad::prelude::*;
use parry3d_f64::query::{NonlinearRigidMotion, Ray, ShapeCastHit};
use std::time::Instant;

pub struct Bullet {
    pub ray: Ray,
    pub born: Instant,
}
