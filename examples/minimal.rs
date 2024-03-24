//! Spatial audio:
//! The spatial audio bundles provide all the components necessary for spatial audio.
//! Make sure your sound has a spatializer assigned to it in FMOD Studio.
//!
//! Controls:
//! Use WASD, Space, Shift and the mouse to move around.

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy::window::PresentMode;
use bevy_fmod::prelude::AudioSource;
use bevy_fmod::prelude::*;
use bevy_fmod_phonon::phonon_mesh::NeedsAudioMesh;
use bevy_fmod_phonon::phonon_plugin::PhononPlugin;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};
use std::f32::consts::PI;
use std::time::Duration;

use iyes_perf_ui::prelude::*;

#[derive(Component)]
struct TorusMarker;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
            FmodPlugin {
                audio_banks_paths: &[
                    "./assets/audio/demo_project/Build/Desktop/Master.bank",
                    "./assets/audio/demo_project/Build/Desktop/Master.strings.bank",
                    "./assets/audio/demo_project/Build/Desktop/Music.bank",
                ],
                plugin_paths: Some(&["./phonon_fmod.dll"]),
            },
            PhononPlugin,
        ))
        .add_plugins(LookTransformPlugin)
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(PostStartup, play_music)
        .add_systems(Update, move_object)
        // .add_systems(
        //     Update,
        //     remove_source.run_if(on_timer(Duration::from_secs(3))),
        // )
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    studio: Res<FmodStudio>,
) {
    commands.spawn(PerfUiCompleteBundle::default());

    // Cubes
    let mesh = meshes.add(Cuboid::from_size(Vec3::splat(0.3)));
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6));

    //28 -> 22k
    let cube_num = 3;

    for x in 0..cube_num {
        for y in 0..cube_num {
            for z in 0..cube_num {
                commands.spawn((
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform: Transform::from_rotation(Quat::from_rotation_x(PI * 0.5))
                            .with_translation(Vec3::new(x as f32, y as f32, z as f32)),
                        ..default()
                    },
                    NeedsAudioMesh,
                    TorusMarker,
                ));
            }
        }
    }

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-5.0, 4.0, -3.0),
        ..default()
    });
    // Camera
    commands
        .spawn(Camera3dBundle::default())
        .insert(SpatialListenerBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController::default(),
            Vec3::new(2.0, 0.0, -2.0),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        ));
    // Audio sources
    let event_description = studio.0.get_event("event:/Music/Radio Station").unwrap();

    commands
        .spawn(SpatialAudioBundle::new(event_description))
        .insert(PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
            transform: Transform::from_xyz(0.0, 0.5, 1.5).with_scale(Vec3::splat(0.05)),
            ..default()
        });

    // commands
    //     .spawn(SpatialAudioBundle::new(event_description))
    //     .insert(PbrBundle {
    //         mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //         material: materials.add(Color::rgb(0.8, 0.2, 0.2).into()),
    //         transform: Transform::from_xyz(0.0, 0.2, -1.5).with_scale(Vec3::splat(0.05)),
    //         ..default()
    //     });
}

fn move_object(mut obj_query: Query<&mut Transform, With<TorusMarker>>, time: Res<Time>) {
    let sin = time.elapsed_seconds().sin() * 0.01;

    for mut transform in &mut obj_query {
        transform.translation.y += sin;
    }
}

fn play_music(mut audio_sources: Query<&AudioSource>) {
    for audio_source in audio_sources.iter_mut() {
        audio_source.play();
    }
}

fn remove_source(
    mut commands: Commands,
    audio_sources: Query<(Entity, &AudioSource), With<AudioSource>>,
) {
    for (ent, audio_source) in audio_sources.iter() {
        audio_source.stop();
        commands.entity(ent).despawn_recursive();
    }
}
