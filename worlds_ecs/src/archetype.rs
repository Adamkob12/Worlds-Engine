use crate::{
    component::{Component, ComponentFactory, ComponentId},
    utils::prime_key::PrimeArchKey,
};
use worlds_derive::all_tuples;

/// Maximum amount of components per archetype, This is also the maximum amount of components per entity.
pub const MAX_COMPS_PER_ARCH: usize = 30;

/// Information representing the information of a [`Archetype`] in the [`World`].
#[derive(Default, Debug)]
pub struct ArchetypeInfo {
    component_ids: Vec<ComponentId>,
    prime_key: PrimeArchKey,
}

impl ArchetypeInfo {
    fn merge_with(&mut self, other: ArchetypeInfo) {
        self.component_ids.extend(other.component_ids);
        self.prime_key.merge_with(other.prime_key);
    }

    /// Get the unique [`PrimeArchKey`] of this [`Archetype`].
    pub fn prime_key(&self) -> PrimeArchKey {
        self.prime_key
    }

    /// Get the [`Component`]s that make up this [`Archetype`].
    pub fn component_ids(&self) -> &[ComponentId] {
        &self.component_ids
    }

    /// Verify that there aren't duplicate components in this archetype
    /// Return `true` if there are duplicate components in this [`Archetype`]. else `false`.
    pub fn check_for_duplicates(&self) -> bool {
        for comp_id in self.component_ids() {
            if self
                .prime_key()
                .is_sub_archetype(comp_id.prime_key().squared())
            {
                return true;
            }
        }
        false
    }
}

/// An archetype is a unique set of components.
// TODO: Expand on documentation with examples and explanations.
///
/// # Safety
/// Do not implement this trait for your types. If this trait is misimplemented.
pub unsafe trait Archetype: Sized {
    /// Get the [`ArchetypeInfo`] of this archetype for a matching [`World`] (whose component info is stored in [`ComponentFactory`]).
    /// If this [`Archetype`]'s components aren't all registered, it registers them first, and then returns the [`ArchetypeInfo`].
    fn get_info_or_register(comp_factory: &mut ComponentFactory) -> ArchetypeInfo;
    /// Get the [`ArchetypeInfo`] of this archetype for a matching [`World`] (whose component info is stored in [`ComponentFactory`])
    fn arch_info(comp_factory: &ComponentFactory) -> Option<ArchetypeInfo>;
    /// Get the [`PrimeArchKey`] of this archetype for a matching [`World`] (whose component info is stored in [`ComponentFactory`]).
    /// If this [`Archetype`]'s components aren't all registered, it registers them first, and then returns the [`PrimeArchKey`].
    fn get_prime_key_or_register(comp_factory: &mut ComponentFactory) -> PrimeArchKey;
    /// Get the [`PrimeArchKey`] of this archetype for a matching [`World`] (whose component info is stored in [`ComponentFactory`])
    fn prime_key(comp_factory: &ComponentFactory) -> Option<PrimeArchKey>;
}

unsafe impl<C> Archetype for C
where
    C: Component,
{
    fn get_info_or_register(comp_factory: &mut ComponentFactory) -> ArchetypeInfo {
        comp_factory
            .register_component::<C>()
            .map(|id| ArchetypeInfo {
                component_ids: vec![id],
                prime_key: id.prime_key(),
            })
            .expect("The maximum amount of registered components has been reached.")
    }

    fn arch_info(comp_factory: &ComponentFactory) -> Option<ArchetypeInfo> {
        comp_factory
            .get_component_id::<C>()
            .map(|id| ArchetypeInfo {
                component_ids: vec![id],
                prime_key: id.prime_key(),
            })
    }

    fn prime_key(comp_factory: &ComponentFactory) -> Option<PrimeArchKey> {
        comp_factory
            .get_component_id::<C>()
            .map(|cid| cid.prime_key())
    }

    fn get_prime_key_or_register(comp_factory: &mut ComponentFactory) -> PrimeArchKey {
        comp_factory
            .register_component::<C>()
            .map(|cid| cid.prime_key())
            .expect("The maximum amout of registered components has been reached.")
    }
}

