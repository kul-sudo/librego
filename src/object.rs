use crate::consts::COLLISION_GAP;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct Cube {
    pub pos: Vec3,
    pub size: Vec3,
}

#[derive(Clone, Copy)]
pub enum Object {
    Cube(Cube),
}
