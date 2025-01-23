use std::collections::HashMap;

use bevy::{gltf::Gltf, reflect::List};
use bevy::prelude::*;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            OrbitCameraPlugin::default(),
            LookTransformPlugin,
        ))
        .add_systems(Startup, (spawn_camera, load_gltf))
        .add_systems(Update, move_cubes.run_if(in_state(MyStates::Running)))
        .add_systems(Update, spawn_gltf_objects.run_if(in_state(MyStates::AssetLoading)))
        .add_systems(OnEnter(MyStates::Running), setup_cube)
        .insert_state(MyStates::default())
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
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
    mut next_state: ResMut<NextState<MyStates>>,
) {
    // Wait until the scene is loaded
    let Some(gltf) = gltf_assets.get(&helmet_scene.0) else {
        return;
    };
    // Spawns the first scene in the file
    commands.spawn(SceneRoot(gltf.scenes[0].clone()));
    next_state.set(MyStates::Running);
}

fn setup_cube(
    mut commands: Commands,
    query: Query<(Entity, &mut Transform, &Name)>,
) {
    for (entity, trans, name) in query.iter() {
        let mut splitsies = name.split("Cube.");
        let _nothing_before = splitsies.next();
        let after_cube = splitsies.next().unwrap();
        if !after_cube.contains(".") {
            commands.entity(entity).insert(RubikCube);
        }
    }
}

fn move_cubes(mut query: Query<(Entity, &mut Transform), With<RubikCube>>, time: Res<Time>) {
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
    /*
    println!("x_buddies");
    print_buddies(&x_buddies);
    println!("y_buddies");
    print_buddies(&y_buddies);
    println!("z_buddies");
    print_buddies(&z_buddies);
    */

    let buddies = &y_buddies;
    // [-2, -4, 0]
    let plane_key = 0;
    let entry = buddies.get(&plane_key).unwrap();
    for cube in entry {
        if let Some((_, mut trans)) = query.iter_mut().find(|(entity, _)| entity == cube) {
            //trans.rotate_y(time.delta_secs() / 2.);
            trans.rotate_around(Vec3::default(), Quat::from_rotation_y(time.delta_secs() / 2.0));
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
        x: -2.0,
        y: 2.5,
        z: 8.0,
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
