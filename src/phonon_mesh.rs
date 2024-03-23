use crate::phonon_plugin::SteamSimulation;
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use steamaudio::scene::InstancedMesh;

#[derive(Component)]
pub struct NeedsAudioMesh;

#[derive(Component)]
pub(crate) struct PhononMesh(InstancedMesh);

pub(crate) fn register_audio_meshes(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    mut object_query: Query<(Entity, &mut Handle<Mesh>), With<NeedsAudioMesh>>,
    simulator: ResMut<SteamSimulation>,
) {
    for (ent, mesh_handle) in &mut object_query {
        // Create some audio geometry
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

            if let Some(scene_root) = &simulator.scene {
                // Turn that mesh into an instanced one, so it can be moved around.
                // todo: Differentiate between set-and-forget and movable audio meshes.
                // Currently compute_matrix will be called every frame for every mesh.
                let some_transform = Transform::default();
                let mut instanced_mesh = scene_root
                    .create_instanced_mesh(&sub_scene, some_transform.compute_matrix())
                    .unwrap();
                instanced_mesh.set_visible(true);
                scene_root.commit();

                commands.entity(ent).insert(PhononMesh(instanced_mesh));
                commands.entity(ent).remove::<NeedsAudioMesh>();
            }
        }
    }
}

pub(crate) fn move_audio_meshes(mut object_query: Query<(&GlobalTransform, &mut PhononMesh)>) {
    for (transform, mut audio_instance) in &mut object_query {
        let instanced_mesh = &mut audio_instance.0;
        instanced_mesh.set_transform(transform.compute_matrix());
    }
}

pub struct AudioMesh {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
    pub materials: Vec<steamaudio::scene::Material>,
    pub material_indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub enum AudioMeshError {
    NoVertices,
    NonTrianglePrimitiveTopology(PrimitiveTopology),
}

// Original code from https://github.com/Aceeri/bevy-steam-audio/blob/main/src/source.rs
//todo: ability to change material
impl TryFrom<&Mesh> for AudioMesh {
    type Error = AudioMeshError;
    fn try_from(mesh: &Mesh) -> Result<Self, Self::Error> {
        let triangles = match mesh.indices() {
            Some(indices) => {
                let indices: Vec<_> = match indices {
                    Indices::U16(indices) => {
                        indices.iter().map(|indices| *indices as u32).collect()
                    }
                    Indices::U32(indices) => {
                        indices.iter().map(|indices| *indices as u32).collect()
                    }
                };

                match mesh.primitive_topology() {
                    PrimitiveTopology::TriangleList => indices
                        .chunks_exact(3)
                        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                        .collect(),
                    PrimitiveTopology::TriangleStrip => {
                        let mut indices: Vec<_> = indices
                            .windows(3)
                            .map(|indices| [indices[0], indices[1], indices[2]])
                            .collect();

                        for (index, indices) in indices.iter_mut().enumerate() {
                            if (index + 1) % 2 == 0 {
                                *indices = [indices[1], indices[0], indices[2]];
                            }
                        }

                        indices
                    }
                    topology => return Err(AudioMeshError::NonTrianglePrimitiveTopology(topology)),
                }
            }
            None => Vec::new(),
        };

        let vertices = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(positions) => match positions {
                VertexAttributeValues::Float32x3(vertices) => {
                    vertices.iter().map(|a| (*a).into()).collect()
                }
                _ => return Err(AudioMeshError::NoVertices),
            },
            _ => return Err(AudioMeshError::NoVertices),
        };

        let material = steamaudio::scene::Material {
            absorption: [0.10, 0.20, 0.30],
            scattering: 0.05,
            transmission: [0.10, 0.05, 0.03],
        };

        let materials = vec![material];
        let material_indices = triangles.iter().map(|_| 0 /* GENERIC index */).collect();

        Ok(Self {
            vertices,
            triangles,
            materials,
            material_indices,
        })
    }
}
