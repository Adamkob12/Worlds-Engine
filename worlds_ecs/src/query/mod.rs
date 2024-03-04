#![allow(missing_docs)] // TODO: Remove

use worlds_derive::all_tuples;

use crate::{
    prelude::{Component, ComponentFactory},
    utils::prime_key::PrimeArchKey,
    world::storage::{
        arch_storage::{ArchStorage, ArchStorageIndex},
        storages::ArchStorages,
    },
};

pub unsafe trait ComponentQuery {
    type Item<'a>;
    fn merge_prime_arch_key_with(pkey: &mut PrimeArchKey, comp_factory: &ComponentFactory);
    /// # Safety
    ///   1) The caller must ensure that the [`ArchStorageIndex`] is withing the bounds of the [`ArchStorage`]
    /// (as specified in [`ArchStorage::get_component_unchecked`]).
    ///   2) The caller must ensure that the raw pointer to [`ArchStorage`] is valid, and usable.
    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> Self::Item<'a>;

    /// # Safety
    ///  [] The caller must ensure that the raw pointer to [`ArchStorages`] is valid, and usable.
    unsafe fn iter_query_matches<'a>(
        arch_storages: *mut ArchStorages,
        comp_factory: &'a ComponentFactory,
    ) -> impl Iterator<Item = Self::Item<'a>> + 'a {
        let mut pkey = PrimeArchKey::IDENTITY;
        Self::merge_prime_arch_key_with(&mut pkey, comp_factory);
        (*arch_storages)
            .iter_storages_with_matching_archetype_mut(pkey)
            .map(|arch_storage| {
                arch_storage
                    .iter_indices()
                    // SAFETY: The index must be in bounds because it came from the storage itself.
                    .map(|index| unsafe { Self::fetch(arch_storage, index, comp_factory) })
            })
            .flatten()
    }
}

unsafe impl<C: Component> ComponentQuery for &C {
    type Item<'a> = &'a C;

    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> Self::Item<'a> {
        (*arch_storage)
            .get_component_unchecked(
                index,
                comp_factory
                    .get_component_id::<C>()
                    .expect("Can't query unregistered component"),
            )
            .deref::<C>()
    }

    fn merge_prime_arch_key_with(pkey: &mut PrimeArchKey, comp_factory: &ComponentFactory) {
        pkey.merge_with_but_panic_if_already_merged(
            comp_factory
                .get_component_id::<C>()
                .expect("Can't query unregistered component")
                .prime_key(),
            "Can't query duplicate components",
        )
    }
}

unsafe impl<C: Component> ComponentQuery for &mut C {
    type Item<'a> = &'a mut C;

    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> Self::Item<'a> {
        (*arch_storage)
            .get_component_mut_unchecked(
                index,
                comp_factory
                    .get_component_id::<C>()
                    .expect("Can't query unregistered component"),
            )
            .deref_mut::<C>()
    }

    fn merge_prime_arch_key_with(pkey: &mut PrimeArchKey, comp_factory: &ComponentFactory) {
        pkey.merge_with_but_panic_if_already_merged(
            comp_factory
                .get_component_id::<C>()
                .expect("Can't query unregistered component")
                .prime_key(),
            "Can't query duplicate components",
        )
    }
}

unsafe impl<C: Component> ComponentQuery for Option<&mut C> {
    type Item<'a> = Option<&'a mut C>;

    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> Self::Item<'a> {
        (*arch_storage)
            .get_component_mut(
                index,
                comp_factory
                    .get_component_id::<C>()
                    .expect("Can't query unregistered component"),
            )
            .map(|c| c.deref_mut::<C>())
    }

    fn merge_prime_arch_key_with(_pkey: &mut PrimeArchKey, _comp_factory: &ComponentFactory) {
        // No need to merge anything, because this [`ComponentQuery`] doesn't restrict the archetype
    }
}

unsafe impl<C: Component> ComponentQuery for Option<&C> {
    type Item<'a> = Option<&'a C>;

    unsafe fn fetch<'a>(
        arch_storage: *mut ArchStorage,
        index: ArchStorageIndex,
        comp_factory: &'a ComponentFactory,
    ) -> Self::Item<'a> {
        (*arch_storage)
            .get_component(
                index,
                comp_factory
                    .get_component_id::<C>()
                    .expect("Can't query unregistered component"),
            )
            .map(|c| c.deref::<C>())
    }

    fn merge_prime_arch_key_with(_pkey: &mut PrimeArchKey, _comp_factory: &ComponentFactory) {
        // No need to merge anything, because this [`ComponentQuery`] doesn't restrict the archetype
    }
}

macro_rules! impl_comp_query_for_tuple {
    ($($name:ident),*) => {
        #[allow(non_snake_case, unused)]
        unsafe impl<$($name: ComponentQuery),*> ComponentQuery for ($($name,)*) {
            type Item<'a> = ($($name::Item<'a>,)*);

            unsafe fn fetch<'a>(
                arch_storage: *mut ArchStorage,
                index: ArchStorageIndex,
                comp_factory: &'a ComponentFactory,
            ) -> Self::Item<'a> {
                ($($name::fetch(arch_storage, index, comp_factory),)*)
            }

            fn merge_prime_arch_key_with(pkey: &mut PrimeArchKey, comp_factory: &ComponentFactory) {
                $($name::merge_prime_arch_key_with(pkey, comp_factory);)*
            }
        }
    };
}

all_tuples!(impl_comp_query_for_tuple, 0, 12, B);

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
            <&B as ComponentQuery>::iter_query_matches(
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
            <&B as ComponentQuery>::iter_query_matches(
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
            <(&C, &mut B) as ComponentQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        query_results.for_each(|(_, B(name))| {
            *name = String::from("BOO!");
        });

        let query_results = unsafe {
            <(&C, &B) as ComponentQuery>::iter_query_matches(
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
            <(&A, &B) as ComponentQuery>::iter_query_matches(
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
            <&mut B as ComponentQuery>::iter_query_matches(
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
            <(&B, &B) as ComponentQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };
    }

    #[test]
    fn test_optional_queries() {
        let mut world = World::default();

        world.spawn((A(1), B(String::from("Cart"))));
        world.spawn((A(2), B(String::from("Alice"))));
        world.spawn((A(3), B(String::from("James"))));

        world.spawn((C(1), B(String::from("Cart"))));
        world.spawn((C(2), B(String::from("Alice"))));
        world.spawn((C(3), B(String::from("James"))));

        let query_results = unsafe {
            <(&B, Option<&A>, Option<&C>) as ComponentQuery>::iter_query_matches(
                &mut world.storages.arch_storages,
                &world.components,
            )
        };

        assert_eq!(query_results.count(), 6);
    }
}
