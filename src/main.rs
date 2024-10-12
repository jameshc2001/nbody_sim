use std::cmp::max;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub, SubAssign};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

const BACKGROUND_COLOR: Color = Color::BLACK;
const DELTA_TIME: f32 = 0.05;
const GRAVITATIONAL_CONSTANT: f32 = 100000.0; //does this matter?
// const MAX_ACCELERATION: f32 = 5000.0;
const MIN_DISTANCE: f32 = 100.0;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (
            first_half_step_velocity,
            calculate_gravitational_force,
            apply_force,
            final_half_step_velocity,
            update_center_of_mass,
        ).chain())
        .run();
}

#[derive(Component)]
struct Body;

#[derive(Component, Deref, DerefMut)] //don't need DerefMut?
struct Mass(f32);

#[derive(Component, Deref, DerefMut)]
struct Force(Vec2);

#[derive(Component)]
struct Acceleration {
    current: Vec2,
    previous: Vec2,
}

#[derive(Component)]
struct CentreOfMass;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(Color::srgb(1.0, 0.1, 0.1)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec2::splat(5.0).extend(1.0)),
            ..default()
        },
        CentreOfMass
    ));


    //classic two body problem
    // spawn_body(&mut commands, &mut meshes, &mut materials, 100.0, Transform::from_xyz(200.0, 0.0, 0.0), Vec2::splat(40.0), Vec2::new(0.0, 100.0));
    // spawn_body(&mut commands, &mut meshes, &mut materials, 100.0, Transform::from_xyz(-200.0, 0.0, 0.0), Vec2::splat(40.0), Vec2::new(0.0, -100.0));

    //three body
    spawn_body(&mut commands, &mut meshes, &mut materials, 50.0, Transform::from_xyz(250.0, 50.0, 0.0), 20.0, Vec2::ZERO, Color::WHITE);
    spawn_body(&mut commands, &mut meshes, &mut materials, 50.0, Transform::from_xyz(-200.0, 150.0, 0.0), 20.0, Vec2::ZERO, Color::srgb(1.0, 0.5, 0.1));
    spawn_body(&mut commands, &mut meshes, &mut materials, 50.0, Transform::from_xyz(-150.0, -150.0, 0.0), 20.0, Vec2::ZERO, Color::srgb(0.0, 1.0, 0.1));
}

fn spawn_body(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<ColorMaterial>>, mass: f32, transform: Transform, radius: f32, initial_velocity: Vec2, color: Color) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(color),
            transform: transform.with_scale(Vec2::splat(radius).extend(1.0)),
            ..default()
        },
        Body,
        Mass(mass),
        Force(Vec2::ZERO),
        Acceleration { current: Vec2::ZERO, previous: Vec2::ZERO },
        Velocity(initial_velocity)
    ));
}

fn calculate_gravitational_force(mut query: Query<(&mut Force, &Transform, &Mass), With<Body>>) {
    for (mut force, _, _) in &mut query { force.0 = Vec2::ZERO; }

    let mut iter = query.iter_combinations_mut();
    while let Some([(mut force, transform, mass), (mut other_force, other_transform, other_mass)]) =
        iter.fetch_next()
    {
        let displacement = other_transform.translation.sub(transform.translation).truncate();
        let distance = displacement.length().max(MIN_DISTANCE);
        let force_magnitude = GRAVITATIONAL_CONSTANT * ((mass.0 * other_mass.0) / (distance * distance * distance));
        let gravitational_force = displacement.mul(force_magnitude);
        force.add_assign(gravitational_force);
        other_force.sub_assign(gravitational_force);
    }
}

fn apply_force(mut query: Query<(&mut Acceleration, &Mass, &Force)>) {
    for (mut acceleration, mass, force) in &mut query {
        acceleration.previous = acceleration.current;
        acceleration.current = force.div(mass.0) //.clamp_length_max(MAX_ACCELERATION);
    }
}

fn apply_acceleration(mut query: Query<(&mut Velocity, &Acceleration)>) {
    for (mut velocity, acceleration) in &mut query {
        velocity.add_assign(acceleration.current.mul(DELTA_TIME));
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.add_assign(velocity.mul(DELTA_TIME).extend(0.0));
    }
}

fn first_half_step_velocity(mut query: Query<(&mut Transform, &Velocity, &Acceleration)>) {
    for (mut transform, velocity, acceleration) in &mut query {
        transform.translation.add_assign(
            velocity
                .mul(DELTA_TIME)
                .add(acceleration.current.mul(0.5 * DELTA_TIME * DELTA_TIME))
                .extend(0.0)
        );
    }
}

fn final_half_step_velocity(mut query: Query<(&mut Velocity, &Acceleration)>) {
    for (mut velocity, acceleration) in &mut query {
        velocity.add_assign(
            acceleration
                .current
                .add(acceleration.previous)
                .mul(0.5 * DELTA_TIME)
        );
    }
}

fn update_center_of_mass(
    mut centre_of_mass_query: Query<(&mut Transform), With<CentreOfMass>>,
    body_query: Query<(&Transform, &Mass), Without<CentreOfMass>>
) {
    let mut centre_of_mass = Vec2::ZERO;
    let mut total_mass: f32 = 0.0;

    for (mut transform, mass) in &body_query {
        centre_of_mass.add_assign(transform.translation.truncate().mul(mass.0));
        total_mass += mass.0;
    }
    centre_of_mass.div_assign(total_mass);

    let mut transform = centre_of_mass_query.single_mut();
    transform.translation = centre_of_mass.extend(0.0);
}