use bevy::ecs::system::SystemId;
use bevy::input::common_conditions::{input_just_pressed, input_pressed};
// use bevy::render::camera::ScalingMode;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy::color::palettes::css::*;
use rand::Rng;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

mod chaos_equations;

use chaos_equations as chaos;

/*
    Stolen Camera Code
*/

// Bundle to spawn our custom camera easily
#[derive(Bundle, Default)]
pub struct PanOrbitCameraBundle {
    pub camera: Camera3dBundle,
    pub state: PanOrbitState,
    pub settings: PanOrbitSettings,
}

// The internal state of the pan-orbit controller
#[derive(Component)]
pub struct PanOrbitState {
    pub center: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    pub pitch: f32,
    pub yaw: f32,
}

/// The configuration of the pan-orbit controller
#[derive(Component)]
pub struct PanOrbitSettings {
    /// World units per pixel of mouse motion
    pub pan_sensitivity: f32,
    /// Radians per pixel of mouse motion
    pub orbit_sensitivity: f32,
    /// Exponent per pixel of mouse motion
    pub zoom_sensitivity: f32,
    /// Key to hold for panning
    pub pan_key: Option<KeyCode>,
    /// Key to hold for orbiting
    pub orbit_key: Option<KeyCode>,
    /// Key to hold for zooming
    pub zoom_key: Option<KeyCode>,
    /// What action is bound to the scroll wheel?
    pub scroll_action: Option<PanOrbitAction>,
    /// For devices with a notched scroll wheel, like desktop mice
    pub scroll_line_sensitivity: f32,
    /// For devices with smooth scrolling, like touchpads
    pub scroll_pixel_sensitivity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanOrbitAction {
    Pan,
    Orbit,
    Zoom,
}

impl Default for PanOrbitState {
    fn default() -> Self {
        PanOrbitState {
            center: Vec3::ZERO,
            radius: 1.0,
            upside_down: false,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

impl Default for PanOrbitSettings {
    fn default() -> Self {
        PanOrbitSettings {
            pan_sensitivity: 0.001, // 1000 pixels per world unit
            orbit_sensitivity: 0.1f32.to_radians(), // 0.1 degree per pixel
            zoom_sensitivity: 0.01,
            pan_key: Some(KeyCode::ControlLeft),
            orbit_key: Some(KeyCode::AltLeft),
            zoom_key: Some(KeyCode::KeyZ),
            scroll_action: Some(PanOrbitAction::Zoom),
            scroll_line_sensitivity: 16.0, // 1 "line" == 16 "pixels of motion"
            scroll_pixel_sensitivity: 1.0,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = PanOrbitCameraBundle::default();
    // Position our camera using our component,
    // not Transform (it would get overwritten)
    if let bevy::prelude::Projection::Perspective(ref mut pp) = camera.camera.projection {
        pp.fov = CAMERA_FOV;
    }
    camera.state.center = Vec3::new(0.0, 0.0, 0.0);
    camera.state.radius = 400.0;
    camera.state.pitch = 0.0; //15.0f32.to_radians();
    camera.state.yaw = 0.0; //30.0f32.to_radians();
    commands.spawn(camera);
}

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};

use std::f32::consts::{FRAC_PI_2, PI, TAU};

fn pan_orbit_camera(
    kbd: Res<ButtonInput<KeyCode>>,
    mut evr_motion: EventReader<MouseMotion>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut q_camera: Query<(
        &PanOrbitSettings,
        &mut PanOrbitState,
        &mut Transform,
    )>,
) {
    // window.set_cursor_visibility(false);

    // First, accumulate the total amount of
    // mouse motion and scroll, from all pending events:
    let mut total_motion: Vec2 = evr_motion.read()
        .map(|ev| ev.delta).sum();

    // Reverse Y (Bevy's Worldspace coordinate system is Y-Up,
    // but events are in window/ui coordinates, which are Y-Down)
    total_motion.y = -total_motion.y;

    let mut total_scroll_lines = Vec2::ZERO;
    let mut total_scroll_pixels = Vec2::ZERO;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                total_scroll_lines.x += ev.x;
                total_scroll_lines.y -= ev.y;
            }
            MouseScrollUnit::Pixel => {
                total_scroll_pixels.x += ev.x;
                total_scroll_pixels.y -= ev.y;
            }
        }
    }

    for (settings, mut state, mut transform) in &mut q_camera {
        // Check how much of each thing we need to apply.
        // Accumulate values from motion and scroll,
        // based on our configuration settings.

        let mut total_pan = Vec2::ZERO;
        if settings.pan_key.map(|key| kbd.pressed(key)).unwrap_or(false) {
            total_pan -= total_motion * settings.pan_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Pan) {
            total_pan -= total_scroll_lines
                * settings.scroll_line_sensitivity * settings.pan_sensitivity;
            total_pan -= total_scroll_pixels
                * settings.scroll_pixel_sensitivity * settings.pan_sensitivity;
        }

        let mut total_orbit = Vec2::ZERO;
        if settings.orbit_key.map(|key| kbd.pressed(key)).unwrap_or(false) {
            total_orbit -= total_motion * settings.orbit_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Orbit) {
            total_orbit -= total_scroll_lines
                * settings.scroll_line_sensitivity * settings.orbit_sensitivity;
            total_orbit -= total_scroll_pixels
                * settings.scroll_pixel_sensitivity * settings.orbit_sensitivity;
        }

        let mut total_zoom = Vec2::ZERO;
        if settings.zoom_key.map(|key| kbd.pressed(key)).unwrap_or(false) {
            total_zoom -= total_motion * settings.zoom_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Zoom) {
            total_zoom -= total_scroll_lines
                * settings.scroll_line_sensitivity * settings.zoom_sensitivity;
            total_zoom -= total_scroll_pixels
                * settings.scroll_pixel_sensitivity * settings.zoom_sensitivity;
        }

        // Upon starting a new orbit maneuver (key is just pressed),
        // check if we are starting it upside-down
        if settings.orbit_key.map(|key| kbd.just_pressed(key)).unwrap_or(false) {
            state.upside_down = state.pitch < -FRAC_PI_2 || state.pitch > FRAC_PI_2;
        }

        // If we are upside down, reverse the X orbiting
        if state.upside_down {
            total_orbit.x = -total_orbit.x;
        }

        // Now we can actually do the things!

        let mut any = false;

        // To ZOOM, we need to multiply our radius.
        if total_zoom != Vec2::ZERO {
            any = true;
            // in order for zoom to feel intuitive,
            // everything needs to be exponential
            // (done via multiplication)
            // not linear
            // (done via addition)

            // so we compute the exponential of our
            // accumulated value and multiply by that
            state.radius *= (-total_zoom.y).exp();
        }

        // To ORBIT, we change our pitch and yaw values
        if total_orbit != Vec2::ZERO {
            any = true;
            state.yaw += total_orbit.x;
            state.pitch -= total_orbit.y;
            // wrap around, to stay between +- 180 degrees
            if state.yaw > PI {
                state.yaw -= TAU; // 2 * PI
            }
            if state.yaw < -PI {
                state.yaw += TAU; // 2 * PI
            }
            if state.pitch > PI {
                state.pitch -= TAU; // 2 * PI
            }
            if state.pitch < -PI {
                state.pitch += TAU; // 2 * PI
            }
        }

        // To PAN, we can get the UP and RIGHT direction
        // vectors from the camera's transform, and use
        // them to move the center point. Multiply by the
        // radius to make the pan adapt to the current zoom.
        if total_pan != Vec2::ZERO {
            any = true;
            let radius = state.radius;
            state.center += transform.right() * total_pan.x * radius;
            state.center += transform.up() * total_pan.y * radius;
        }

        // Finally, compute the new camera transform.
        // (if we changed anything, or if the pan-orbit
        // controller was just added and thus we are running
        // for the first time and need to initialize)
        if any || state.is_added() {
            // YXZ Euler Rotation performs yaw/pitch/roll.
            transform.rotation =
                Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
            // To position the camera, get the backward direction vector
            // and place the camera at the desired radius from the center.
            transform.translation = state.center + transform.back() * state.radius;
        }
    }
}


/*
    Constants
*/

pub const VIRT_ZOOM: f64 = 10.0;

pub const COLOR_PARTICLE: Color = Color::Srgba(WHITE_SMOKE);
pub const SIZE_PARTICLE: f32 = 5.0;
pub const LIGHT_STRENGTH: f32 = 5000.0;
pub const LIGHT_COLOR: Color = Color::Srgba(WHITE);

pub const GIZMOS_AXES_LENGTH: f32 = 100.0;

pub const CAMERA_FOV: f32 = 1.2;

pub const SIM_DT: f64 = 0.001;

/*
    Resources
*/

#[derive(Resource)]
struct ChaosEquationResource {
    pub eq: chaos::ChaosEq, 
    pub steps: u8,
    pub dt_mult: f32,
}

impl FromWorld for ChaosEquationResource {
    fn from_world(_world: &mut World) -> Self {
        return ChaosEquationResource {
            eq: chaos_equations::lorenz_attractor_equation,
            steps: 1,
            dt_mult: 2.5,
        };
    }
}

#[derive(Resource)]
pub struct CubeMeshMaterial(pub Handle<Mesh>, pub Handle<StandardMaterial>);

impl FromWorld for CubeMeshMaterial {
    fn from_world(world: &mut World) -> Self {
        let mesh = world.get_resource_mut::<Assets<Mesh>>().unwrap().add(Cuboid::new(SIZE_PARTICLE, SIZE_PARTICLE, SIZE_PARTICLE));
        let material =  world.get_resource_mut::<Assets<StandardMaterial>>().unwrap().add(COLOR_PARTICLE);
        return CubeMeshMaterial(mesh,material);
    }
}

#[derive(Resource)]
pub struct OneShotSystems{pub despawn_particles: SystemId}

impl FromWorld for OneShotSystems {
    fn from_world(world: &mut World) -> Self {
        let despawn = world.register_system(despawn_all_particles);
        return OneShotSystems {
            despawn_particles: despawn
        };

    }
}

#[derive(Resource)]
pub struct ParticleCount(pub i32);

impl FromWorld for ParticleCount {
    fn from_world(_world: &mut World) -> Self {
        return ParticleCount(0);
    }
}

// #[derive(Resource)]
// pub struct SimSettings;

/*
    Components
*/

#[derive(Component)]
enum DisplayText {
    Fps,
    ParticleCount,
    DeltaTime,
    StepsPerFrame,
}

#[derive(Component)]
struct Particle(pub chaos::Coord);

/*
    Bundles
*/

#[derive(Bundle)]
pub struct ParticleBundle {
    particle: Particle,
    pbr_bundle: PbrBundle,
}
impl ParticleBundle {
    pub fn from_world_xy(x: f32, y: f32, asset: &CubeMeshMaterial)->Self {
        return Self {
            particle: Particle(world_to_virt_coord(x, y, 0.0)),
            pbr_bundle: PbrBundle {
                mesh: asset.0.clone(),
                material: asset.1.clone(),
                transform: Transform::from_xyz(x, y, 1.0),
                ..default()
            }
        };
    }
    pub fn from_coord(c: chaos::Coord, asset: &CubeMeshMaterial)->Self {
        return Self {
            particle: Particle(c),
            pbr_bundle: PbrBundle {
                mesh: asset.0.clone(),
                material: asset.1.clone(),
                transform: virt_to_world(&c),
                ..default()
            }
        };
    }
}

/*
    Systems
*/

fn draw_axes(mut gizmos: Gizmos) {
    gizmos.axes(Transform::default(), GIZMOS_AXES_LENGTH);
}

fn mouse_click_system(
    mut cmd: Commands, 
    q_window: Query<&Window, With<PrimaryWindow>>,
    cube_mesh_material: Res<CubeMeshMaterial>,
    q_camera: Query<(
        &PanOrbitState,
        &Transform,
    )>,
    mut pc: ResMut<ParticleCount>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let bunch_spawn = keys.pressed(KeyCode::ShiftLeft);
    let window = q_window.single();
    let mut rng = rand::thread_rng();
    
    for (state, transform) in &q_camera {

        if let Some(world_position) = window.cursor_position() {
            let half_height = window.height()/2.0;
            let half_width = window.width()/2.0;
            let px = (world_position.x / half_width) - 1.0;
            let py = (world_position.y / half_height) - 1.0;
            let dx = state.radius * (CAMERA_FOV / 2.0).tan() * px * (half_width/half_height);
            let dy = state.radius * (CAMERA_FOV / 2.0).tan() * py; //fov is vertical AND half of CAMERA_FOV (half screen, it's weird)
            let spawn_at = state.center + transform.down()*dy + transform.right()*dx;
            if bunch_spawn {
                for _ in 0..20 {
                    // cmd.spawn(ParticleBundle::from_coord(world_to_virt_coord(2.0, 1.0, 1.0)));
                    let dx = rng.gen::<f64>().fract();
                    let dy = rng.gen::<f64>().fract();
                    let dz = rng.gen::<f64>().fract();
                    cmd.spawn(ParticleBundle::from_coord(
                        chaos::Coord{x:dx,y:dy,z:dz} + world_to_virt_coord(spawn_at.x, spawn_at.y, spawn_at.z),
                        &cube_mesh_material
                    ));
                }
                pc.0 += 20;
            } else {
                cmd.spawn(ParticleBundle::from_coord(world_to_virt_coord(spawn_at.x, spawn_at.y, spawn_at.z), &cube_mesh_material));
                pc.0 += 1;
            }
        }
    }
}

fn vmove_particle_system(mut particles: Query<&mut Particle>, chaos_eq: Res<ChaosEquationResource>) {
    for mut particle in &mut particles {
        for _ in 0..chaos_eq.steps {
            particle.0 = (chaos_eq.eq)(&particle.0, SIM_DT * (f32::powf(2.0, chaos_eq.dt_mult)) as f64);
        }
    }
}

fn transform_particle_system(mut particles: Query<(&Particle, &mut Transform)>) {
    for (particle, mut transform) in &mut particles {
        virt_to_world_mut(&particle.0, &mut transform);
    }
}

fn init_lighting(mut cmd: Commands) {
    // ambient light
    cmd.insert_resource(AmbientLight {
        color: LIGHT_COLOR.into(),
        brightness: LIGHT_STRENGTH,
    });
}

fn init_text(mut cmd: Commands) {
    cmd.spawn((DisplayText::Fps, TextBundle {
        text: Text::from_section("--", TextStyle::default()),
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
        ..default()
    }));
    cmd.spawn((DisplayText::ParticleCount, TextBundle {
        text: Text::from_section("--", TextStyle::default()),
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(25.0),
            left: Val::Px(5.0),
            ..default()
        },
        ..default()
    }));
    cmd.spawn((DisplayText::DeltaTime, TextBundle {
        text: Text::from_section("--", TextStyle::default()),
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(45.0),
            left: Val::Px(5.0),
            ..default()
        },
        ..default()
    }));
    cmd.spawn((DisplayText::StepsPerFrame, TextBundle {
        text: Text::from_section("--", TextStyle::default()),
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(65.0),
            left: Val::Px(5.0),
            ..default()
        },
        ..default()
    }));
}

