use crate::consts::COLLISION_GAP;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct Cube {
    pub pos: Vec3,
    pub size: Vec3,
}

impl Cube {
    pub fn adjust_if_contains(&self, mut position: Vec3) -> (Vec3, bool) {
        let mut contains = false;
        let size_x_half = self.size.x / 2.0;
        let size_z_half = self.size.z / 2.0;

        if (self.pos.x - size_x_half - COLLISION_GAP..=self.pos.x + size_x_half + COLLISION_GAP)
            .contains(&position.x)
            && (self.pos.z - size_z_half - COLLISION_GAP..=self.pos.z + size_z_half + COLLISION_GAP)
                .contains(&position.z)
        {
            contains = true;

            let a = position.distance(self.pos.with_x(self.pos.x - size_x_half));
            let b = position.distance(self.pos.with_x(self.pos.x + size_x_half));
            let c = position.distance(self.pos.with_z(self.pos.z - size_z_half));
            let d = position.distance(self.pos.with_z(self.pos.z + size_z_half));

            if a < b && a < c && a < d {
                position.x = self.pos.x - size_x_half - COLLISION_GAP;
            } else if b < a && b < c && b < d {
                position.x = self.pos.x + size_x_half + COLLISION_GAP;
            } else if c < a && c < b && c < d {
                position.z = self.pos.z - size_z_half - COLLISION_GAP;
            } else {
                position.z = self.pos.z + size_z_half + COLLISION_GAP;
            }
        }

        (position, contains)
    }
}

#[derive(Clone, Copy)]
pub enum Object {
    Cube(Cube),
}
