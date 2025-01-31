use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};

use bevy::window::WindowResolution;
use bevy::{gltf::Gltf, reflect::List};
use bevy::prelude::*;
use bevy_tokio_tasks::{TaskContext, TokioTasksPlugin, TokioTasksRuntime};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

mod tokioio;

use hyper::{body::{Bytes, Body, Frame}, Method, Request, Response, StatusCode};

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(600.0, 600.0),
                title: "Cube this".to_string(),
                resizable: true,
                position: WindowPosition::At(IVec2::new(200, 200)),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            OrbitCameraPlugin::default(),
            LookTransformPlugin,
            TokioTasksPlugin::default(),
        ))
        .add_systems(Startup, (spawn_camera, load_gltf, start_hyper))
        .add_systems(Update, move_cubes.run_if(in_state(AppState::Running)))
        .add_systems(Update, spawn_gltf_objects.run_if(in_state(AppState::AssetLoading)))
        .add_systems(Update, keyboard.run_if(in_state(AppState::Running)))
        .add_systems(OnEnter(AppState::Running), setup_cube)
        .insert_state(AppState::default())
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AppState {
    #[default]
    AssetLoading,
    Running,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct RubikCube;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct RubikPlane {
}

#[derive(Resource)]
struct RubikScene(Handle<Gltf>);

fn load_gltf(mut commands: Commands, asset_server: Res<AssetServer>) {
    let gltf = asset_server.load("rubik.glb");
    commands.insert_resource(RubikScene(gltf));
}

fn spawn_gltf_objects(
    mut commands: Commands,
    helmet_scene: Res<RubikScene>,
    gltf_assets: Res<Assets<Gltf>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Wait until the scene is loaded
    let Some(gltf) = gltf_assets.get(&helmet_scene.0) else {
        return;
    };
    // Spawns the first scene in the file
    commands.spawn(SceneRoot(gltf.scenes[0].clone()));
    next_state.set(AppState::Running);
}

fn start_hyper(runtime: ResMut<TokioTasksRuntime>) {
    runtime.spawn_background_task(start_bloody_thing);
    println!("start_hyper");
}

async fn start_bloody_thing(ctx: TaskContext) {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = tokio::net::TcpListener::bind(addr).await.expect("fudge");
    println!("Listening on http://{}", addr);

    loop {

        let (stream, _) = listener.accept().await.expect("fudge");
        let io = tokioio::TokioIo::new(stream);

        tokio::task::spawn(async move {
            let http = hyper::server::conn::http1::Builder::new();
            let conn = http.serve_connection(io, hyper::service::service_fn(echo));
            if let Err(err) = conn.await {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        // curl 127.0.0.1:3000
        (&Method::GET, "/") => Ok(Response::new(full(
            "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d \"hello world\"`",
        ))),

        // Simply echo the body back to the client.
        // curl 127.0.0.1:3000/echo -XPOST -d "hello world"
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body().boxed())),

        // Convert to uppercase before sending back to client using a stream.
        (&Method::POST, "/echo/uppercase") => {
            let frame_stream = req.into_body().map_frame(|frame| {
                let frame = if let Ok(data) = frame.into_data() {
                    data.iter()
                        .map(|byte| byte.to_ascii_uppercase())
                        .collect::<Bytes>()
                } else {
                    Bytes::new()
                };

                Frame::data(frame)
            });

            Ok(Response::new(frame_stream.boxed()))
        }

        // Reverse the entire body before sending back to the client.
        //
        // Since we don't know the end yet, we can't simply stream
        // the chunks as they arrive as we did with the above uppercase endpoint.
        // So here we do `.await` on the future, waiting on concatenating the full body,
        // then afterwards the content can be reversed. Only then can we return a `Response`.
        (&Method::POST, "/echo/reversed") => {
            // To protect our server, reject requests with bodies larger than
            // 64kbs of data.
            let max = req.body().size_hint().upper().unwrap_or(u64::MAX);
            if max > 1024 * 64 {
                let mut resp = Response::new(full("Body too big"));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(resp);
            }

            let whole_body = req.collect().await?.to_bytes();

            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
            Ok(Response::new(full(reversed_body)))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn setup_cube(
    mut commands: Commands,
    query: Query<(Entity, &mut Transform, &Name)>,
) {
    for (entity, trans, name) in query.iter() {
        if name.starts_with("Cube.") {
            let mut splitsies = name.split("Cube.");
            let _nothing_before = splitsies.next();
            let after_cube = splitsies.next().unwrap();
            if !after_cube.contains(".") {
                commands.entity(entity).insert(RubikCube);
            }
        }
    }
}

fn move_cubes(mut query: Query<(Entity, &mut Transform), With<RubikCube>>, time: Res<Time>) {
    /*
    let mut z_buddies: HashMap<i32, Vec<Entity>> = HashMap::new();
    let mut y_buddies: HashMap<i32, Vec<Entity>> = HashMap::new();
    let mut x_buddies: HashMap<i32, Vec<Entity>> = HashMap::new();
    for (entity, transform) in &mut query {
        let entry = z_buddies.entry(transform.translation.z as i32).or_insert(vec![]);
        entry.push(entity);
        let entry = y_buddies.entry(transform.translation.y as i32).or_insert(vec![]);
        entry.push(entity);
        let entry = x_buddies.entry(transform.translation.x as i32).or_insert(vec![]);
        entry.push(entity);
    }
    println!("x_buddies");
    print_buddies(&x_buddies);
    println!("y_buddies");
    print_buddies(&y_buddies);
    println!("z_buddies");
    print_buddies(&z_buddies);

    let buddies = &y_buddies;
    // [-2, -4, 0]
    let plane_key = 0;
    let entry = buddies.get(&plane_key).unwrap();
    for cube in entry {
        if let Some((_, mut trans)) = query.iter_mut().find(|(entity, _)| entity == cube) {
            //trans.rotate_y(time.delta_secs() / 2.);
            trans.rotate_around(Vec3::default(), Quat::from_rotation_y(time.delta_secs()));
        }
    }
    */
}

fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Transform, &Name), With<RubikCube>>,
) {
    for press in keys.get_just_released() {

        // cubes can't be moving when getting the plane locations
        let mut z_planes: HashMap<i32, Vec<(Entity, Name)>> = HashMap::new();
        let mut y_planes: HashMap<i32, Vec<(Entity, Name)>> = HashMap::new();
        let mut x_planes: HashMap<i32, Vec<(Entity, Name)>> = HashMap::new();
        for (entity, transform, name) in &mut query {
            let entry = x_planes.entry(transform.translation.x.round() as i32).or_insert(vec![]);
            //println!("x_buddies {} {} -> {}", name, transform.translation.x, transform.translation.x.round() as i32);
            entry.push((entity, name.clone()));
            let entry = y_planes.entry(transform.translation.y.round() as i32).or_insert(vec![]);
            //println!("y_buddies {} {} -> {}", name, transform.translation.y, transform.translation.y.round() as i32);
            entry.push((entity, name.clone()));
            let entry = z_planes.entry(transform.translation.z.round() as i32).or_insert(vec![]);
            //println!("z_buddies {} {} -> {}", name, transform.translation.z, transform.translation.z.round() as i32);
            entry.push((entity, name.clone()));
            //println!();
        }
        for buddies in [&z_planes, &y_planes, &x_planes] {
            for (_k, v) in buddies {
                //assert!(v.len() == 9);
            }
        }
                /*
                let rotate = Tween::new(
                    EaseFunction::ExponentialInOut,
                    std::time::Duration::from_millis(REMOVE_DURATION),
                    TransformRotateZLens {
                        start: 0.0,
                        end: (90.0 as f32).to_radians(),
                    },
                );
                */


        match press {
            KeyCode::KeyU => {
                //println!("left vertical plane counter clockwise");
                let buddies = &z_planes;
                let plane_key = -2;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_z((-45.0_f32).to_radians()));
                    }
                }
            }
            KeyCode::KeyI => {
                //println!("middle vertical plane counter clockwise");
                let buddies = &z_planes;
                let plane_key = 0;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_z((-45.0_f32).to_radians()));
                    }
                }
            }
            KeyCode::KeyO => {
                //println!("right vertical plane counter clockwise");
                let buddies = &z_planes;
                let plane_key = 2;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_z((-45.0_f32).to_radians()));
                    }
                }
            }
            KeyCode::KeyJ => {
                //println!("left vertical plane clockwise");
                let buddies = &z_planes;
                let plane_key = -2;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_z((45.0_f32).to_radians()));
                    }
                }
            }
            KeyCode::KeyK => {
                //println!("middle vertical plane clockwise");
                let buddies = &z_planes;
                let plane_key = 0;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_z((45.0_f32).to_radians()));
                    }
                }
            }
            KeyCode::KeyL => {
                //println!("right vertical plane clockwise");
                let buddies = &z_planes;
                let plane_key = 2;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_z((45.0_f32).to_radians()));
                    }
                }
            }

            KeyCode::KeyW => {
                //println!("top plane clockwise");
                let buddies = &y_planes;
                let plane_key = 2;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_y((-45.0_f32).to_radians()));
                    }
                }
            }
            KeyCode::KeyR => {
                //println!("top plane counter clockwise");
                let buddies = &y_planes;
                let plane_key = 2;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_y((45.0_f32).to_radians()));
                    }
                }
            }
            KeyCode::KeyS => {
                //println!("middle horizontal plane clockwise");
                let buddies = &y_planes;
                let plane_key = 0;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_y((-45.0_f32).to_radians()));
                    }
                }
            }
            KeyCode::KeyF => {
                //println!("middle horizontal plane counter clockwise");
                let buddies = &y_planes;
                let plane_key = 0;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_y((45.0_f32).to_radians()));
                    }
                }
            }
            KeyCode::KeyX => {
                //println!("bottom plane clockwise");
                let buddies = &y_planes;
                let plane_key = -2;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_y((-45.0_f32).to_radians()));
                    }
                }
            }
            KeyCode::KeyV => {
                //println!("bottom plane counter clockwise");
                let buddies = &y_planes;
                let plane_key = -2;
                let entry = buddies.get(&plane_key).unwrap();
                for (cube, _) in entry {
                    if let Some((_, mut trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                        //trans.rotate_y(time.delta_secs() / 2.);
                        trans.rotate_around(Vec3::default(), Quat::from_rotation_y((45.0_f32).to_radians()));
                    }
                }
            }

            _ => {
            }
        }
    }
}

                /*
                let buddies = &x_buddies;
                println!("x_buddies");
                print_buddies_name(buddies);

                let buddies = &y_buddies;
                println!("y_buddies");
                print_buddies_name(buddies);

                let buddies = &z_buddies;
                println!("z_buddies");
                print_buddies_name(buddies);
                */

