use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
//use bevy_mod_picking::prelude::Highlight;
//use bevy_mod_picking::prelude::HighlightKind::Fixed;
use bevy_gltf_components::ComponentsFromGltfPlugin;
use bevy_mod_picking::*;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

use bevy_asset_loader::prelude::*;
use bevy_registry_export::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultPickingPlugins, ComponentsFromGltfPlugin))
        .add_plugins((LookTransformPlugin, OrbitCameraPlugin::default()))
        .add_plugins(ExportRegistryPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, move_cubes)
        //.add_systems(Update, spawn_gltf_objects)
        .add_systems(OnEnter(MyStates::Next), load_gltf)
        .add_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>(),
        )
        .register_type::<RubikCube>()
        .run();
}


#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "rubiks.glb")]
    rubik: Handle<Gltf>,
}

#[derive(
    Clone, Eq, PartialEq, Debug, Hash, Default, States,
)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
struct RubikCube;


fn load_gltf(
    mut commands: Commands,
    assets: Res<MyAssets>,
    models: Res<Assets<bevy::gltf::Gltf>>,
) {
    let gltf = models.get(assets.rubik.clone()).unwrap();
    commands.spawn(SceneBundle {
        scene: gltf.scenes[0].clone(),
        ..Default::default()
    });
}

fn move_cubes(
    mut query: Query<&mut Transform, With<RubikCube>>,
    time: Res<Time>
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
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
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(controller, eye, target, Vec3::Y));
}


// asset Highlighting for each entity
// resource DefaultHighlighting global default

/*
commands.spawn((
    PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(-1.0, 0.0, 0.0),
        ..default()
    },
    Highlight {
        hovered: Some(Fixed(materials.add(Color::rgb(0.9, 0.8, 0.7).into()))),
        pressed: Some(Fixed(materials.add(Color::rgb(0.0, 1.0, 0.0).into()))),
        selected: Some(Fixed(materials.add(Color::rgb(0.7, 0.6, 0.5).into()))),
    },
    PickableBundle::default(),
    //bevy_transform_gizmo::GizmoTransformable,
));


for x in 0..3 {
    for y in 0..3 {
        for z in 0..3 {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
                    transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                    ..default()
                },
                PickableBundle::default(),
                //bevy_transform_gizmo::GizmoTransformable,
            ));
        }
    }
}
*/


/*
fn spawn_gltf_objects(
    mut commands: Commands,
    my: Res<MyAssetPack>,
    assets_gltf: Res<Assets<Gltf>>,
    mut spawned: Local<bool>,
) {
    if !*spawned {
        if let Some(gltf) = assets_gltf.get(&my.0) {
            commands.spawn(SceneBundle {
                scene: gltf.scenes[0].clone(),
                ..Default::default()
            });
            *spawned = true;
        }
    }
}
*/

