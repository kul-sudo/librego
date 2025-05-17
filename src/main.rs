mod consts;
mod player;

use ::rand::{Rng, SeedableRng, rngs::StdRng};
use bincode::{Decode, Encode, config, decode_from_slice, encode_into_slice};
use consts::*;
use macroquad::{audio::load_sound, prelude::*};
use parry3d_f64::{
    math::{Isometry, Vector},
    shape::{Compound, Cuboid, SharedShape},
};
use player::Player;
use std::{
    collections::HashMap,
    env::vars,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, RwLock},
    thread::spawn,
    time::Instant,
};

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

#[derive(Encode, Decode)]
struct Peers {
    peers: Vec<(SocketAddr, [f64; 3])>,
}

#[derive(Encode, Decode)]
struct MoveQuery {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Encode, Decode)]
struct RegisterQuery {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Encode, Decode)]
enum Event {
    MoveQuery(MoveQuery),
    RegisterQuery(RegisterQuery),
    Killed,
    Peers(Peers),
}

#[derive(Encode, Decode)]
struct Packet {
    event: Event,
}

async fn start(
    mut player: Player,
    peers: Arc<RwLock<HashMap<SocketAddr, Player>>>,
    socket: Arc<RwLock<UdpSocket>>,
    rng: &mut StdRng,
) {
    for _ in 0..8 {
        set_fullscreen(true);
        next_frame().await;
    }

    let screen_size = vec2(screen_width(), screen_height());

    let peers_clone = peers.clone();
    let socket_clone = socket.clone();
    let player_clone = player.clone();

    spawn(move || {
        let socket = socket_clone.read().unwrap();

        let config = config::standard();
        let peers_clone = peers_clone.clone();
        let player_clone = player_clone.clone();

        loop {
            let mut buf = [0; 100];
            let (amt, src) = socket.recv_from(&mut buf).unwrap();

            let (packet, _): (Packet, _) = decode_from_slice(&buf[..amt], config).unwrap();

            match packet.event {
                Event::MoveQuery(query) => {
                    let mut peers_write = peers_clone.write().unwrap();

                    let peer = peers_write.get_mut(&src).unwrap();
                    peer.position = peer.position.lerp(dvec3(query.x, query.y, query.z), 0.5);
                    peer.position.y = query.y;
                    if peer.ticks.len() > TICKS_PER_SECOND {
                        peer.ticks.clear()
                    } else {
                        peer.ticks.push(Some(peer.position))
                    }
                }
                Event::RegisterQuery(query) => {
                    let mut peers_write = peers_clone.write().unwrap();
                    let mut new_peers = (*peers_write)
                        .clone()
                        .iter()
                        .map(|(peer_host, peer)| {
                            (
                                peer_host.clone(),
                                [peer.position.x, peer.position.y, peer.position.z],
                            )
                        })
                        .collect::<Vec<_>>();

                    new_peers.push((
                        socket.local_addr().unwrap(),
                        [
                            player_clone.position.x,
                            player_clone.position.y,
                            player_clone.position.z,
                        ],
                    ));

                    peers_write.insert(src, Player::new(dvec3(query.x, query.y, query.z)));

                    let mut buf_send = [0; 100];

                    let length = encode_into_slice(
                        Packet {
                            event: Event::Peers(Peers { peers: new_peers }),
                        },
                        &mut buf_send,
                        config,
                    )
                    .unwrap();
                    let buf_send_filled = &buf_send[..length];
                    socket.send_to(&buf_send_filled, &src).unwrap();
                }
                Event::Killed => {}
                _ => panic!(),
            }
        }
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
        player.bullets(peers.clone(), &bullet_sound, moved, rng);

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

            for peer in peers_read.values() {
                draw_cube(
                    peer.position.as_vec3(),
                    DVec3::from_slice(PLAYER_SIZE.as_slice()).as_vec3() * 2.0,
                    None,
                    if peer.killed { GRAY } else { RED },
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

        let peers_clone = (*peers.read().unwrap()).clone();

        let config = config::standard();

        if player.last_tick_timestamp.elapsed() >= *DURATION_PER_TICK {
            player.last_tick_timestamp = Instant::now();
            if player.ticks.len() > TICKS_PER_SECOND {
                player.ticks.clear()
            } else {
                player.ticks.push(Some(player.position))
            }

            let mut buf_send = [0; 100];
            let length = encode_into_slice(
                Packet {
                    event: Event::MoveQuery(MoveQuery {
                        x: player.position.x,
                        y: player.position.y,
                        z: player.position.z,
                    }),
                },
                &mut buf_send,
                config,
            )
            .unwrap();
            let buf_send_filled = &buf_send[..length];

            let socket_read = socket.read().unwrap();
            for peer_host in peers_clone.keys() {
                socket_read.send_to(&buf_send_filled, peer_host).unwrap();
            }
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
    let socket = Arc::new(RwLock::new(UdpSocket::bind(&host).unwrap()));

    let peers = Arc::new(RwLock::new(HashMap::<SocketAddr, Player>::new()));

    let mut player = Player::new(dvec3(0.0, PLAYER_SIZE.y, 0.0));

    if let Some(server) = server {
        let config = config::standard();

        let mut buf_send = [0; 100];
        let length = encode_into_slice(
            Packet {
                event: Event::RegisterQuery(RegisterQuery {
                    x: player.position.x,
                    y: player.position.y,
                    z: player.position.z,
                }),
            },
            &mut buf_send,
            config,
        )
        .unwrap();
        let buf_send_filled = &buf_send[..length];
        let socket_read = socket.read().unwrap();
        socket_read.send_to(&buf_send_filled, server).unwrap();

        let mut buf = [0; 100];
        let (amt, src) = socket_read.recv_from(&mut buf).unwrap();

        let (packet, _): (Packet, _) = decode_from_slice(&buf[..amt], config).unwrap();
        if let Event::Peers(query) = packet.event {
            let mut peers_write = peers.write().unwrap();

            for (peer_host, pos) in query.peers {
                if peer_host != src {
                    socket_read.send_to(&buf_send_filled, &peer_host).unwrap();
                }
                peers_write.insert(peer_host, Player::new(DVec3::from_slice(&pos)));
            }
        } else {
            panic!()
        }
    }

    start(player, peers, socket, &mut rng).await;
}