fn keybind_listener(mut cmd: Commands, keys: Res<ButtonInput<KeyCode>>, rem_particles: Res<OneShotSystems>, mut eq: ResMut<ChaosEquationResource>) {
    if keys.just_pressed(KeyCode::KeyC) {
        cmd.run_system(rem_particles.despawn_particles);
    }
    if keys.just_pressed(KeyCode::Digit1) {
        eq.eq = chaos::basic_equation;
    } else if keys.just_pressed(KeyCode::Digit2) {
        eq.eq = chaos::lorenz_attractor_equation;
    }
    if keys.just_pressed(KeyCode::NumpadAdd) {
        eq.steps += 1;
    } else if keys.just_pressed(KeyCode::NumpadSubtract) {
        if eq.steps != 0 {
            eq.steps -= 1;
        }
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        eq.dt_mult += 0.2;
    } else if keys.just_pressed(KeyCode::BracketLeft) {
        eq.dt_mult -= 0.2;
    }
}

//some more stolen code for displaying fps
fn display_stats(diagnostics: Res<DiagnosticsStore>, mut dtexts: Query<(&mut Text, &DisplayText)>, pc: Res<ParticleCount>, eq: Res<ChaosEquationResource>) {
    for (mut text, text_type) in &mut dtexts {
        match text_type {
            DisplayText::Fps => {
                // try to get a "smoothed" FPS value from Bevy
                if let Some(value) = diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                {
                    // Format the number as to leave space for 4 digits, just in case,
                    // right-aligned and rounded. This helps readability when the
                    // number changes rapidly.
                    text.sections[0].value = format!("{value:>4.0}fps");

                    // Let's make it extra fancy by changing the color of the
                    // text according to the FPS value:
                    text.sections[0].style.color = if value >= 120.0 {
                        // Above 120 FPS, use green color
                        Color::Srgba(GREEN)
                    } else if value >= 60.0 {
                        // Between 60-120 FPS, gradually transition from yellow to green
                        Color::srgb(
                            (1.0 - (value - 60.0) / (120.0 - 60.0)) as f32,
                            1.0,
                            0.0,
                        )
                    } else if value >= 30.0 {
                        // Between 30-60 FPS, gradually transition from red to yellow
                        Color::srgb(
                            1.0,
                            ((value - 30.0) / (60.0 - 30.0)) as f32,
                            0.0,
                        )
                    } else {
                        // Below 30 FPS, use red color
                        Color::Srgba(RED)
                    }
                } else {
                    // display "N/A" if we can't get a FPS measurement
                    // add an extra space to preserve alignment
                    text.sections[0].value = " N/A".into();
                    text.sections[0].style.color = Color::WHITE;
                }
            }
            DisplayText::ParticleCount => {
                let pnum = pc.0;
                text.sections[0].value = format!("{} particles", pnum);
            }
            DisplayText::StepsPerFrame => {
                let pnum = eq.steps;
                text.sections[0].value = format!("{} steps per frame", pnum);
            }
            DisplayText::DeltaTime => {
                let pnum = f32::powf(2.0, eq.dt_mult);
                text.sections[0].value = format!("dt={pnum:.2}", );
            }
        }
    }
}

