#![allow(unused)]

use bevy_ecs::prelude as bevy;
use worlds_ecs::entity::EntityId;
use worlds_ecs::prelude::*;

#[derive(Component, bevy::Component)]
struct A(usize);

#[derive(Component, bevy::Component)]
struct B(usize);

#[derive(Component, bevy::Component)]
struct C(usize);

#[derive(Component, bevy::Component)]
struct D(usize);

fn main() {
    let mut bevy_world = bevy::World::default();
    let mut world = World::default();

    println!(" \n ");

    compare_spawning_entities(&mut bevy_world, &mut world, 10_000_000);
    compare_querying(&mut bevy_world, &mut world);
}

fn compare_spawning_entities(
    bevy_world: &mut bevy::World,
    world: &mut World,
    amount_to_spawn: usize,
) {
    // Spawn Bench 1
    compare_code_blocks! {
        { (0..amount_to_spawn).for_each(|i| {
            bevy_world.spawn((A(i), B(i), C(i), D(i)));
        }) },

         { (0..amount_to_spawn).for_each(|i| {
            world.spawn((A(i), B(i), C(i), D(i)));
        })}, "Spawn bench 1"
    }

    // Spawn Bench 1
    compare_code_blocks! {
        { (0..amount_to_spawn).for_each(|i| {
            bevy_world.spawn((B(i), D(i)));
        }) },

         { (0..amount_to_spawn).for_each(|i| {
            world.spawn((B(i), D(i)));
        })}, "Spawn bench 2"
    }
}

fn compare_querying(bevy_world: &mut bevy::World, world: &mut World) {
    println!(" \n ");
    // Query Bench 1
    compare_code_blocks! {
        {
            bevy_world
                .query::<&A>()
                .iter(bevy_world)
                .for_each(|_| {});
        },
        {
            world.query::<&A>().for_each(|_| {});
        },
        "Query bench 1"
    }

    // Query Bench 2
    compare_code_blocks! {
        {
            bevy_world
                .query::<(&A, &mut B)>()
                .iter(bevy_world)
                .for_each(|_| {});
        },
        {
            world.query::<(&A, &mut B)>().for_each(|_| {});
        },
        "Query bench 2"
    }

    // Query Bench 3
    compare_code_blocks! {
        {
            bevy_world
                .query::<(&A, &mut B, &C, Option<&D>)>()
                .iter(bevy_world)
                .for_each(|_| {});
        },
        {
            world.query::<(&A, &mut B, &C, Option<&D>)>().for_each(|_| {});
        },
        "Query bench 3"
    }

    // Query Bench 4
    compare_code_blocks! {
        {
            bevy_world
                .query_filtered::<(bevy::Entity, &mut B), bevy::Without<A>>()
                .iter(bevy_world)
                .for_each(|_| {});
        },
        {
            world.query_filtered::<(EntityId, &mut B), Not<Has<A>>>().for_each(|_| {});
        },
        "Query bench 4"
    }
}

#[macro_export]
macro_rules! compare_code_blocks {
    ($bevy:block, $worlds:block, $msg:literal) => {

        println!("|  {}  |", $msg);

        let bevy_instant = std::time::Instant::now();
        $bevy
        let bevy_time = bevy_instant.elapsed();
        println!("\t Bevy: {:?}", bevy_time);

        let worlds_instant = std::time::Instant::now();
        $worlds
        let worlds_time = worlds_instant.elapsed();
        println!("\t Worlds: {:?}", worlds_time);
        println!("  RATIO: {} (worlds / bevy)  ", worlds_time.as_nanos() / bevy_time.as_nanos());

        println!("  {}  ", "-".repeat($msg.len()));
    };
}