macro_rules! impl_archetype {
    ($($name:ident),*) => {
        #[allow(non_snake_case, unused)]
        unsafe impl<$($name: Archetype),*> Archetype for ($($name,)*) {
            fn get_info_or_register(components: &mut ComponentFactory) -> ArchetypeInfo {
                let mut arch_info = ArchetypeInfo::default();
                $(arch_info.merge_with($name::get_info_or_register(components));)*
                arch_info
            }

            fn arch_info(components: &ComponentFactory) -> Option<ArchetypeInfo> {
                let mut arch_info = ArchetypeInfo::default();
                $(arch_info.merge_with($name::arch_info(components)?);)*
                Some(arch_info)
            }

            fn prime_key(comp_factory: &ComponentFactory) -> Option<PrimeArchKey> {
                let mut pkey = PrimeArchKey::IDENTITY;
                $(pkey.merge_with($name::prime_key(comp_factory)?);)*
                Some(pkey)
            }

            fn get_prime_key_or_register(comp_factory: &mut ComponentFactory) -> PrimeArchKey {
                let mut pkey = PrimeArchKey::IDENTITY;
                $(pkey.merge_with($name::get_prime_key_or_register(comp_factory));)*
                pkey
            }
        }
    };
}

all_tuples!(impl_archetype, 0, 15, A);

#[cfg(test)]
mod tests {
    use super::Archetype;
    use crate::prelude::*;

    #[derive(Component)]
    struct A;
    #[derive(Component)]
    struct B;
    #[derive(Component)]
    struct C;

    #[test]
    fn test_archetype_prime_keys() {
        let mut comp_factory = ComponentFactory::default();
        comp_factory.register_component::<A>();
        comp_factory.register_component::<B>();
        comp_factory.register_component::<C>();

        assert_eq!(
            <A as Archetype>::arch_info(&comp_factory)
                .unwrap()
                .prime_key()
                .as_u64(),
            2
        );
        assert_eq!(
            <B as Archetype>::arch_info(&comp_factory)
                .unwrap()
                .prime_key()
                .as_u64(),
            3
        );
        assert_eq!(
            <C as Archetype>::arch_info(&comp_factory)
                .unwrap()
                .prime_key()
                .as_u64(),
            5
        );
        assert_eq!(
            <(A, B) as Archetype>::arch_info(&comp_factory)
                .unwrap()
                .prime_key()
                .as_u64(),
            6
        );
        assert_eq!(
            <(B, C) as Archetype>::arch_info(&comp_factory)
                .unwrap()
                .prime_key()
                .as_u64(),
            15
        );
        assert_eq!(
            <(A, C) as Archetype>::arch_info(&comp_factory)
                .unwrap()
                .prime_key()
                .as_u64(),
            10
        );
        assert_eq!(
            <(A, B, C) as Archetype>::arch_info(&comp_factory)
                .unwrap()
                .prime_key()
                .as_u64(),
            30
        );
        assert_eq!(
            <(A, B, C) as Archetype>::arch_info(&comp_factory)
                .unwrap()
                .prime_key()
                .as_u64(),
            <(C, B, A) as Archetype>::arch_info(&comp_factory)
                .unwrap()
                .prime_key()
                .as_u64(),
        );
    }

    #[test]
    fn test_archetype_components() {
        let mut comp_factory = ComponentFactory::default();
        comp_factory.register_component::<A>();
        comp_factory.register_component::<B>();
        comp_factory.register_component::<C>();

        let arch_info = <(A, B, C) as Archetype>::arch_info(&comp_factory).unwrap();
        let comps = arch_info.component_ids();
        assert_eq!(comps[0], ComponentId::new(0));
        assert_eq!(comps[1], ComponentId::new(1));
        assert_eq!(comps[2], ComponentId::new(2));
        assert!(!arch_info.check_for_duplicates());

        let arch_info = <(A, B, C, C) as Archetype>::arch_info(&comp_factory).unwrap();
        let comps = arch_info.component_ids();
        assert_eq!(comps[0], ComponentId::new(0));
        assert_eq!(comps[1], ComponentId::new(1));
        assert_eq!(comps[2], ComponentId::new(2));
        assert_eq!(comps[3], ComponentId::new(2));
        assert!(arch_info.check_for_duplicates());
    }
}
