use crate::phonon_mesh;
use crate::phonon_mesh::StaticMeshes;
use bevy::prelude::*;
use bevy_fmod::prelude::AudioListener;
use bevy_fmod::prelude::AudioSource;
use libfmod::{Dsp, DspType, EventInstance};
use steamaudio::context::Context;
use steamaudio::fmod;
use steamaudio::geometry::Orientation;
use steamaudio::hrtf::Hrtf;
use steamaudio::simulation::{AirAbsorptionModel, DistanceAttenuationModel, Simulator, Source};

#[derive(Component)]
struct PhononSource {
    address: i32,
    source: Source,
}

#[derive(Component)]
pub struct PhononStaticMeshMarker;

//todo move or remove pub
#[derive(Resource)]
pub struct SteamSimulation {
    pub context: Context,
    pub hrtf: Hrtf,
    pub simulator: Simulator,
    pub scene: steamaudio::scene::Scene,
}

pub struct PhononPlugin;

impl Plugin for PhononPlugin {
    fn build(&self, app: &mut App) {
        let sampling_rate = 48000; // Needs to be equal to FMOD sampling rate.
        let frame_size = 1024;
        let context = Context::new().unwrap();

        let hrtf = context.create_hrtf(sampling_rate, frame_size).unwrap();

        // todo! simulationsettings !!!!!!!!!!!!!!!!!!!!!!!
        // simulation_settings.max_num_occlusion_samples = 10; // This only sets the max, the actual amount is set per source
        let mut simulator = context.create_simulator(sampling_rate, frame_size).unwrap();

        let scene = context.create_scene().unwrap();

        scene.commit();

        simulator.set_scene(&scene);
        simulator.commit(); //todo remove?

        // todo: must be initialized before creating any steam audio DSP effects. So it might be possible to do it somewhere else if that helps code organization
        fmod::init_fmod(&context);
        fmod::fmod_set_hrtf(&hrtf);
        //todo get rid of hardcoded value
        let settings = fmod::fmod_create_settings(48000, 1024);
        fmod::fmod_set_simulation_settings(settings);

        app.insert_resource(SteamSimulation {
            simulator,
            context,
            hrtf,
            scene, //todo remove option(?)
        })
        .insert_resource(StaticMeshes::default())
        .add_systems(
            Update,
            (
                (
                    register_phonon_sources,
                    phonon_mesh::register_audio_meshes,
                    phonon_mesh::move_audio_meshes,
                    update_steam_audio_listener,
                    update_steam_audio_source,
                ),
                update_steam_audio,
            )
                .chain(),
        );
    }
}

fn update_steam_audio_listener(
    mut sim_res: ResMut<SteamSimulation>,
    listener_query: Query<&GlobalTransform, With<AudioListener>>,
) {
    let listener_transform = listener_query.get_single().unwrap();
    let (_rotation, rotation, translation) = listener_transform.to_scale_rotation_translation();

    sim_res.simulator.set_listener(Orientation {
        translation,
        rotation,
    });

    //sim_res.simulator.set_reflections(4096, 16, 2.0, 1, 1.0);
}

fn update_steam_audio_source(mut source_query: Query<(&GlobalTransform, &mut PhononSource)>) {
    for (source_transform, mut phonon_source) in source_query.iter_mut() {
        let (_rotation, rotation, translation) = source_transform.to_scale_rotation_translation();

        phonon_source.source.set_source(Orientation {
            translation,
            rotation,
        });
    }
}

fn update_steam_audio(sim_res: ResMut<SteamSimulation>) {
    // Commit changes to the sources, listener and scene.
    sim_res.simulator.commit();

    sim_res.simulator.run_direct();
    //sim_res.simulator.run_reflections(); //todo make optional

    // The Steam Audio FMOD plugin will periodically collect the simulation outputs
    // as long as the plugin has handles to the Steam Audio sources.
    // See function `register_phonon_sources`.
}

fn register_phonon_sources(
    mut audio_sources: Query<(Entity, &AudioSource), Without<PhononSource>>,
    mut commands: Commands,
    sim_res: Res<SteamSimulation>,
) {
    for (audio_entity, audio_source_fmod) in audio_sources.iter_mut() {
        if let Some(phonon_dsp) = get_phonon_spatializer(audio_source_fmod.event_instance) {
            let mut source = sim_res.simulator.create_source().unwrap();
            source.set_active(true);
            source.set_distance_attenuation(DistanceAttenuationModel::Default);
            source.set_air_absorption(AirAbsorptionModel::Default);
            source.set_occlusion();
            source.set_transmission(3);
            //source.set_reflections();

            let source_address = fmod::fmod_add_source(&source);
            let simulation_outputs_parameter_index = 33; //todo explain where this number comes from

            // By setting this field the Steam Audio FMOD plugin can retrieve the
            // simulation results like occlusion and reflection.
            phonon_dsp
                .set_parameter_int(simulation_outputs_parameter_index, source_address)
                .unwrap();

            commands.entity(audio_entity).insert(PhononSource {
                address: source_address,
                source,
            });
        }
    }
}

// // Deregister phonon source
// impl Drop for PhononSource {
//     fn drop(&mut self) {
//         println!("Dropping source!");
//         //todo fmod remove source
//     }
// }

//todo delete PhononSource using the address and iplFMODRemoveSource

// todo: The function below could potentially be simplified/improved with the help
// of FMODGetPluginDescriptionList() or FMOD_SteamAudio_Spatialize_GetDSPDescription()

/// The goal here is to find the Steam Audio Spatializer DSP associated with an instance.
/// This way we can later set its parameters.
/// The DSP can basically be anywhere in the DSP chain, so we have to search for it.
pub fn get_phonon_spatializer(instance: EventInstance) -> Option<Dsp> {
    if let Ok(channel_group) = instance.get_channel_group() {
        let num_groups = channel_group.get_num_groups().unwrap();
        //println!("num Groups: {}", num_groups);

        for index_group in 0..num_groups {
            let group = channel_group.get_group(index_group).unwrap();
            let group_num_dsp = group.get_num_ds_ps().unwrap();
            //println!("Group {} num DSPs: {}", index_group, group_num_dsp);

            for index_dsp in 0..group_num_dsp {
                let dsp = group.get_dsp(index_dsp).unwrap();
                let dsp_type = dsp.get_type().unwrap();
                //println!("type: {:?}", dsp_type);

                // Plugin DSPs don't have a known type
                if dsp_type == DspType::Unknown {
                    let dsp_num_parameters = dsp.get_num_parameters().unwrap();
                    //println!("num parameters: {:?}", dsp_num_parameters);

                    // The Steam Audio Spatializer DSP has 34 parameters
                    if dsp_num_parameters == 34 {
                        // Now we know that it's most likely the Steam Audio Spatializer.
                        return Some(dsp);
                    }
                }
            }
        }
    }

    None
}
