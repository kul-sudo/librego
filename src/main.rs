mod bullet;
mod consts;
mod object;
mod player;

use ::rand::{Rng, SeedableRng, rngs::StdRng};
use bullet::Bullet;
use consts::*;
use macroquad::prelude::*;
use object::{Cube, Object};
use player::Player;
use std::{collections::HashMap, time::Instant};

fn conf() -> Conf {
    Conf {
        window_title: String::from("Macroquad"),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut rng = StdRng::from_os_rng();

    let objects = Vec::from([
        Object::Cube(Cube {
            pos: vec3(0.0, 0.0, 0.0),
            size: vec3(2.0, 5.0, 2.0),
        }),
        Object::Cube(Cube {
            pos: vec3(5.0, 0.0, 5.0),
            size: vec3(2.0, 5.0, 2.0),
        }),
    ]);

    let world_up = vec3(0.0, 1.0, 0.0);

    let mut player = Player::default();
    player.crouched = false;
    player.walking = false;
    player.jump = None;
    player.yaw = 0.0;
    player.pitch = 0.0;
    player.front = vec3(
        player.yaw.cos() * player.pitch.cos(),
        player.pitch.sin(),
        player.yaw.sin() * player.pitch.cos(),
    )
    .normalize();
    player.right = player.front.cross(world_up).normalize();
    player.up = player.right.cross(player.front).normalize();
    player.position = vec3(0.0, CAMERA_Y, 0.0);
    player.bullets = HashMap::new();
    player.last_bullet_timestamp = None;

    let mut last_mouse_position: Vec2 = mouse_position().into();

    let mut grabbed = true;
    set_cursor_grab(grabbed);
    show_mouse(false);

    loop {
        let delta = get_frame_time();
        let mut moved = player.jump.is_some();

        if is_key_pressed(KeyCode::Tab) {
            grabbed = !grabbed;
            set_cursor_grab(grabbed);
            show_mouse(!grabbed);
        }

        if is_key_pressed(KeyCode::LeftShift) {
            player.walking = !player.walking;
        }

        if is_key_pressed(KeyCode::Space) && player.jump.is_none() && !player.crouched {
            moved = true;
            player.jump = Some(-JUMP_VELOCITY);
        }

        match &mut player.jump {
            Some(jump) => {
                player.position.y -= *jump;
                *jump += GRAVITY;
                if player.position.y <= CAMERA_Y {
                    player.position.y = CAMERA_Y;
                    player.jump = None;
                }
            }
            None => {
                player.crouched = is_key_down(KeyCode::C);

                player.position.y = CAMERA_Y - player.crouched as usize as f32 * CROUCH_LEVEL_CONST;
                player.front.y = 0.0;
                player.front = player.front.normalize();
            }
        }

        let move_speed = MOVE_SPEED
            * (if player.crouched {
                CROUCH_SPEED_CONST
            } else if player.walking {
                WALKING_SPEED_CONST
            } else {
                1.0
            });

        let mut pos_delta = Vec3::ZERO;
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

        if pos_delta.length() > 0.0 {
            pos_delta = pos_delta.normalize();
        }

        let position = player.position + pos_delta * move_speed;

        for object in &objects {
            match object {
                Object::Cube(cube) => {
                    let (adjustment, contains) = cube.adjust_if_contains(position);
                    player.position = adjustment;
                    if contains {
                        break;
                    }
                }
            }
        }

        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - last_mouse_position;

        last_mouse_position = mouse_position;

        if grabbed {
            player.yaw += mouse_delta.x * delta * LOOK_SPEED;
            player.pitch += mouse_delta.y * delta * -LOOK_SPEED;
            player.pitch = player.pitch.clamp(-PITCH_BOUND, PITCH_BOUND);
            player.front = vec3(
                player.yaw.cos() * player.pitch.cos(),
                player.pitch.sin(),
                player.yaw.sin() * player.pitch.cos(),
            )
            .normalize();

            player.right = player.front.cross(world_up).normalize();
            player.up = player.right.cross(player.front).normalize();
        }

        if is_mouse_button_down(MouseButton::Left)
            && if let Some(last_bullet_timestamp) = player.last_bullet_timestamp {
                last_bullet_timestamp.elapsed() > BULLET_INTERVAL
            } else {
                true
            }
        {
            let now = Instant::now();
            player.last_bullet_timestamp = Some(now);
            player.bullets.insert(
                now,
                Bullet {
                    position: player.position + player.front,
                    front: player.front
                        + moved as usize as f32 * rng.random_range(0.0..BULLET_SPREAD),
                },
            );
        }

        clear_background(LIGHTGRAY);

        set_camera(&Camera3D {
            position: player.position,
            up: player.up,
            target: player.position + player.front,
            fovy: FOV,
            ..Default::default()
        });

        for object in &objects {
            match object {
                Object::Cube(cube) => {
                    let Cube { pos, size } = cube;
                    draw_cube(*pos, *size, None, BLACK);
                    draw_cube_wires(*pos, *size, WHITE);
                }
            }
        }

        let mut removed_bullets = Vec::new();
        for (started, bullet) in &mut player.bullets {
            bullet.position += BULLET_STEP * bullet.front;
            draw_sphere(bullet.position, BULLET_RADIUS, None, BULLET_COLOR);

            for object in &objects {
                match object {
                    Object::Cube(cube) => {
                        if !(-HALF..HALF).contains(&bullet.position.x)
                            || !(-HALF..HALF).contains(&bullet.position.z)
                            || cube.adjust_if_contains(bullet.position).1
                        {
                            removed_bullets.push(*started)
                        }
                    }
                }
            }
        }

        for timestamp in &removed_bullets {
            player.bullets.remove(timestamp);
        }

        draw_cube(
            vec3(0.0, 0.0, 0.0),
            vec3((COLUMNS as f32) * SIZE, 0.0, (COLUMNS as f32) * SIZE),
            None,
            GRAY,
        );

        set_default_camera();
        draw_line(
            screen_width() / 2.0 - CROSSHAIR_LINE_LENGTH,
            screen_height() / 2.0,
            screen_width() / 2.0 + CROSSHAIR_LINE_LENGTH,
            screen_height() / 2.0,
            CROSSHAIR_THICKNESS,
            CROSSHAIR_COLOR,
        );
        draw_line(
            screen_width() / 2.0,
            screen_height() / 2.0 - CROSSHAIR_LINE_LENGTH,
            screen_width() / 2.0,
            screen_height() / 2.0 + CROSSHAIR_LINE_LENGTH,
            CROSSHAIR_THICKNESS,
            CROSSHAIR_COLOR,
        );

        next_frame().await
    }
}
