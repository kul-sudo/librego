use crate::consts::*;
use ::rand::{Rng, rngs::StdRng};
use macroquad::{
    audio::{Sound, play_sound_once},
    prelude::*,
};
use parry3d_f64::{
    math::{Isometry, Point, Vector},
    query::{Ray, RayCast, contact},
    shape::{Compound, Cuboid},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Instant,
};

#[derive(Clone)]
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
    pub last_bullet_timestamp: Option<Instant>,
    pub last_move_timestamp: Option<Instant>,
    pub bullets_since_last_reload: u8,
    pub last_reload_timestamp: Option<Instant>,
    pub mouse_position: DVec2,
    pub ticks: Vec<Option<DVec3>>,
    pub last_tick_timestamp: Instant,
    pub killed: bool,
}

impl Player {
    pub fn new(position: DVec3) -> Self {
        let yaw: f64 = 0.0;
        let pitch: f64 = 0.0;
        let front = dvec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
        .normalize();

        let right = front.cross(WORLD_UP).normalize();

        Self {
            crouched: false,
            walking: false,
            jump: None,
            yaw,
            pitch,
            front,
            right,
            up: right.cross(front).normalize(),
            position,
            last_bullet_timestamp: None,
            last_move_timestamp: None,
            bullets_since_last_reload: 0,
            last_reload_timestamp: None,
            mouse_position: DVec2::ZERO,
            ticks: Vec::with_capacity(TICKS_PER_SECOND as usize),
            last_tick_timestamp: Instant::now(),
            killed: false,
        }
    }

    pub fn movement(&mut self, compound: &Compound) -> bool {
        let mut moved = false;

        // Walking toggle
        if is_key_pressed(KeyCode::LeftShift) {
            self.walking = !self.walking;
        }

        // Space
        let just_jumped = is_key_pressed(KeyCode::Space) && !self.crouched;
        if just_jumped && self.jump.is_none() {
            self.jump = Some(-JUMP_VELOCITY);
            if self.last_move_timestamp.is_none() {
                self.last_move_timestamp = Some(Instant::now());
            }
        }

        let self_cuboid = Cuboid::new(PLAYER_SIZE);

        self.front.y = 0.0;
        self.front = self.front.normalize();

        // Movement
        let move_speed = MOVE_SPEED
            * (if self.crouched {
                CROUCH_SPEED_CONST
            } else if self.walking {
                WALKING_SPEED_CONST
            } else {
                1.0
            });

        let mut pos_delta = DVec3::ZERO;
        if is_key_down(KeyCode::W) {
            pos_delta += self.front;
            moved = true;
        }
        if is_key_down(KeyCode::S) {
            pos_delta -= self.front;
            moved = true;
        }
        if is_key_down(KeyCode::A) {
            pos_delta -= self.right;
            moved = true;
        }
        if is_key_down(KeyCode::D) {
            pos_delta += self.right;
            moved = true;
        }

        if pos_delta.length() > 0.0 {
            pos_delta = pos_delta.normalize();
        }

        let position = self.position + pos_delta * move_speed;

        // Collisions
        let maybe_contact = contact(
            &Isometry::identity(),
            compound,
            &Isometry::translation(self.position.x, self.position.y, self.position.z),
            &self_cuboid,
            0.0,
        )
        .unwrap();
        let mut y_intersection =
            !just_jumped && maybe_contact.is_some_and(|contact| self.position.y > contact.point2.y);

        if self.position.y > PLAYER_SIZE.y && !y_intersection && self.jump.is_none() {
            self.jump = Some(0.0);
            if self.last_move_timestamp.is_none() {
                self.last_move_timestamp = Some(Instant::now());
            }
        }

        if let Some(jump) = &mut self.jump {
            if y_intersection {
                self.position.y = maybe_contact.unwrap().point1.y as f64 + PLAYER_SIZE.y;
                self.jump = None;
            } else if !just_jumped && self.position.y <= PLAYER_SIZE.y && maybe_contact.is_none() {
                self.position.y = PLAYER_SIZE.y;
                self.jump = None;
            } else {
                if let Some(contact) = maybe_contact {
                    if self.position.y <= contact.point2.y {
                        *jump = 0.0;
                        self.position.y = (contact.point1.y - PLAYER_SIZE.y) * 0.99999;
                        y_intersection = false;
                    }
                }
                self.position.y -= *jump;
                *jump += GRAVITY;
            }
        }

        let x_intersection = contact(
            &Isometry::identity(),
            compound,
            &Isometry::translation(position.x, self.position.y, self.position.z),
            &self_cuboid,
            0.0,
        )
        .unwrap()
        .is_some();

        let z_intersection = !x_intersection
            && contact(
                &Isometry::identity(),
                compound,
                &Isometry::translation(self.position.x, self.position.y, position.z),
                &self_cuboid,
                0.0,
            )
            .unwrap()
            .is_some();

        if y_intersection {
            self.position.x = position.x;
            self.position.z = position.z;
        } else {
            if !x_intersection {
                self.position.x = position.x
            }
            if !z_intersection {
                self.position.z = position.z
            };
        }

        // Reload
        if is_key_pressed(KeyCode::R) {
            self.bullets_since_last_reload = 0;
            self.last_reload_timestamp = Some(Instant::now());
        }

        if moved && self.last_move_timestamp.is_none() {
            self.last_move_timestamp = Some(Instant::now());
        } else if !moved && self.jump.is_none() {
            self.last_move_timestamp = None;
        }

        moved
    }

