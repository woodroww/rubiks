use bevy::prelude::*;
use bevy_mod_picking::prelude::Highlight;
use bevy_mod_picking::prelude::HighlightKind::Fixed;
use bevy_mod_picking::*;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};
use bevy::gltf::Gltf;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultPickingPlugins))
        .add_plugins((LookTransformPlugin, OrbitCameraPlugin::default()))
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, (spawn_cubes, spawn_camera, load_gltf))
        .add_systems(Update, spawn_gltf_objects)
        .run();
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

#[derive(Resource)]
struct MyAssetPack(Handle<Gltf>);

fn load_gltf(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let gltf = asset_server.load("rubiks.gltf");
    commands.insert_resource(MyAssetPack(gltf));
}

fn spawn_gltf_objects(
    mut commands: Commands,
    my: Res<MyAssetPack>,
    assets_gltf: Res<Assets<Gltf>>,
    mut spawned: Local<bool>
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

fn spawn_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
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
