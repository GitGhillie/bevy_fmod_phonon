pub(crate) mod instancing;
mod material;
mod mesh;

use crate::phonon_mesh::instancing::MeshParam;
use bevy::prelude::*;
use steamaudio::scene::InstancedMesh;

#[derive(Component)]
pub struct NeedsAudioMesh;

#[derive(Component)]
pub(crate) struct PhononMesh(InstancedMesh);

/// If an entity with a `NeedsAudioMesh` marker and a Bevy mesh exist, it will attempt to convert
/// the mesh to a Steam Audio mesh and add it to the audio world.
pub(crate) fn register_audio_meshes(
    mut commands: Commands,
    mut mesh_param: MeshParam,
    mut object_query: Query<(Entity, &Handle<Mesh>), With<NeedsAudioMesh>>,
) {
    for (ent, mesh_handle) in &mut object_query {
        let mut instanced_mesh = mesh_param.create_instanced_mesh(mesh_handle).unwrap();
        instanced_mesh.set_visible(true);

        let scene_root = &mesh_param.simulator.scene;
        scene_root.commit();

        commands.entity(ent).insert(PhononMesh(instanced_mesh));
        commands.entity(ent).remove::<NeedsAudioMesh>();
    }
}

pub(crate) fn update_audio_mesh_transforms(
    mut object_query: Query<(&GlobalTransform, &mut PhononMesh), Changed<GlobalTransform>>,
) {
    for (transform, mut audio_instance) in &mut object_query {
        let instanced_mesh = &mut audio_instance.0;
        instanced_mesh.set_transform(transform.compute_matrix());
    }
}