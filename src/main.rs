use bevy::input::common_conditions::{input_just_pressed, input_pressed};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, window::PrimaryWindow};
use bevy::color::palettes::css::*;
use rand::Rng;

mod chaos_equations;

use chaos_equations as chaos;

/*
    Constants
*/

pub const COLOR_PARTICLE: Color = Color::Srgba(WHITE_SMOKE);
pub const SIZE_PARTICLE: f32 = 5.0;

/*
    Resources
*/

#[derive(Resource)]
struct ChaosEquationResource(pub chaos::ChaosEq);

impl FromWorld for ChaosEquationResource {
    fn from_world(_world: &mut World) -> Self {
        return ChaosEquationResource(chaos_equations::lorenz_attractor_equation);
    }
}

/*
    Components
*/

#[derive(Component, Default)]
struct MainCamera;

#[derive(Component)]
struct Particle(pub chaos::Coord);
//sprite_bundle: SpriteBundle,

/*
    Bundles
*/

#[derive(Bundle)]
pub struct ParticleBundle {
    particle: Particle,
    sprite_bundle: SpriteBundle,
}
impl ParticleBundle {
    pub fn from_world_xy(x: f32, y: f32)->Self {
        return Self {
            particle: Particle(world_to_virt_coord(x, y, 0.0)),
            sprite_bundle: SpriteBundle {
                sprite: Sprite { color: COLOR_PARTICLE, custom_size: Some(Vec2::splat(SIZE_PARTICLE)), ..default() },
                transform: Transform::from_xyz(x, y, 1.0),
                ..default()
            },
        };
    }
    pub fn from_coord(c: chaos::Coord)->Self {
        return Self {
            particle: Particle(c),
            sprite_bundle: SpriteBundle {
                sprite: Sprite { color: COLOR_PARTICLE, custom_size: Some(Vec2::splat(SIZE_PARTICLE)), ..default() },
                transform: virt_to_world(&c),
                ..default()
            },
        };
    }
}

#[derive(Bundle, Default)]
pub struct MainCameraBundle {
    main_cam: MainCamera,
    sprite_bundle: Camera2dBundle,
}

/*
    Systems
*/

fn draw_axes(mut gizmos: Gizmos) {
    gizmos.axes(Transform::default(), 100.0);
}

fn mouse_click_system(
    mut cmd: Commands, 
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();
    let mut rng = rand::thread_rng();

    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.xy())
    {
        for _ in 0..20 {
            // cmd.spawn(ParticleBundle::from_coord(world_to_virt_coord(2.0, 1.0, 1.0)));
            let dx = rng.gen::<f32>().fract();
            let dy = rng.gen::<f32>().fract();
            let dz = rng.gen::<f32>().fract();
            cmd.spawn(ParticleBundle::from_coord(world_to_virt_coord(world_position.x + dx, world_position.y + dy, dz)));
        }
    }
}

fn vmove_particle_system(mut particles: Query<&mut Particle>, chaos_eq: Res<ChaosEquationResource>) {
    for mut particle in &mut particles {
        particle.0 = chaos_eq.0(&particle.0,0.004);
    }
}

fn transform_particle_system(mut particles: Query<(&Particle, &mut Transform)>) {
    for (particle, mut transform) in &mut particles {
        virt_to_world_mut(&particle.0, &mut transform);
    }
}

fn init_camera_system(mut cmd: Commands) {
    cmd.spawn(MainCameraBundle::default());
}

/*
    Plugins
*/

pub struct ChaosPlugin;

impl Plugin for ChaosPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChaosEquationResource>()
            .add_systems(Startup, init_camera_system)
            .add_systems(Update, vmove_particle_system)
            .add_systems(Update, transform_particle_system)
            .add_systems(Update, mouse_click_system.run_if(input_just_pressed(MouseButton::Left)))
            .add_systems(Update, draw_axes);
    }
}

/*
    Helper functions
*/

const ZOOM: f64 = 10.0;

fn world_to_virt_coord(x: f32, y: f32, z: f32)->chaos::Coord {
    return chaos::Coord {
        x: x as f64 / ZOOM,
        y: y as f64 / ZOOM,
        z: z as f64 / ZOOM,
    };
}

fn world_to_virt(t: &Transform)->chaos::Coord {
    return chaos::Coord {
        x: t.translation.x as f64 / ZOOM,
        y: t.translation.y as f64 / ZOOM,
        z: 0.0
    };
}

fn virt_to_world(c: &chaos::Coord)->Transform {
    return Transform::from_xyz((c.x * ZOOM) as f32, (c.y * ZOOM) as f32, 1.0);
}

fn virt_to_world_mut(c: &chaos::Coord, t: &mut Transform) {
    t.translation.x = (c.x * ZOOM) as f32;
    t.translation.y = (c.y * ZOOM) as f32;
}

/*
    Main
*/

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,ChaosPlugin,FrameTimeDiagnosticsPlugin))
        .run();
}