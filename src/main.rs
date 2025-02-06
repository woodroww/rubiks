use std::collections::HashMap;
use bevy::window::WindowResolution;
use bevy::{gltf::Gltf, reflect::List};
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_tweening::lens::TransformRotateAxisLens;
use bevy_tweening::{Animator, Tween, TweeningPlugin};
use rotate_plane::RotatePlane;

mod rotate_plane;

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
            PanOrbitCameraPlugin,
            TweeningPlugin,
        ))
        .add_systems(Startup, (spawn_camera, load_gltf))
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

/*
#[derive(Resource, Deref)]
struct StreamReceiver(Receiver<u32>);

#[derive(Event)]
struct StreamEvent(u32);
    */

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

fn setup_cube(
    mut commands: Commands,
    query: Query<(Entity, &mut Transform, &Name)>,
) {
    for (entity, _trans, name) in query.iter() {
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

fn keyboard(
    mut commands: Commands,
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

        let rotate_cubes = match press {
            KeyCode::KeyU => {
                //println!("left vertical plane counter clockwise");
                let plane_key = -2;
                Some((Vec3::Z, z_planes.get(&plane_key).unwrap()))
            }
            KeyCode::KeyI => {
                //println!("middle vertical plane counter clockwise");
                let plane_key = 0;
                Some((Vec3::Z, z_planes.get(&plane_key).unwrap()))
            }
            KeyCode::KeyO => {
                //println!("right vertical plane counter clockwise");
                let plane_key = 2;
                Some((Vec3::Z, z_planes.get(&plane_key).unwrap()))
            }
            KeyCode::KeyJ => {
                //println!("left vertical plane clockwise");
                let plane_key = -2;
                Some((Vec3::NEG_Z, z_planes.get(&plane_key).unwrap()))
            }
            KeyCode::KeyK => {
                //println!("middle vertical plane clockwise");
                let plane_key = 0;
                Some((Vec3::NEG_Z, z_planes.get(&plane_key).unwrap()))
            }
            KeyCode::KeyL => {
                //println!("right vertical plane clockwise");
                let plane_key = 2;
                Some((Vec3::NEG_Z, z_planes.get(&plane_key).unwrap()))
            }

            KeyCode::KeyW => {
                //println!("top plane clockwise");
                let plane_key = 2;
                Some((Vec3::Y, y_planes.get(&plane_key).unwrap()))
            }
            KeyCode::KeyR => {
                //println!("top plane counter clockwise");
                let plane_key = 2;
                Some((Vec3::Y, y_planes.get(&plane_key).unwrap()))
            }
            KeyCode::KeyS => {
                //println!("middle horizontal plane clockwise");
                let plane_key = 0;
                Some((Vec3::Y, y_planes.get(&plane_key).unwrap()))
            }
            KeyCode::KeyF => {
                //println!("middle horizontal plane counter clockwise");
                let plane_key = 0;
                Some((Vec3::NEG_Y, y_planes.get(&plane_key).unwrap()))
            }
            KeyCode::KeyX => {
                //println!("bottom plane clockwise");
                let plane_key = -2;
                Some((Vec3::NEG_Y, y_planes.get(&plane_key).unwrap()))
            }
            KeyCode::KeyV => {
                //println!("bottom plane counter clockwise");
                let plane_key = -2;
                Some((Vec3::NEG_Y, y_planes.get(&plane_key).unwrap()))
            }

            _ => {
                None
            }
        };

        if let Some((axis, cubes)) = rotate_cubes {
            for (cube, _) in cubes {
                if let Some((_, trans, _)) = query.iter_mut().find(|(entity, _, _)| entity == cube) {
                    let rotate = Tween::new(
                        EaseFunction::Linear,
                        std::time::Duration::from_millis(400),
                        RotatePlane {
                            axis,
                            start: 0.0,
                            end: (90.0 as f32).to_radians(),
                            org: *trans,
                        },
                    );
                    commands.entity(*cube).insert(Animator::new(rotate));
                }
            }
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    let eye = Vec3 {
        x: -20.0,
        y: 10.0,
        z: 7.0,
    };
    commands.spawn((
        // Note we're setting the initial position below with yaw, pitch, and radius, hence
        // we don't set transform on the camera.
        PanOrbitCamera {
            focus: Vec3::ZERO,
            button_orbit: MouseButton::Middle,
            button_pan: MouseButton::Middle,
            modifier_pan: Some(KeyCode::ShiftLeft),
            radius: Some(7.0),
            orbit_sensitivity: 0.5,
            ..default()
        },
    ));
}

/*
fn spawn_main_axis(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let length = 200.0;
    let width = 0.2;
    //let x = Cuboid::new(x_length, y_length, z_length);
    let x = Cuboid::new(length, width, width);
    let y = Cuboid::new(width, length, width);
    let z = Cuboid::new(width, width, length);

    let empty_transform = Transform::from_translation(Vec3::ZERO);
    let empty: Entity = commands
        .spawn_empty()
        .insert(empty_transform)
        .insert(Visibility::Visible)
        .insert(InheritedVisibility::default())
        .insert(Name::from("Main Axis"))
        .id();

    let mut transform = Transform::default();
    transform.translation.x = length / 2.0;

    commands.entity(empty).with_children(|parent| {
        parent
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(x)),
                    material: materials.add(Color::rgb(1.0, 0.0, 0.0)),
                    transform,
                    visibility: Visibility::Visible,
                    ..default()
                },
                bevy::pbr::NotShadowCaster,
            ))
            .insert(Name::from("x-axis"));
        let mut transform = Transform::default();
        transform.translation.y = length / 2.0;
        parent
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(y)),
                    material: materials.add(Color::rgb(0.0, 1.0, 0.0)),
                    transform,
                    visibility: Visibility::Visible,
                    ..default()
                },
                NotShadowCaster,
            ))
            .insert(Name::from("y-axis"));
        let mut transform = Transform::default();
        transform.translation.z = length / 2.0;
        parent
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(z)),
                    material: materials.add(Color::rgb(0.0, 0.0, 1.0)),
                    transform,
                    visibility: Visibility::Visible,
                    ..default()
                },
                NotShadowCaster,
                BoneAxis,
            ))
            .insert(Name::from("z-axis"));
    });
}
*/
