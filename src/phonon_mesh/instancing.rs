use crate::phonon_mesh::mesh::AudioMesh;
use crate::phonon_plugin::SteamSimulation;
use bevy::asset::{Assets, Handle};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Deref, DerefMut, Mesh, ResMut, Resource, Transform};
use std::collections::HashMap;
use steamaudio::scene::InstancedMesh;

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct StaticMeshes(HashMap<Handle<Mesh>, steamaudio::scene::Scene>);

/// Some information necessary to convert Bevy meshes to Steam Audio meshes
#[derive(SystemParam)]
pub(crate) struct MeshParam<'w> {
    pub bevy_meshes: ResMut<'w, Assets<Mesh>>,
    pub static_meshes: ResMut<'w, StaticMeshes>,
    pub simulator: ResMut<'w, SteamSimulation>,
}

impl<'w> MeshParam<'w> {
    /// Creates a Steam Audio Instanced Mesh from a Bevy Mesh.
    /// If the Bevy mesh has been converted before it will re-use the Steam Audio mesh.
    pub(crate) fn create_instanced_mesh(
        &mut self,
        mesh_handle: &Handle<Mesh>,
    ) -> Option<InstancedMesh> {
        create_instanced_mesh_internal(self, mesh_handle)
    }
}

fn create_instanced_mesh_internal(
    mesh_param: &mut MeshParam,
    mesh_handle: &Handle<Mesh>,
) -> Option<InstancedMesh> {
    let static_meshes = &mut mesh_param.static_meshes;
    let meshes = &mesh_param.bevy_meshes;
    let simulator = &mesh_param.simulator;
    let scene_root = &simulator.scene;

    if let Some(static_mesh_scene) = static_meshes.get(&mesh_handle) {
        // Turn that mesh into an instanced one, so it can be moved around.
        // todo: Differentiate between set-and-forget and movable audio meshes.
        // Currently compute_matrix will be called every frame for every mesh.

        let instanced_mesh = scene_root
            .create_instanced_mesh(static_mesh_scene, Transform::default().compute_matrix())
            .unwrap();

        Some(instanced_mesh)
    } else {
        // Create audio geometry
        if let Some(mesh) = meshes.get(&*mesh_handle) {
            let audio_mesh: AudioMesh = mesh.try_into().unwrap();

            // Create sub scene with static mesh, this will later be used to create the instanced mesh
            let sub_scene = simulator.context.create_scene().unwrap();

            // Add mesh
            let mut static_mesh = sub_scene
                .create_static_mesh(
                    audio_mesh.triangles.as_slice(),
                    audio_mesh.vertices.as_slice(),
                    audio_mesh.material_indices.as_slice(),
                    audio_mesh.materials.as_slice(),
                )
                .unwrap();
            static_mesh.set_visible(true);
            sub_scene.commit();

            static_meshes.insert(mesh_handle.clone(), sub_scene.clone());

            // Turn that mesh into an instanced one, so it can be moved around.
            // todo: Differentiate between set-and-forget and movable audio meshes.
            // Currently compute_matrix will be called every frame for every mesh.
            let instanced_mesh = scene_root
                .create_instanced_mesh(&sub_scene, Transform::default().compute_matrix())
                .unwrap();

            Some(instanced_mesh)
        } else {
            None // todo: Improve this mess. There is also a bit of duplicated code above
        }
    }
}
