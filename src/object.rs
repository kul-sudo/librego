use macroquad::prelude::*;
use parry3d_f64::shape::Compound;

#[derive(Clone)]
pub struct Cube {
    pub pos: Vec3,
    pub size: Vec3,
}

#[derive(Clone)]
pub enum Object {
    Compound(Compound),
}
