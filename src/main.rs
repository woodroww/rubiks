use bevy::gltf::Gltf;
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
        .register_type::<RubikCube>()
        .add_systems(Startup, (spawn_camera, load_gltf))
        .add_systems(Update, move_cubes)
        .add_systems(Update, spawn_gltf_objects.run_if(in_state(MyStates::AssetLoading)))
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
pub struct RubikCube {
    name: String
}

#[derive(Resource)]
struct RubikScene(Handle<Gltf>);

fn load_gltf(mut commands: Commands, asset_server: Res<AssetServer>) {
    let gltf = asset_server.load("rubiks.glb");
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

fn move_cubes(mut query: Query<(&mut Transform, &Name)>, time: Res<Time>) {
    for (mut transform, _name) in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
}

fn spawn_camera(mut commands: Commands) {
    let target: Vec3 = Vec3::ZERO;

    let eye = Vec3 {
        x: -2.0,
        y: 2.5,
        z: 5.0,
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
