use macroquad::prelude::*;
use parry3d::shape::Compound;

#[derive(Clone)]
pub struct Cube {
    pub pos: Vec3,
    pub size: Vec3,
}

#[derive(Clone)]
pub enum Object {
    Compound(Compound),
}
