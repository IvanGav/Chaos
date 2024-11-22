# Chaos
### Made for TPC Fall 2024 Game Jam.

It's a small chaos theory inspired demo.

Default equation is Lorenz Attractor.

## Building

- To run in debug mode: `$cargo run`
- To build for release: `$cargo build --release` 
    - the compiled exe will be here: `./target/release/Chaos.exe`

## Controls

- `LMB` - spawn a particle
    - `Shift + LMB` - spawn a bunch of particles very close to each other
- `Left Alt + Move Mouse` - orbit camera
- `Left Ctrl + Move Mouse` - pan camera
- `Z + Move Mouse` OR `Scroll Wheel` - zoom
- Numpad `+`/`-` - increase/decrease amount of steps taken every frame
- `[`/`]` - decrease/increase the delta time (will affect the simulation)
    - Low dt will make everything converge at the origin. This is correct Lorenz Attractor behavior, as far as I understand.
- `c` - clear the screen of particles

## Credits

- This project uses the Bevy Engine.
- Ryan did not help making this project.