mod consts;
mod player;

use ::rand::{SeedableRng, rngs::StdRng};
use axum::{
    Json, Router,
    extract::Query,
    routing::{get, post},
    serve,
};
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
    peers: Vec<(String, (f64, f64, f64))>,
}

#[derive(Serialize, Deserialize)]
struct MoveQuery {
    host: String,
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize, Deserialize)]
struct RegisterQuery {
    host: String,
    x: f64,
    y: f64,
    z: f64,
}

async fn start(
    mut player: Player,
    peers: Arc<RwLock<HashMap<String, Player>>>,
    protocol: String,
    host: String,
    rng: &mut StdRng,
) {
    for _ in 0..8 {
        set_fullscreen(true);
        next_frame().await;
    }

    let screen_size = vec2(screen_width(), screen_height());

    let peers_clone = peers.clone();
    let host_clone = host.clone();

    spawn(move || {
        let app = Router::new()
            .route(
                "/register",
                post({
                    let peers_clone = peers_clone.clone();
                    let host_clone = host_clone.clone();

                    move |query: Query<RegisterQuery>| async move {
                        let mut peers_write = peers_clone.write().unwrap();
                        let mut new_peers = (*peers_write)
                            .clone()
                            .iter()
                            .map(|(peer_host, peer)| {
                                (
                                    peer_host.clone(),
                                    (peer.position.x, peer.position.y, peer.position.z),
                                )
                            })
                            .collect::<Vec<_>>();

                        new_peers.push((
                            host_clone,
                            (player.position.x, player.position.y, player.position.z),
                        ));

                        peers_write.insert(
                            query.host.clone(),
                            Player::new(dvec3(query.x, query.y, query.z)),
                        );

                        Json(Register { peers: new_peers })
                    }
                }),
            )
            .route(
                "/move",
                post({
                    let peers_clone = peers_clone.clone();

                    move |query: Query<MoveQuery>| async move {
                        let mut peers_write = peers_clone.write().unwrap();

                        peers_write.get_mut(&query.host).unwrap().position =
                            dvec3(query.x, query.y, query.z);
                    }
                }),
            );

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let listener = TcpListener::bind(host_clone).await.unwrap();
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
        if grabbed {
            player.look(delta);
        }
        player.bullets(&compound, &bullet_sound, moved, rng);

        clear_background(BLACK);

        set_camera(&Camera3D {
            position: player.position.as_vec3(),
            up: player.up.as_vec3(),
            target: player.position.as_vec3() + player.front.as_vec3(),
            fovy: FOV,
            ..Default::default()
        });

        let peers_clone = peers.clone();

        {
            let peers_read = peers_clone.read().unwrap();

            for other_player in peers_read.values() {
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

        let peers_clone = peers.clone();
        let host_clone = host.clone();
        let protocol_clone = protocol.clone();

        if moved || player.jump.is_some() {
            spawn(move || {
                // let proxy = Proxy::http("http://localhost:4444").unwrap();
                let client = Client::builder().build().unwrap();
                let peers_clone = peers_clone.read().unwrap();

                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    for peer_host in peers_clone.keys() {
                        let host_clone = host_clone.clone();
                        let protocol_clone = protocol_clone.clone();

                        let _ = client
                            .post(protocol_clone + peer_host + "/move")
                            // .timeout(Duration::from_millis(10))
                            .query(&MoveQuery {
                                host: host_clone,
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

#[macroquad::main(window_conf)]
#[tokio::main]
async fn main() {
    let mut rng = StdRng::from_os_rng();

    let server = match vars().find(|(key, _)| key == "SERVER") {
        Some(server) => Some(server.1),
        None => None,
    };
    let host = vars()
        .find(|(key, _)| key == "HOST")
        .expect("HOST must be specified.")
        .1;
    let protocol = vars()
        .find(|(key, _)| key == "PROTOCOL")
        .expect("PROTOCOL must be specified.")
        .1
        + "://";

    let peers = Arc::new(RwLock::new(HashMap::<String, Player>::new()));

    let player = Player::new(dvec3(0.0, PLAYER_SIZE.y, 0.0));

    if let Some(server) = server {
        let peers_clone = peers.clone();
        let host_clone = host.clone();
        let protocol_clone = protocol.clone();

        spawn(move || {
            // let proxy = Proxy::http("http://localhost:4444").unwrap();
            let client = Client::builder().build().unwrap();

            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let res = client
                    .post(protocol_clone.clone() + &server + "/register")
                    .query(&RegisterQuery {
                        host: host_clone.clone(),
                        x: player.position.x,
                        y: player.position.y,
                        z: player.position.z,
                    })
                    .send()
                    .await
                    .unwrap();

                let mut peers_write = peers_clone.write().unwrap();

                for peer_host in peers_write.keys() {
                    let _ = client
                        .post(protocol_clone.clone() + peer_host + "/register")
                        .query(&RegisterQuery {
                            host: host_clone.clone(),
                            x: player.position.x,
                            y: player.position.y,
                            z: player.position.z,
                        })
                        .send()
                        .await;
                }

                let register: Register = res.json().await.unwrap();

                for (peer_host, (x, y, z)) in register.peers {
                    peers_write.insert(peer_host, Player::new(dvec3(x, y, z)));
                }
            })
        })
        .join()
        .unwrap();
    }

    start(player, peers, protocol, host, &mut rng).await;
}
