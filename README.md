Currently unmaintained, working on https://github.com/GitGhillie/phonon_rs instead.

# bevy_fmod_phonon

## Experimental warning:
Currently there are some major open issues so please don't use this for a serious project.

## Phonon?
Phonon is the name Valve uses internally for the Steam Audio library and shows up in this crate as well since it's easier to write.

## About
Bevy_fmod_phonon is a Bevy integration for the Steam Audio plugin for FMOD. This means that the Steam Audio effects are mostly handled by the FMOD plugin, and this crate just provides an easy way to supply the Steam Audio effects with geometry information from Bevy.

It is therefore assumed that the user is somewhat familiar with FMOD and in this case using it through bevy_fmod.

## Instructions/examples:
todo

## Build instructions/examples:
How to run the minimal example
1. Clone this repository
2. Follow the instructions for bevy_fmod
3. Follow the instructions for audionimbus
4. Create a folder named 'assets' in the root of the project
5. Open FMOD Studio, open the example project
6. Save the example project to the assets folder, with the name 'demo_project'
7. Create a folder called 'Plugins' in the FMOD project folder
8. From the Steam Audio FMOD release copy the libraries and the phonon_fmod.plugin.js into it
9. Replace the FMOD Spatializer in Music/Radio Station with the Steam Audio Spatializer effect
10. Save and build the FMOD project
11. Make sure the paths in examples/minimal.rs are correct
12. `cargo run --examples minimal`

## Contributing
Before opening a non-trivial PR please check with me in the issues section. Or send a DM on Discord.

## License
Dual licensed under Apache-2.0, MIT licenses.