fn despawn_all_particles(
    mut cmd: Commands,
    query: Query<Entity, With<Particle>>,
    mut pc: ResMut<ParticleCount>,
) {
    pc.0 = 0;
    for entity in query.iter() {
        cmd.entity(entity).despawn();
    }
}

/*
    Plugins
*/

pub struct ChaosPlugin;

impl Plugin for ChaosPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChaosEquationResource>()
            .init_resource::<CubeMeshMaterial>()
            .init_resource::<OneShotSystems>()
            .init_resource::<ParticleCount>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Startup, init_lighting)
            .add_systems(Startup, init_text)
            .add_systems(FixedUpdate, vmove_particle_system)
            .add_systems(Update, transform_particle_system)
            .add_systems(Update, mouse_click_system.run_if(input_pressed(MouseButton::Left)))
            .add_systems(Update, draw_axes)
            .add_systems(Update, keybind_listener)
            .add_systems(Update, display_stats)
            .add_systems(Update,
                pan_orbit_camera
                    .run_if(any_with_component::<PanOrbitState>),
            );
    }
}

/*
    Helper functions
*/

fn world_to_virt_coord(x: f32, y: f32, z: f32)->chaos::Coord {
    return chaos::Coord {
        x: x as f64 / VIRT_ZOOM,
        y: y as f64 / VIRT_ZOOM,
        z: z as f64 / VIRT_ZOOM,
    };
}

fn _world_to_virt(t: &Transform)->chaos::Coord {
    return chaos::Coord {
        x: t.translation.x as f64 / VIRT_ZOOM,
        y: t.translation.y as f64 / VIRT_ZOOM,
        z: t.translation.z as f64 / VIRT_ZOOM,
    };
}

fn virt_to_world(c: &chaos::Coord)->Transform {
    return Transform::from_xyz((c.x * VIRT_ZOOM) as f32, (c.y * VIRT_ZOOM) as f32, (c.z * VIRT_ZOOM) as f32);
}

fn virt_to_world_mut(c: &chaos::Coord, t: &mut Transform) {
    t.translation.x = (c.x * VIRT_ZOOM) as f32;
    t.translation.y = (c.y * VIRT_ZOOM) as f32;
    t.translation.z = (c.z * VIRT_ZOOM) as f32;
}

/*
    Main
*/

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,ChaosPlugin,FrameTimeDiagnosticsPlugin))
        .run();
}