    pub fn look(&mut self, delta: f64) {
        let mouse_position: DVec2 = Vec2::from(mouse_position()).as_dvec2();
        let mouse_delta = mouse_position - self.mouse_position;

        self.mouse_position = mouse_position;

        self.yaw += mouse_delta.x * delta * LOOK_SPEED;
        self.pitch += mouse_delta.y * delta * -LOOK_SPEED;
        self.pitch = self.pitch.clamp(-PITCH_BOUND, PITCH_BOUND);
        self.front = dvec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        self.right = self.front.cross(WORLD_UP).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    pub fn bullets(
        &mut self,
        peers: Arc<RwLock<HashMap<SocketAddr, Player>>>,
        bullet_sound: &Sound,
        moved: bool,
        rng: &mut StdRng,
    ) {
        if is_mouse_button_down(MouseButton::Left)
            && self.bullets_since_last_reload < BULLETS_BEFORE_RELOAD
            && self
                .last_reload_timestamp
                .is_none_or(|last_reload_timestamp| {
                    last_reload_timestamp.elapsed() > RELOAD_DURATION
                })
            && self
                .last_bullet_timestamp
                .is_none_or(|last_bullet_timestamp| {
                    last_bullet_timestamp.elapsed() > BULLET_INTERVAL
                })
        {
            self.bullets_since_last_reload += 1;

            let inaccurate = !self.crouched && (self.jump.is_some() || moved);
            let now = Instant::now();
            self.last_bullet_timestamp = Some(now);

            let spread_level = match self.last_move_timestamp {
                Some(timestamp) => {
                    timestamp.elapsed().as_nanos() as f64 / BULLET_SPREAD_PERIOD.as_nanos() as f64
                }
                None => 0.0,
            }
            .min(1.0);

            let ray = Ray::new(
                Point::new(self.position.x, self.position.y, self.position.z),
                Vector::new(
                    self.front.x
                        + inaccurate as usize as f64
                            * rng.random_range(-BULLET_SPREAD..BULLET_SPREAD)
                            * spread_level,
                    self.front.y
                        + inaccurate as usize as f64
                            * rng.random_range(-BULLET_SPREAD..BULLET_SPREAD)
                            * spread_level,
                    self.front.z
                        + inaccurate as usize as f64
                            * rng.random_range(-BULLET_SPREAD..BULLET_SPREAD)
                            * spread_level,
                ),
            );

            let mut peers_write = peers.write().unwrap();

            for peer in peers_write.values_mut() {
                for tick in &peer.ticks {
                    if let Some(position) = tick {
                        if Cuboid::new(PLAYER_SIZE * 2.0)
                            .cast_ray(
                                &Isometry::translation(position.x, position.y, position.z),
                                &ray,
                                f64::INFINITY,
                                true,
                            )
                            .is_some()
                        {
                            peer.killed = true;
                        }
                    }
                }
            }

            play_sound_once(bullet_sound);
        }
    }
}