fn print_buddies_name(map: &HashMap<i32, Vec<(Entity, Name)>>) {
    for (k, v) in map {
        println!("key: {} len: {}", k, v.len());
        for (_, item) in v {
            println!("key: {} name: {}", k, item.as_str());
        }
    }
}
fn print_buddies(map: &HashMap<i32, Vec<Entity>>) {
    for (k, v) in map {
        println!("key: {} len: {}", k, v.len());
    }
}

fn spawn_camera(mut commands: Commands) {
    let target: Vec3 = Vec3::ZERO;

    let eye = Vec3 {
        x: -20.0,
        y: 10.0,
        z: 7.0,
    };
    let controller = OrbitCameraController::default();
    println!("controller.enabled: {}", controller.enabled);
    println!(
        "controller.mouse_rotate_sensitivity: {}",
        controller.enabled
    );
    println!(
        "controller.mouse_translate_sensitivity: {}",
        controller.enabled
    );
    println!(
        "controller.mouse_wheel_zoom_sensitivity: {}",
        controller.enabled
    );
    println!("controller.pixels_per_line: {}", controller.enabled);
    println!("controller.smoothing_weight: {}", controller.enabled);

    commands
        .spawn(Camera3d::default())
        .insert(OrbitCameraBundle::new(controller, eye, target, Vec3::Y));
}
