use bevy::prelude::*;
use bevy_mod_picking::prelude::Highlight;
use bevy_mod_picking::*;
use bevy_mod_picking::prelude::HighlightKind::Fixed;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, (spawn_cubes, spawn_camera))
        .run();
}

fn spawn_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // asset Highlighting for each entity
    // resource DefaultHighlighting global default

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
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.5, 0.1, 0.1).into()),
            transform: Transform::from_xyz(1.0, 0.0, 0.0),
            ..default()
        },
        PickableBundle::default(),
        //bevy_transform_gizmo::GizmoTransformable,
    ));

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
    let focus: Vec3 = Vec3::ZERO;

    let mut transform = Transform::default();
    transform.translation = Vec3 {
        x: -2.0,
        y: 2.5,
        z: 5.0,
    };
    transform.look_at(focus, Vec3::Y);

    let camera = Camera3dBundle {
        transform,
        ..Default::default()
    };

    commands.spawn(camera);
}
