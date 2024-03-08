#![allow(missing_docs)] // TODO: Remove

pub mod arch_query;
pub mod query_data;
pub mod query_filter;

pub use arch_query::*;
pub use query_filter::*;

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[derive(Component)]
    struct A(usize);

    #[derive(Component)]
    struct B(String);

    #[derive(Component)]
    struct C(usize);

    #[test]
    fn test_basic_component_queries_1() {
        let mut world = World::default();

        world.spawn((A(1), B(String::from("Cart"))));
        world.spawn((A(2), B(String::from("Alice"))));
        world.spawn((A(3), B(String::from("James"))));

        world.spawn((C(1), B(String::from("Cart"))));
        world.spawn((C(2), B(String::from("Alice"))));
        world.spawn((C(3), B(String::from("James"))));

        let query_results = unsafe {
            <&B as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        assert_eq!(query_results.count(), 6);
    }

    #[test]
    fn test_basic_component_queries_2() {
        let mut world = World::default();

        let cart1 = world.spawn((A(1), B(String::from("Cart"))));
        let alice1 = world.spawn((A(2), B(String::from("Alice"))));
        let james1 = world.spawn((A(3), B(String::from("James"))));

        let cart2 = world.spawn((C(1), B(String::from("Cart"))));
        let alice2 = world.spawn((C(2), B(String::from("Alice"))));
        let james2 = world.spawn((C(3), B(String::from("James"))));

        let query_results = unsafe {
            <&B as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        let mut alice_count = 0;
        let mut james_count = 0;
        let mut cart_count = 0;

        for B(name) in query_results {
            if *name == "Alice" {
                alice_count += 1;
            }

            if *name == "James" {
                james_count += 1;
            }

            if *name == "Cart" {
                cart_count += 1;
            }
        }

        assert_eq!(alice_count, 2);
        assert_eq!(james_count, 2);
        assert_eq!(cart_count, 2);

        // Mutate the value and check

        let query_results = unsafe {
            <(&C, &mut B) as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        query_results.for_each(|(_, B(name))| {
            *name = String::from("BOO!");
        });

        let query_results = unsafe {
            <(&C, &B) as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        query_results.for_each(|(_, B(name))| {
            assert_eq!(*name, "BOO!");
        });

        assert_eq!(world.get_component::<B>(alice2).unwrap().0, "BOO!");
        assert_eq!(world.get_component::<B>(cart2).unwrap().0, "BOO!");
        assert_eq!(world.get_component::<B>(james2).unwrap().0, "BOO!");

        let query_results = unsafe {
            <(&A, &B) as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        query_results.for_each(|(_, B(name))| {
            assert_ne!(*name, "BOO!");
        });

        assert_eq!(world.get_component::<B>(alice1).unwrap().0, "Alice");
        assert_eq!(world.get_component::<B>(cart1).unwrap().0, "Cart");
        assert_eq!(world.get_component::<B>(james1).unwrap().0, "James");

        let query_results = unsafe {
            <&mut B as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        query_results.for_each(|B(name)| {
            *name = String::from("Hej!");
        });

        assert_eq!(world.get_component::<B>(alice1).unwrap().0, "Hej!");
        assert_eq!(world.get_component::<B>(cart1).unwrap().0, "Hej!");
        assert_eq!(world.get_component::<B>(james1).unwrap().0, "Hej!");

        assert_eq!(world.get_component::<B>(alice2).unwrap().0, "Hej!");
        assert_eq!(world.get_component::<B>(cart2).unwrap().0, "Hej!");
        assert_eq!(world.get_component::<B>(james2).unwrap().0, "Hej!");
    }

    #[test]
    #[should_panic]
    fn test_panic_on_duplicate_access_in_query() {
        let mut world = World::default();

        let _ = world.spawn((A(1), B(String::from("Cart"))));
        let _ = world.spawn((A(2), B(String::from("Alice"))));
        let _ = world.spawn((A(3), B(String::from("James"))));

        let _ = unsafe {
            <(&B, &B) as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };
    }

    #[test]
    fn test_optional_queries_1() {
        let mut world = World::default();

        world.spawn((A(1), B(String::from("Cart"))));
        world.spawn((A(2), B(String::from("Alice"))));
        world.spawn((A(3), B(String::from("James"))));

        world.spawn((C(1), B(String::from("Cart"))));
        world.spawn((C(2), B(String::from("Alice"))));
        world.spawn((C(3), B(String::from("James"))));

        world.spawn(C(1));
        world.spawn(C(2));
        world.spawn(C(3));

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));

        let optional_query_results = unsafe {
            <(Option<&B>, Option<&A>, Option<&C>) as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        let empty_query_results = unsafe {
            <() as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        assert_eq!(empty_query_results.count(), 12);
        assert_eq!(optional_query_results.count(), 12);
    }

    #[test]
    fn test_optional_queries_2() {
        let mut world = World::default();

        world.spawn((A(1), B(String::from("Cart"))));
        world.spawn((A(2), B(String::from("Alice"))));
        world.spawn((A(3), B(String::from("James"))));

        world.spawn((C(1), B(String::from("Cart"))));
        world.spawn((C(2), B(String::from("Alice"))));
        world.spawn((C(3), B(String::from("James"))));

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));

        let optional_query_results = unsafe {
            <(Option<&A>, Option<&B>, Option<&C>) as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        let mut a_count = 0;
        let mut b_count = 0;
        let mut c_count = 0;

        optional_query_results.for_each(|(a, b, c)| {
            if a.is_some() {
                a_count += 1;
            }
            if b.is_some() {
                b_count += 1;
            }
            if c.is_some() {
                c_count += 1;
            }
        });

        assert_eq!(a_count, 9);
        assert_eq!(b_count, 6);
        assert_eq!(c_count, 3);
    }

    #[test]
    fn test_containment_queries() {
        let mut world = World::default();

        world.spawn((A(1), B(String::from("Cart"))));
        world.spawn((A(2), B(String::from("Alice"))));
        world.spawn((A(3), B(String::from("James"))));

        world.spawn((C(1), B(String::from("Cart"))));
        world.spawn((C(2), B(String::from("Alice"))));
        world.spawn((C(3), B(String::from("James"))));

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));

        world.spawn(A(1));
        world.spawn(A(2));
        world.spawn(A(3));

        let optional_query_results = unsafe {
            <(Contains<A>, Contains<B>, Contains<C>) as ArchQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        let mut a_count = 0;
        let mut b_count = 0;
        let mut c_count = 0;

        optional_query_results.for_each(|(a, b, c)| {
            if a {
                a_count += 1;
            }
            if b {
                b_count += 1;
            }
            if c {
                c_count += 1;
            }
        });

        assert_eq!(a_count, 9);
        assert_eq!(b_count, 6);
        assert_eq!(c_count, 3);
    }
}
