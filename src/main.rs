use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub, SubAssign};

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

const BACKGROUND_COLOR: Color = Color::BLACK;
const DELTA_TIME: f32 = 0.02;
const GRAVITATIONAL_CONSTANT: f32 = 100000.0; //does this matter?
// const MAX_ACCELERATION: f32 = 5000.0;
const MIN_DISTANCE: f32 = 300.0;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (
            new_line,
            first_half_step_velocity,
            continuous_collision_detection,
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

#[derive(Component, Deref, DerefMut)] //don't need DerefMut?
struct Radius(f32);

#[derive(Component, Deref, DerefMut, Debug)]
struct Force(Vec2);

#[derive(Component)]
struct Acceleration {
    current: Vec2,
    previous: Vec2,
}

#[derive(Component)]
struct CentreOfMass;

#[derive(Component, Deref, DerefMut, Debug)]
struct Velocity(Vec2);

#[derive(Component, Deref, DerefMut, Debug)]
struct PreviousPosition(Vec2);

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

    //colliding two body next to each other
    // spawn_body(&mut commands, &mut meshes, &mut materials, 100.0, Transform::from_xyz(50.0, 0.0, 0.0), 40.0, Vec2::new(0.0, 0.0), Color::WHITE);
    // spawn_body(&mut commands, &mut meshes, &mut materials, 100.0, Transform::from_xyz(-50.0, 0.0, 0.0), 40.0, Vec2::new(0.0, 0.0), Color::srgb(1.0, 0.5, 0.1));

    //colliding two body far away
    // spawn_body(&mut commands, &mut meshes, &mut materials, 100.0, Transform::from_xyz(300.0, 0.0, 0.0), 40.0, Vec2::new(0.0, 0.0), Color::WHITE);
    // spawn_body(&mut commands, &mut meshes, &mut materials, 100.0, Transform::from_xyz(-300.0, 0.0, 0.0), 40.0, Vec2::new(0.0, 0.0), Color::srgb(1.0, 0.5, 0.1));


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
            transform: transform.with_scale(Vec2::splat(radius * 2.0).extend(1.0)),
            ..default()
        },
        Body,
        Mass(mass),
        Radius(radius),
        Force(Vec2::ZERO),
        Acceleration { current: Vec2::ZERO, previous: Vec2::ZERO },
        Velocity(initial_velocity),
        PreviousPosition(transform.translation.truncate()),
    ));
}

fn new_line() { println!(); }

fn calculate_gravitational_force(mut query: Query<(&mut Force, &Transform, &Mass), With<Body>>) {
    for (mut force, _, _) in &mut query { force.0 = Vec2::ZERO; }

    let mut iter = query.iter_combinations_mut();
    while let Some([(mut force, transform, mass), (mut other_force, other_transform, other_mass)]) =
        iter.fetch_next()
    {
        let displacement = other_transform.translation.sub(transform.translation).truncate();
        let distance = displacement.length();//.max(MIN_DISTANCE);
        let force_magnitude = GRAVITATIONAL_CONSTANT * ((mass.0 * other_mass.0) / (distance * distance * distance));
        let gravitational_force = displacement.mul(force_magnitude);
        force.add_assign(gravitational_force);
        other_force.sub_assign(gravitational_force);
        // println!("f1: {force:?}, f2: {other_force:?}");
    }
}

fn apply_force(mut query: Query<(&mut Acceleration, &Mass, &Force)>) {
    for (mut acceleration, mass, force) in &mut query {
        acceleration.previous = acceleration.current;
        acceleration.current = force.div(mass.0) //.clamp_length_max(MAX_ACCELERATION);
    }
}

