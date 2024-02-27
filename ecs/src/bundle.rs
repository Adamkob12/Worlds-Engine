use bevy_ptr::OwningPtr;
use worlds_derive::all_tuples;

use crate::prelude::{Component, ComponentFactory, ComponentId};

/// A bundle of components.
pub trait Bundle {
    /// This method calls `f` on all of the components in the bundle. This could, for example,
    ///  be used in conjunction with [`Vec::push`] to collect all the components into a [`Vec`].
    fn raw_components_scope(
        self,
        comp_factory: &ComponentFactory,
        f: &mut impl FnMut(ComponentId, OwningPtr<'_>),
    );
}

impl<C: Component> Bundle for C {
    fn raw_components_scope(
        self,
        comp_factory: &ComponentFactory,
        f: &mut impl FnMut(ComponentId, OwningPtr<'_>),
    ) {
        OwningPtr::make(self, |ptr| {
            f(
                comp_factory.get_component_id::<C>().unwrap(),
                // SAFETY: We own self
                ptr,
            )
        })
    }
}

macro_rules! impl_bundle_for_tuple {
    ($($name:ident),*) => {
        impl<$($name: Bundle),*> Bundle for ($($name,)*) {
            #[allow(non_snake_case, unused)]
            fn raw_components_scope(self, comp_factory: &ComponentFactory, f: &mut impl FnMut(ComponentId, OwningPtr<'_>)) {
                let ($($name,)*) = self;
                $($name.raw_components_scope(comp_factory, f);)*
            }
        }
    };
}

all_tuples!(impl_bundle_for_tuple, 0, 12, B);

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use worlds_derive::Component;

    #[derive(Component)]
    struct A(usize);

    #[derive(Component)]
    struct B(isize, isize, [isize; 20]);

    #[test]
    fn test_bundle() {
        let mut comp_factory = ComponentFactory::default();
        comp_factory.register_component::<A>();
        comp_factory.register_component::<B>();

        let blob_vec_a = unsafe {
            comp_factory
                .new_component_storage(ComponentId::new(0))
                .unwrap()
        };
        let blob_vec_b = unsafe {
            comp_factory
                .new_component_storage(ComponentId::new(1))
                .unwrap()
        };

        let mut storage = vec![blob_vec_a, blob_vec_b];
        <(A, B) as Bundle>::raw_components_scope(
            (A(33), B(-11, -99, [-456; 20])),
            &comp_factory,
            &mut |comp_id, raw_comp| unsafe { storage[comp_id.id()].push(raw_comp) },
        );
        <(A, B) as Bundle>::raw_components_scope(
            (A(66), B(-22, -99, [-56; 20])),
            &comp_factory,
            &mut |comp_id, raw_comp| unsafe { storage[comp_id.id()].push(raw_comp) },
        );
        <(A, B) as Bundle>::raw_components_scope(
            (A(99), B(-33, -99, [-4; 20])),
            &comp_factory,
            &mut |comp_id, raw_comp| unsafe { storage[comp_id.id()].push(raw_comp) },
        );

        assert_eq!(unsafe { storage[0].get_unchecked(0).deref::<A>().0 }, 33);
        assert_eq!(unsafe { storage[0].get_unchecked(1).deref::<A>().0 }, 66);
        assert_eq!(unsafe { storage[0].get_unchecked(2).deref::<A>().0 }, 99);

        assert_eq!(unsafe { storage[1].get_unchecked(0).deref::<B>().0 }, -11);
        assert_eq!(unsafe { storage[1].get_unchecked(1).deref::<B>().0 }, -22);
        assert_eq!(unsafe { storage[1].get_unchecked(2).deref::<B>().0 }, -33);

        assert_eq!(unsafe { storage[1].get_unchecked(0).deref::<B>().1 }, -99);
        assert_eq!(unsafe { storage[1].get_unchecked(1).deref::<B>().1 }, -99);
        assert_eq!(unsafe { storage[1].get_unchecked(2).deref::<B>().1 }, -99);

        assert_eq!(
            unsafe { storage[1].get_unchecked(0).deref::<B>().2 },
            [-456; 20]
        );
        assert_eq!(
            unsafe { storage[1].get_unchecked(1).deref::<B>().2 },
            [-56; 20]
        );
        assert_eq!(
            unsafe { storage[1].get_unchecked(2).deref::<B>().2 },
            [-4; 20]
        );
    }
}
