use macroquad::prelude::*;
use std::f32::consts::FRAC_PI_2;

const FOV: f32 = -80.0;

const PITCH_BOUND: f32 = FRAC_PI_2 * 0.99;

const GRAVITY: f32 = 0.005;

const MOVE_SPEED: f32 = 0.05;
const LOOK_SPEED: f32 = 0.05;

const JUMP_VELOCITY: f32 = 0.1;

const CAMERA_Y: f32 = 1.0;
const CROUCH_SPEED_CONST: f32 = 0.3;
const WALKING_SPEED_CONST: f32 = 0.5;

const CROUCH_LEVEL_CONST: f32 = 0.3;

const COLLISION_GAP: f32 = 0.1;

const CROSSHAIR_LINE_LENGTH: f32 = 8.0;
const CROSSHAIR_THICKNESS: f32 = 3.0;
const CROSSHAIR_COLOR: Color = DARKGREEN;

fn conf() -> Conf {
    Conf {
        window_title: String::from("Macroquad"),
        fullscreen: true,
        ..Default::default()
    }
}

#[derive(Default)]
struct Player {
    crouched: bool,
    walking: bool,
    jump: Option<f32>,
    yaw: f32,
    pitch: f32,
    front: Vec3,
    right: Vec3,
    up: Vec3,
    position: Vec3,
}

#[derive(Clone, Copy)]
enum Object {
    Cube { pos: Vec3, size: Vec3 },
}

const SIZE: f32 = 5.0;
const COLUMNS: usize = 10;

#[macroquad::main(conf)]
async fn main() {
    let mut objects = Vec::from([Object::Cube {
        pos: vec3(9.0, 0.0, 5.0),
        size: vec3(2.0, 8.0, 2.0),
    }]);

    let mut grid: [[[Option<Object>; COLUMNS]; COLUMNS]; 4] = [[[None; COLUMNS]; COLUMNS]; 4];

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

    let mut last_mouse_position: Vec2 = mouse_position().into();

    let mut grabbed = true;
    set_cursor_grab(grabbed);
    show_mouse(false);

    loop {
        let delta = get_frame_time();

        if is_key_pressed(KeyCode::Tab) {
            grabbed = !grabbed;
            set_cursor_grab(grabbed);
            show_mouse(!grabbed);
        }

        if is_key_pressed(KeyCode::LeftShift) {
            player.walking = !player.walking;
        }

        if is_key_pressed(KeyCode::Space) {
            if player.jump.is_none() && !player.crouched {
                player.jump = Some(-JUMP_VELOCITY);
            }
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
        }
        if is_key_down(KeyCode::S) {
            pos_delta -= player.front;
        }
        if is_key_down(KeyCode::A) {
            pos_delta -= player.right;
        }
        if is_key_down(KeyCode::D) {
            pos_delta += player.right;
        }

        if pos_delta.length() > 0.0 {
            pos_delta = pos_delta.normalize();
        }

        let mut position = player.position + pos_delta * move_speed;

        for object in &objects {
            match object {
                Object::Cube { pos, size } => {
                    let size_x_half = size.x / 2.0;
                    let size_z_half = size.z / 2.0;

                    dbg!(position);
                    dbg!(pos.with_x(pos.x - size_x_half));
                    dbg!(pos.with_x(pos.x + size_x_half));
                    dbg!(pos.with_z(pos.z - size_z_half));
                    dbg!(pos.with_z(pos.z + size_z_half));

                    // position.x = position.x.clamp(pos.x - size_x_half, pos.x + size_x_half);
                    // position.z = position.z.clamp(pos.z - size_z_half, pos.z + size_z_half);
                    if (pos.x - size_x_half - COLLISION_GAP..=pos.x + size_x_half + COLLISION_GAP)
                        .contains(&position.x)
                        && (pos.z - size_z_half - COLLISION_GAP
                            ..=pos.z + size_z_half + COLLISION_GAP)
                            .contains(&position.z)
                    {
                        let a = position.distance(pos.with_x(pos.x - size_x_half));
                        let b = position.distance(pos.with_x(pos.x + size_x_half));
                        let c = position.distance(pos.with_z(pos.z - size_z_half));
                        let d = position.distance(pos.with_z(pos.z + size_z_half));

                        if a < b && a < c && a < d {
                            position.x = pos.x - size_x_half - COLLISION_GAP;
                        } else if b < a && b < c && b < d {
                            position.x = pos.x + size_x_half + COLLISION_GAP;
                        } else if c < a && c < b && c < d {
                            position.z = pos.z - size_z_half - COLLISION_GAP;
                        } else if d < a && d < b && d < c {
                            position.z = pos.z + size_z_half + COLLISION_GAP;
                        }
                    }
                    // {
                    //     if pos.x - size_x_half < position.x {
                    //         dbg!(pos.x - size_x_half, position.x);
                    //         position.x = pos.x - size_x_half;
                    //     }
                    //     // else if pos.x + size_x_half < position.x {
                    //     //     position.x = pos.x + size_x_half;
                    //     // }
                    //
                    //     // if pos.z - size_z_half < position.z {
                    //     //     position.z = pos.z - size_z_half;
                    //     // }
                    // // else {
                    //     //     position.z = pos.z + size_z_half;
                    //     // }
                    // }

                    player.position = position;

                    // if ((pos.x - size_x_half..=pos.x + size_x_half).contains(&position.x)
                    //     && (pos.z - size_z_half..=pos.z + size_z_half).contains(&position.z))
                    // {
                    //     player.position.x = position.x.max(pos.x - size_x_half);
                    //     player.position.x = position.x.max(pos.x + size_x_half);
                    //
                    //     player.position.z = position.z.max(pos.z - size_z_half);
                    //     player.position.z = position.z.max(pos.z + size_z_half);
                    // } else {

                    // }
                    draw_cube(*pos, *size, None, BLACK);
                }
            }
        }
        // draw_cube(vec3(2.0, 1.0, 2.0), vec3(5.0, 5.0, 9.0), None, BLACK);

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
                Object::Cube { pos, size } => {
                    draw_cube(*pos, *size, None, BLACK);
                    draw_cube_wires(*pos, *size, WHITE);
                }
            }
        }
        draw_grid(COLUMNS as u32, SIZE, BLACK, GRAY);

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
