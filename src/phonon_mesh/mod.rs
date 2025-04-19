pub(crate) mod instancing;
pub(crate) mod material;
mod mesh;

use crate::phonon_mesh::instancing::MeshParam;
use crate::phonon_plugin::SteamSimulation;
use audionimbus::InstancedMesh;
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct NeedsAudioMesh(pub material::PhononMaterial);

#[derive(Component)]
pub(crate) struct PhononMesh(InstancedMesh);

/// If an entity with a `NeedsAudioMesh` marker and a Bevy mesh exist, it will attempt to convert
/// the mesh to a Steam Audio mesh and add it to the audio world.
pub(crate) fn register_audio_meshes(
    mut commands: Commands,
    mut mesh_param: MeshParam,
    mut object_query: Query<(Entity, &Mesh3d, &NeedsAudioMesh)>,
) {
    for (ent, mesh_handle, requested_material) in &mut object_query {
        let instanced_mesh = mesh_param
            .create_instanced_mesh(mesh_handle, &requested_material.0)
            .unwrap();
        mesh_param.simulator.scene.add_instanced_mesh(&instanced_mesh);

        let scene_root = &mut mesh_param.simulator.scene;
        scene_root.commit();

        commands.entity(ent).insert(PhononMesh(instanced_mesh));
        commands.entity(ent).remove::<NeedsAudioMesh>();
    }
}

//Changed<GlobalTransform> or Changed Mesh? not worth it probably
pub(crate) fn update_audio_mesh_transforms(
    mut object_query: Query<(&GlobalTransform, &mut PhononMesh)>,
    simulation: ResMut<SteamSimulation>,
) {
    for (transform, mut audio_instance) in &mut object_query {
        let instanced_mesh = &mut audio_instance.0;
        let scene_root = &simulation.scene;
        // todo check if transpose is correct
        let tf_matrix = transform.compute_matrix().transpose();
        instanced_mesh.update_transform(
            scene_root,
            &audionimbus::Matrix::new(tf_matrix.to_cols_array_2d()),
        );
    }
}
