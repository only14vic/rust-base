use {
    crate::prelude::*,
    alloc::{format, sync::Arc},
    core::{
        any::{Any, TypeId, type_name},
        ptr::addr_eq
    }
};

#[derive(Default, FromStatic)]
pub struct Di {
    container: IndexMap<TypeId, Arc<dyn Any + Send + Sync>>
}

impl Drop for Di {
    fn drop(&mut self) {
        if self.container.is_empty() == false {
            self.clear();
        }
    }
}

impl Di {
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.container
            .get(&TypeId::of::<T>())
            .map(|v| v.clone().downcast::<T>().unwrap())
    }

    pub fn get_ref<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.container
            .get(&TypeId::of::<T>())
            .map(|v| v.downcast_ref::<T>().unwrap())
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Ok<&mut T> {
        match self.container.get_mut(&TypeId::of::<T>()) {
            Some(v) => {
                match Arc::get_mut(v) {
                    Some(v) => v.downcast_mut::<T>().unwrap().into_ok(),
                    None => {
                        Err(format!(
                            "Could not get mutable '{}' from container",
                            type_name::<T>()
                        ))?
                    },
                }
            },
            None => {
                Err(format!(
                    "There is no item '{}' in container",
                    type_name::<T>()
                ))?
            },
        }
    }

    pub fn add<T: Send + Sync + 'static>(&mut self, obj: T) -> Option<Arc<T>> {
        self.container
            .insert(TypeId::of::<T>(), Arc::new(obj))
            .map(|v| v.downcast::<T>().unwrap())
    }

    pub fn add_ref<T: Send + Sync + 'static>(
        &mut self,
        obj_ref: Arc<T>
    ) -> Option<Arc<T>> {
        self.container
            .insert(TypeId::of::<T>(), obj_ref)
            .map(|v| v.downcast::<T>().unwrap())
    }

    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<Arc<T>> {
        self.container
            .swap_remove(&TypeId::of::<T>())
            .map(|v| v.downcast::<T>().unwrap())
    }

    pub fn take<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.container
            .swap_remove(&TypeId::of::<T>())
            .and_then(|v| Arc::into_inner(v.downcast::<T>().unwrap()))
    }

    pub fn has<T: Send + Sync + 'static>(&self) -> bool {
        self.container.contains_key(&TypeId::of::<T>())
    }

    pub fn len(&self) -> usize {
        self.container.len()
    }

    pub fn is_empty(&self) -> bool {
        self.container.is_empty()
    }

    pub fn clear(&mut self) {
        core::mem::take(&mut self.container);

        if addr_eq(self, Self::from_static()) {
            Env::is_debug().then(|| log::trace!("Global Di cleared"));
        } else {
            Env::is_debug().then(|| log::trace!("Di cleared"));
        }
    }
}
