mod bullet;
mod consts;
mod object;
mod player;

use ::rand::{Rng, SeedableRng, rngs::StdRng};
use bullet::Bullet;
use consts::*;
use macroquad::{
    audio::{load_sound, play_sound_once},
    prelude::*,
};
use parry3d_f64::{
    bounding_volume::BoundingVolume,
    math::{Isometry, Point, Vector},
    query,
    shape::{Ball, Compound, Cuboid, SharedShape},
};
use player::Player;
use std::{collections::HashMap, time::Instant};

fn window_conf() -> Conf {
    Conf {
        window_title: "librego".to_owned(),
        fullscreen: true,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    for _ in 0..8 {
        set_fullscreen(true);
        next_frame().await;
    }

    let mut rng = StdRng::from_os_rng();

    let compounds = [
        (
            dvec3(5.0, 0.1, 1.0),
            Cuboid::new(Vector::new(1.0, 0.1, 1.0)),
        ),
        (
            dvec3(5.0, 1.8, 1.0),
            Cuboid::new(Vector::new(1.0, 0.1, 1.0)),
        ),
        (
            dvec3(HALF, 0.0, 0.0),
            Cuboid::new(Vector::new(1.0, 20.0, HALF)),
        ),
        (
            dvec3(0.0, 0.0, HALF),
            Cuboid::new(Vector::new(HALF, 20.0, 1.0)),
        ),
        (
            dvec3(0.0, 0.0, -HALF),
            Cuboid::new(Vector::new(HALF, 20.0, 1.0)),
        ),
        (
            dvec3(-HALF, 0.0, 0.0),
            Cuboid::new(Vector::new(1.0, 20.0, HALF)),
        ),
    ];

    let compound = Compound::new(
        compounds
            .map(|(position, shape)| {
                (
                    Isometry::translation(position.x, position.y, position.z),
                    SharedShape::new(shape),
                )
            })
            .to_vec(),
    );

    let world_up = dvec3(0.0, 1.0, 0.0);

    let mut player = Player::default();
    player.crouched = false;
    player.walking = false;
    player.jump = None;
    player.yaw = 0.0;
    player.pitch = 0.0;
    player.front = dvec3(
        player.yaw.cos() * player.pitch.cos(),
        player.pitch.sin(),
        player.yaw.sin() * player.pitch.cos(),
    )
    .normalize();
    player.right = player.front.cross(world_up).normalize();
    player.up = player.right.cross(player.front).normalize();
    player.position = dvec3(0.0, PLAYER_SIZE.y, 0.0);
    player.bullets = HashMap::new();
    player.last_bullet_timestamp = None;
    player.last_move_timestamp = None;

    let mut last_mouse_position: DVec2 = Vec2::from(mouse_position()).as_dvec2();

    let mut grabbed = true;
    set_cursor_grab(grabbed);
    show_mouse(false);

    set_fullscreen(true);

    let screen_size = vec2(screen_width(), screen_height());

    set_pc_assets_folder("assets");
    let sound = load_sound("bullet.ogg").await.unwrap();

    loop {
        let delta = get_frame_time() as f64;
        let mut moved = false;

        if is_key_pressed(KeyCode::Tab) {
            grabbed = !grabbed;
            set_cursor_grab(grabbed);
            show_mouse(!grabbed);
        }

        if is_key_pressed(KeyCode::LeftShift) {
            player.walking = !player.walking;
        }

        let just_jumped =
            is_key_pressed(KeyCode::Space) && player.jump.is_none() && !player.crouched;
        if just_jumped {
            player.jump = Some(-JUMP_VELOCITY);
            if player.last_move_timestamp.is_none() {
                player.last_move_timestamp = Some(Instant::now());
            }
        }

        let player_cuboid = Cuboid::new(PLAYER_SIZE);

        let move_speed = MOVE_SPEED
            * (if player.crouched {
                CROUCH_SPEED_CONST
            } else if player.walking {
                WALKING_SPEED_CONST
            } else {
                1.0
            });

        let mut pos_delta = DVec3::ZERO;
        if is_key_down(KeyCode::W) {
            pos_delta += player.front;
            moved = true;
        }
        if is_key_down(KeyCode::S) {
            pos_delta -= player.front;
            moved = true;
        }
        if is_key_down(KeyCode::A) {
            pos_delta -= player.right;
            moved = true;
        }
        if is_key_down(KeyCode::D) {
            pos_delta += player.right;
            moved = true;
        }

        if is_key_pressed(KeyCode::R) {
            player.bullets_since_last_reload = 0;
            player.last_reload_timestamp = Some(Instant::now());
        }

        if moved && player.last_move_timestamp.is_none() {
            player.last_move_timestamp = Some(Instant::now());
        } else if !moved && player.jump.is_none() {
            player.last_move_timestamp = None;
        }

        if pos_delta.length() > 0.0 {
            pos_delta = pos_delta.normalize();
        }

        let position = player.position + pos_delta * move_speed;

        let current_pos = player.position;

        let contact = query::contact(
            &Isometry::identity(),
            &compound,
            &Isometry::translation(player.position.x, current_pos.y, player.position.z),
            &player_cuboid,
            0.0,
        )
        .unwrap();
        let mut y_intersection =
            !just_jumped && contact.is_some_and(|contact| player.position.y > contact.point2.y);

        if player.position.y > PLAYER_SIZE.y && !y_intersection && player.jump.is_none() {
            player.jump = Some(0.0);
            if player.last_move_timestamp.is_none() {
                player.last_move_timestamp = Some(Instant::now());
            }
        }

        match &mut player.jump {
            Some(jump) => {
                if y_intersection {
                    player.position.y = contact.unwrap().point1.y as f64 + PLAYER_SIZE.y;
                    player.jump = None;
                } else if !just_jumped && player.position.y <= PLAYER_SIZE.y && contact.is_none() {
                    player.position.y = PLAYER_SIZE.y;
                    player.jump = None;
                } else {
                    if let Some(contact) = contact {
                        if player.position.y <= contact.point2.y {
                            *jump = 0.0;
                            player.position.y = (contact.point1.y - PLAYER_SIZE.y) * 0.99999;
                            y_intersection = false;
                        }
                    }
                    player.position.y -= *jump;
                    *jump += GRAVITY;
                }
            }
            None => {
                player.front.y = 0.0;
                player.front = player.front.normalize();
            }
        }

        let x_intersection = query::contact(
            &Isometry::identity(),
            &compound,
            &Isometry::translation(position.x, current_pos.y, current_pos.z),
            &player_cuboid,
            0.0,
        )
        .unwrap()
        .is_some();
        let z_intersection = !x_intersection
            && query::contact(
                &Isometry::identity(),
                &compound,
                &Isometry::translation(current_pos.x, current_pos.y, position.z),
                &player_cuboid,
                0.0,
            )
            .unwrap()
            .is_some();

        if y_intersection {
            player.position.x = position.x;
            player.position.z = position.z;
        } else {
            if !x_intersection {
                player.position.x = position.x
            }
            if !z_intersection {
                player.position.z = position.z
            };
        }

        let mouse_position: DVec2 = Vec2::from(mouse_position()).as_dvec2();
        let mouse_delta = mouse_position - last_mouse_position;

        last_mouse_position = mouse_position;

        if grabbed {
            player.yaw += mouse_delta.x * delta * LOOK_SPEED;
            player.pitch += mouse_delta.y * delta * -LOOK_SPEED;
            player.pitch = player.pitch.clamp(-PITCH_BOUND, PITCH_BOUND);
            player.front = dvec3(
                player.yaw.cos() * player.pitch.cos(),
                player.pitch.sin(),
                player.yaw.sin() * player.pitch.cos(),
            )
            .normalize();

            player.right = player.front.cross(world_up).normalize();
            player.up = player.right.cross(player.front).normalize();
        }

        if is_mouse_button_down(MouseButton::Left)
            && player.bullets_since_last_reload < BULLETS_BEFORE_RELOAD
            && if let Some(last_reload_timestamp) = player.last_reload_timestamp {
                last_reload_timestamp.elapsed() > RELOAD_DURATION
            } else {
                true
            }
            && if let Some(last_bullet_timestamp) = player.last_bullet_timestamp {
                last_bullet_timestamp.elapsed() > BULLET_INTERVAL
            } else {
                true
            }
        {
            player.bullets_since_last_reload += 1;

            let inaccurate = !player.crouched && (player.jump.is_some() || moved);
            let now = Instant::now();
            player.last_bullet_timestamp = Some(now);

            let spread_level = match player.last_move_timestamp {
                Some(timestamp) => {
                    timestamp.elapsed().as_nanos() as f64 / BULLET_SPREAD_PERIOD.as_nanos() as f64
                }
                None => 0.0,
            }
            .min(1.0);

            let mut motion = query::NonlinearRigidMotion::new(
                Isometry::translation(player.position.x, player.position.y, player.position.z),
                Point::new(player.position.x, player.position.y, player.position.z),
                Vector::new(player.front.x, player.front.y, player.front.z) * BULLET_STEP,
                Vector::new(0.0, 0.0, 0.0),
            );

            let cast = query::cast_shapes_nonlinear(
                &motion,
                &Ball::new(BULLET_RADIUS as f64),
                &query::NonlinearRigidMotion::constant_position(Isometry::identity()),
                &compound,
                f64::NEG_INFINITY,
                f64::INFINITY,
                true,
            );

            if let Some(cast) = cast.unwrap() {
                player.bullets.insert(
                    now,
                    Bullet {
                        motion,
                        born: Instant::now(),
                    },
                );
            }

            play_sound_once(&sound);
        }

        clear_background(BLACK);
        // set_camera(&Camera3D {
        //     position: vec3(5.0, 0.5, 5.0),
        //     up: player.up.as_vec3(),
        //     target: player.position.as_vec3(),
        //     fovy: FOV,
        //     ..Default::default()
        // });

        set_camera(&Camera3D {
            position: player
                .position
                // .with_x(player.position.x + PLAYER_SIZE.x)
                .as_vec3(),
            up: player.up.as_vec3(),
            target: player.position.as_vec3() + player.front.as_vec3(),
            fovy: FOV,
            ..Default::default()
        });

        player.bullets.retain(|_, bullet| {
            let maybe_cast = query::cast_shapes_nonlinear(
                &bullet.motion,
                &Ball::new(BULLET_RADIUS as f64),
                &query::NonlinearRigidMotion::constant_position(Isometry::identity()),
                &compound,
                0.0,
                f64::INFINITY,
                true,
            )
            .unwrap();

            !maybe_cast.is_some_and(|cast| {
                matches!(
                    cast.status,
                    query::ShapeCastStatus::PenetratingOrWithinTargetDist
                )
            })
        });

        for (started, bullet) in &mut player.bullets {
            draw_sphere(
                vec3(
                    bullet.motion.start.translation.x as f32,
                    bullet.motion.start.translation.y as f32,
                    bullet.motion.start.translation.z as f32,
                ),
                BULLET_RADIUS,
                None,
                BULLET_COLOR,
            );

            bullet.motion = bullet.motion.prepend_translation(bullet.motion.linvel);
        }

        // draw_cube(
        //     player.position.as_vec3(),
        //     DVec3::from_slice(PLAYER_SIZE.as_slice()).as_vec3() * 2.0,
        //     None,
        //     RED,
        // );

        for (isometry, shape) in compound.shapes() {
            let pos = isometry.translation.vector;
            let size = shape.as_cuboid().unwrap().half_extents;
            draw_cube(
                DVec3::from_slice(pos.as_slice()).as_vec3(),
                DVec3::from_slice(size.as_slice()).as_vec3() * 2.0,
                None,
                BLACK,
            );
            draw_cube_wires(
                DVec3::from_slice(pos.as_slice()).as_vec3(),
                DVec3::from_slice(size.as_slice()).as_vec3() * 2.0,
                WHITE,
            );
        }

        draw_cube(
            vec3(0.0, 0.0, 0.0),
            vec3((COLUMNS as f32) * SIZE, 0.0, (COLUMNS as f32) * SIZE),
            None,
            GRAY,
        );

        // draw_cube(
        //     player.position.as_vec3(),
        //     (DVec3::from_slice(PLAYER_SIZE.as_slice()) * 2.0).as_vec3(),
        //     None,
        //     GRAY,
        // );

        set_default_camera();

        draw_line(
            screen_size.x / 2.0 - CROSSHAIR_LINE_LENGTH,
            screen_size.y / 2.0,
            screen_size.x / 2.0 + CROSSHAIR_LINE_LENGTH,
            screen_size.y / 2.0,
            CROSSHAIR_THICKNESS,
            CROSSHAIR_COLOR,
        );
        draw_line(
            screen_size.x / 2.0,
            screen_size.y / 2.0 - CROSSHAIR_LINE_LENGTH,
            screen_size.x / 2.0,
            screen_size.y / 2.0 + CROSSHAIR_LINE_LENGTH,
            CROSSHAIR_THICKNESS,
            CROSSHAIR_COLOR,
        );

        let bullets_text = format!(
            "{}/{}",
            BULLETS_BEFORE_RELOAD - player.bullets_since_last_reload,
            BULLETS_BEFORE_RELOAD
        );
        let bullets_text_measured = measure_text(
            &bullets_text,
            None,
            (BULLETS_FONT_SIZE as f32 * (screen_size.x * screen_size.y)
                / (DEFAULT_SCREEN_SIZE.x * DEFAULT_SCREEN_SIZE.y)) as u16,
            1.0,
        );
        draw_text(
            &bullets_text,
            screen_size.x - bullets_text_measured.width,
            bullets_text_measured.height,
            BULLETS_FONT_SIZE as f32,
            WHITE,
        );

        next_frame().await
    }
}
