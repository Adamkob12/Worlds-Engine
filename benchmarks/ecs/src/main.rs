#![allow(unused)]

use bevy_ecs_1::prelude as bevy1;
use bevy_ecs_13 as bevy_ecs;
use bevy_ecs_13::prelude as bevy13;
use worlds_ecs::entity::EntityId;
use worlds_ecs::prelude::*;

#[derive(Component, bevy13::Component)]
struct A(usize);

#[derive(Component, bevy13::Component)]
struct B(usize);

#[derive(Component, bevy13::Component)]
struct C(usize);

#[derive(Component, bevy13::Component)]
struct D(usize);

#[derive(Component, bevy13::Component)]
struct E(usize);

#[derive(Component, bevy13::Component)]
struct F(usize);

#[derive(Component, bevy13::Component)]
struct G(usize);

#[derive(Component, bevy13::Component)]
struct H(usize);

fn main() {
    let mut bevy_world = bevy13::World::default();
    let mut world = World::default();
    let mut bevy1_world = bevy1::World::default();

    println!(" \n ");

    compare_spawning_entities(&mut bevy_world, &mut bevy1_world, &mut world, 200_000);
    compare_querying(&mut bevy_world, &mut bevy1_world, &mut world);
}

fn compare_spawning_entities(
    bevy_world: &mut bevy13::World,
    bevy1_world: &mut bevy1::World,
    world: &mut World,
    amount_to_spawn: usize,
) {
    // Spawn Bench 1
    compare_code_blocks! {
        { (0..amount_to_spawn).for_each(|i| {
            bevy_world.spawn((A(i), B(i), C(i), D(i)));
        }) },
        { (0..amount_to_spawn).for_each(|i| {
            bevy1_world.spawn((A(i), B(i), C(i), D(i)));
        })},
         { (0..amount_to_spawn).for_each(|i| {
            world.spawn((A(i), B(i), C(i), D(i)));
        })},

         "Spawn bench 1"
    }

    // Spawn Bench 2
    compare_code_blocks! {
        { (0..amount_to_spawn).for_each(|i| {
            bevy_world.spawn((B(i), D(i)));
        }) },
        { (0..amount_to_spawn).for_each(|i| {
            bevy_world.spawn((B(i), D(i)));
        }) },
         { (0..amount_to_spawn).for_each(|i| {
            world.spawn((B(i), D(i)));
        })},

         "Spawn bench 2"
    }

    // Spawn Bench 3
    compare_code_blocks! {
        { (0..amount_to_spawn).for_each(|_| {
            bevy_world.spawn((A(0), B(0), C(0), D(0), E(0), F(0), G(0), H(0)));
        }) },
        { (0..amount_to_spawn).for_each(|_| {
            bevy_world.spawn((A(0), B(0), C(0), D(0), E(0), F(0), G(0), H(0)));
        }) },
         { (0..amount_to_spawn).for_each(|_| {
            world.spawn((A(0), B(0), C(0), D(0), E(0), F(0), G(0), H(0)));
        })},

         "Spawn bench 3"
    }
}

fn compare_querying(
    bevy_world: &mut bevy13::World,
    bevy1_world: &mut bevy1::World,
    world: &mut World,
) {
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
            bevy1_world
                .query::<&A>()
                .iter()
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
            bevy1_world
                .query::<(&A, &mut B)>()
                .iter()
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
            bevy1_world
                .query::<(&A, &mut B, &C, Option<&D>)>()
                .iter()
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
                .query_filtered::<(bevy13::Entity, &mut B), bevy13::Without<A>>()
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
        println!("\t Bevy ECS 0.13 \t: {:?}", bevy_time);

        let worlds_instant = std::time::Instant::now();
        $worlds
        let worlds_time = worlds_instant.elapsed();
        println!("\t Worlds ECS  \t: {:?}", worlds_time);

        println!("  RATIO: {} (worlds / bevy)  ", worlds_time.as_nanos() / bevy_time.as_nanos());
        println!("  {}  ", "-".repeat($msg.len()));
    };

    ($bevy:block, $bevy1:block, $worlds:block, $msg:literal) => {

        println!("|  {}  |", $msg);

        let bevy_instant = std::time::Instant::now();
        $bevy
        let bevy_time = bevy_instant.elapsed();
        println!("\t Bevy ECS 0.13 \t: {:?}", bevy_time);

        let bevy1_instant = std::time::Instant::now();
        $bevy1
        let bevy1_time = bevy_instant.elapsed();
        println!("\t Bevy ECS 0.1 \t: {:?}", bevy1_time);

        let worlds_instant = std::time::Instant::now();
        $worlds
        let worlds_time = worlds_instant.elapsed();
        println!("\t Worlds ECS  \t: {:?}", worlds_time);

        println!("  RATIO: {} (worlds / bevy)  ", worlds_time.as_secs_f64() / bevy_time.as_secs_f64());
        println!("  RATIO: {} (worlds / bevy1)  ", worlds_time.as_secs_f64() / bevy1_time.as_secs_f64());
        println!("  {}  ", "-".repeat($msg.len()));
    }
}
