use crate::phonon_mesh::material::{materials, Material};
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};

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
                    Indices::U32(indices) => indices.iter().map(|indices| *indices).collect(),
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

        let material: steamaudio::scene::Material = materials::GLASS.into();

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
