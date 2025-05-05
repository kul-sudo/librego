use macroquad::prelude::*;
use parry3d_f64::query::{NonlinearRigidMotion, ShapeCastHit};
use std::time::Instant;

pub struct Bullet {
    pub motion: NonlinearRigidMotion,
    pub born: Instant,
}
