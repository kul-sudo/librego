mod consts;
mod player;

use ::rand::{Rng, SeedableRng, rngs::StdRng};
use axum::{Json, Router, extract::Query, routing::post, serve};
use consts::*;
use macroquad::{audio::load_sound, prelude::*};
use parry3d_f64::{
    math::{Isometry, Vector},
    shape::{Compound, Cuboid, SharedShape},
};
use player::Player;
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env::vars,
    sync::{Arc, RwLock},
    thread::spawn,
    time::Duration,
};
use tokio::{net::TcpListener, runtime::Runtime};

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

#[derive(Serialize, Deserialize)]
struct Register {
    users: Vec<(usize, (f64, f64, f64))>,
    peers: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct MoveQuery {
    id: usize,
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize, Deserialize)]
struct RegisterQuery {
    id: usize,
    server: String,
    x: f64,
    y: f64,
    z: f64,
}

#[macroquad::main(window_conf)]
#[tokio::main]
async fn main() {
    for _ in 0..8 {
        set_fullscreen(true);
        next_frame().await;
    }

    let screen_size = vec2(screen_width(), screen_height());

    let mut rng = StdRng::from_os_rng();

    let id = rng.random_range(0..10000);
    let server = vars()
        .find(|(key, _)| key == "SERVER")
        .expect("SERVER must be specified.")
        .1;
    let host = vars()
        .find(|(key, _)| key == "HOST")
        .expect("HOST must be specified.")
        .1;

    let players = Arc::new(RwLock::new(HashMap::new()));
    let peers = Arc::new(RwLock::new(Vec::new()));

    let mut player = Player::new(dvec3(0.0, PLAYER_SIZE.y, 0.0));

    let players_clone = players.clone();
    let peers_clone = peers.clone();
    let server_clone = server.clone();

    spawn(move || {
        let proxy = Proxy::http("http://localhost:4444").unwrap();
        let client = Client::builder().proxy(proxy).build().unwrap();
        let players_clone = players_clone.clone();
        let peers_clone = peers_clone.clone();
        let server_clone = server_clone.clone();

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut params = HashMap::new();
            params.insert("id", id);
            let server_clone_clone = server_clone.clone();

            let res = client
                .post(server_clone + "/register")
                .query(&RegisterQuery {
                    id,
                    server: server_clone_clone,
                    x: player.position.x,
                    y: player.position.y,
                    z: player.position.z,
                })
                .send()
                .await
                .unwrap();

            let register: Register = res.json().await.unwrap();

            let mut players_write = players_clone.write().unwrap();
            for (id, (x, y, z)) in register.users {
                players_write.insert(id, Player::new(dvec3(x, y, z)));
            }

            let mut peers_write = peers_clone.write().unwrap();
            *peers_write = register.peers;
        })
    })
    .join()
    .unwrap();

    let players_clone = players.clone();
    let peers_clone = peers.clone();
    let server_clone = server.clone();

    spawn(move || {
        let app = Router::new()
            .route(
                "/register",
                post({
                    let players_clone = players_clone.clone();
                    let peers_clone = peers_clone.clone();
                    let server_clone = server_clone.clone();

                    move |query: Query<RegisterQuery>| async move {
                        let proxy = Proxy::http("http://localhost:4444").unwrap();
                        let client = Client::builder().proxy(proxy).build().unwrap();

                        let mut players_write = players_clone.write().unwrap();
                        players_write
                            .insert(query.id, Player::new(dvec3(query.x, query.y, query.z)));

                        let mut peers_write = peers_clone.write().unwrap();
                        for peer in peers_write.iter() {
                            let server_clone = server_clone.clone();
                            
                            let res = client
                                .post(peer.to_owned() + "/register")
                                .query(&RegisterQuery {
                                    id,
                                    server: server_clone,
                                    x: query.x,
                                    y: query.y,
                                    z: query.z,
                                })
                                .send()
                                .await
                                .unwrap();
                        }

                        peers_write.push(query.server.clone());

                        Json(Register {
                            users: players_write
                                .iter()
                                .map(|(id, player)| {
                                    (
                                        *id,
                                        (player.position.x, player.position.y, player.position.z),
                                    )
                                })
                                .collect::<Vec<_>>(),
                            peers: peers_write.to_vec(),
                        })
                    }
                }),
            )
            .route(
                "/move",
                post({
                    let players_clone = players_clone.clone();

                    move |query: Query<MoveQuery>| async move {
                        let mut players_write = players_clone.write().unwrap();
                        players_write.get_mut(&query.id).unwrap().position =
                            dvec3(query.x, query.y, query.z);
                    }
                }),
            );

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
            serve(listener, app).await.unwrap();
        })
    });

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
            Cuboid::new(Vector::new(0.0, 20.0, HALF)),
        ),
        (
            dvec3(0.0, 0.0, HALF),
            Cuboid::new(Vector::new(HALF, 20.0, 0.0)),
        ),
        (
            dvec3(0.0, 0.0, -HALF),
            Cuboid::new(Vector::new(HALF, 20.0, 0.0)),
        ),
        (
            dvec3(-HALF, 0.0, 0.0),
            Cuboid::new(Vector::new(0.0, 20.0, HALF)),
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

    let mut grabbed = true;
    set_cursor_grab(grabbed);
    show_mouse(false);

    set_pc_assets_folder("assets");
    let bullet_sound = load_sound("bullet.ogg").await.unwrap();

    loop {
        let delta = get_frame_time() as f64;

        if is_key_pressed(KeyCode::Tab) {
            grabbed = !grabbed;
            set_cursor_grab(grabbed);
            show_mouse(!grabbed);
        }

        let moved = player.movement(&compound);
        player.look(delta);
        player.bullets(&compound, &bullet_sound, moved, &mut rng);

        clear_background(BLACK);

        set_camera(&Camera3D {
            position: player.position.as_vec3(),
            up: player.up.as_vec3(),
            target: player.position.as_vec3() + player.front.as_vec3(),
            fovy: FOV,
            ..Default::default()
        });

        {
            let players_clone = players.read().unwrap();
            for other_player in players_clone.values() {
                draw_cube(
                    other_player.position.as_vec3(),
                    DVec3::from_slice(PLAYER_SIZE.as_slice()).as_vec3() * 2.0,
                    None,
                    RED,
                );
            }
        }

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

        if moved || player.jump.is_some() {
            let peers_clone = peers.clone();
            let server_clone = server.clone();

            spawn(move || {
                let proxy = Proxy::http("http://localhost:4444").unwrap();
                let client = Client::builder().proxy(proxy).build().unwrap();

                let peers_read = peers_clone.read().unwrap();

                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    for peer in peers_read.iter() {
                        let _ = client
                            .post(peer.to_owned() + "/move")
                            .timeout(Duration::from_millis(500))
                            .query(&MoveQuery {
                                id: id,
                                x: player.position.x,
                                y: player.position.y,
                                z: player.position.z,
                            })
                            .send()
                            .await;
                    }
                })
            });
        }

        next_frame().await
    }
}