fn first_half_step_velocity(mut query: Query<(&mut Transform, &mut PreviousPosition, &Velocity, &Acceleration)>) {
    for (mut transform, mut previous_position, velocity, acceleration) in &mut query {
        previous_position.0 = transform.translation.truncate();
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
    mut centre_of_mass_query: Query<&mut Transform, With<CentreOfMass>>,
    body_query: Query<(&Transform, &Mass), Without<CentreOfMass>>
) {
    let mut centre_of_mass = Vec2::ZERO;
    let mut total_mass: f32 = 0.0;

    for (transform, mass) in &body_query {
        centre_of_mass.add_assign(transform.translation.truncate().mul(mass.0));
        total_mass += mass.0;
    }
    centre_of_mass.div_assign(total_mass);

    let mut transform = centre_of_mass_query.single_mut();
    transform.translation = centre_of_mass.extend(0.0);
}

fn continuous_collision_detection(
    mut query: Query<(&Radius, &Mass, &PreviousPosition, &mut Transform, &mut Velocity), With<Body>>
) {
    let mut iter = query.iter_combinations_mut();
    while let Some([
                   (radius_1, mass_1, previous_position_1, mut transform_1, mut velocity_1),
                   (radius_2, mass_2, previous_position_2, mut transform_2, mut velocity_2)
                   ]) =
        iter.fetch_next()
    {
        //get time of collision
        let relative_position = previous_position_1.0.sub(previous_position_2.0);
        let relative_velocity = velocity_1.0.sub(velocity_2.0);

        let a = relative_velocity.dot(relative_velocity);
        let b = 2.0 * relative_position.dot(relative_velocity);
        let c = relative_position.dot(relative_position) - ((radius_1.0 + radius_2.0) * (radius_1.0 + radius_2.0));

        let discriminant = (b * b) - (4.0 * a * c);
        if discriminant < 0.0 { continue; }

        let discriminant_sqrt = discriminant.sqrt();
        let t_a = (-b + discriminant_sqrt) / (2.0 * a);
        let t_b = (-b - discriminant_sqrt) / (2.0 * a);

        println!("t_a: {t_a}, t_b: {t_b}");
        println!("before: prev_pos_1 {previous_position_1:?} prev_pos_2 {previous_position_2:?}");



        let t_collision = t_a.min(t_b);
        // if t_a > 0.0 && t_b < 0.0 { t_collision = t_a; }
        // else if t_a < 0.0 && t_b > 0.0 { t_collision = t_b; }
        // else if t_a < 0.0 && t_b < 0.0 { continue; }
        // else { t_collision = t_a.min(t_b); }
        if t_collision.abs() <= DELTA_TIME { println!("collision!") } else { continue; }

        //resolve

        //track to point of collision
        // println!("before: prev_pos_1 {previous_position_1:?} prev_pos_2 {previous_position_2:?}");
        print_transform_translations("before", &mut transform_1, &mut transform_2);
        transform_1.translation = previous_position_1.0.add(velocity_1.0.mul(t_collision)).extend(0.0);
        transform_2.translation = previous_position_2.0.add(velocity_2.0.mul(t_collision)).extend(0.0);
        print_transform_translations("after", &mut transform_1, &mut transform_2);

        if previous_position_1.x.abs() < transform_1.translation.x.abs() { println!("fudge"); }

        //get new velocities
        let position_difference = transform_1.translation.sub(transform_2.translation).truncate();
        let position_difference_squared_distance = position_difference.length_squared();
        let stuff = position_difference_squared_distance.sqrt();
        println!("dist: {stuff}");

        let velocity_difference = velocity_1.0.sub(velocity_2.0);
        let total_mass = mass_1.0 + mass_2.0;

        let velocity_1_response_magnitude = (2.0 * mass_2.0 * velocity_difference.dot(position_difference)) / (total_mass * position_difference_squared_distance);
        let velocity_2_response_magnitude = (2.0 * mass_1.0 * (-velocity_difference).dot(-position_difference)) / (total_mass * position_difference_squared_distance);

        println!("before: velocity_1 {velocity_1:?} velocity_2 {velocity_2:?}");
        velocity_1.sub_assign(position_difference.mul(velocity_1_response_magnitude));
        velocity_2.sub_assign((-position_difference).mul(velocity_2_response_magnitude));
        println!("after: velocity_1 {velocity_1:?} velocity_2 {velocity_2:?}");


        //move forward for rest of time step
        let t_remaining = DELTA_TIME - t_collision;
        println!("{t_remaining}");
        transform_1.translation.add_assign(velocity_1.0.mul(t_remaining).extend(0.0));
        transform_2.translation.add_assign(velocity_2.0.mul(t_remaining).extend(0.0));
        print_transform_translations("final", &mut transform_1, &mut transform_2);

    }
}

fn print_transform_translations(when: &str, transform_1: &mut Mut<Transform>, transform_2: &mut Mut<Transform>) {
    let t_1 = transform_1.translation;
    let t_2 = transform_2.translation;
    println!("{when}: transform_1 {t_1:?} transform_2 {t_2:?}");
}