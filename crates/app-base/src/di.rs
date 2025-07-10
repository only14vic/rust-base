use {
    crate::prelude::*,
    alloc::{boxed::Box, format, sync::Arc},
    core::{
        any::{type_name, Any, TypeId},
        ptr::null_mut,
        sync::atomic::{AtomicPtr, Ordering}
    }
};

#[derive(Default)]
pub struct Di {
    container: IndexMap<TypeId, Arc<dyn Any + Send + Sync>>
}

impl Di {
    pub fn from_static() -> &'static mut Self {
        static DI: AtomicPtr<Di> = AtomicPtr::new(null_mut());

        let mut di = DI.load(Ordering::Acquire);

        if di.is_null() {
            di = Box::leak(Self::default().into());
            if let Err(prev) = DI.compare_exchange(
                null_mut(),
                di,
                Ordering::SeqCst,
                Ordering::Relaxed
            ) {
                let _ = unsafe { Box::from_raw(di) };
                di = prev;
            }
        }

        unsafe { &mut *di }
    }

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

    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> OkAsync<Option<&mut T>> {
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
            None => Ok(None)
        }
    }

    pub fn set<T: Send + Sync + 'static>(&mut self, obj: T) -> Option<Arc<T>> {
        self.container
            .insert(TypeId::of::<T>(), Arc::new(obj))
            .map(|v| v.downcast::<T>().unwrap())
    }

    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<Arc<T>> {
        self.container
            .swap_remove(&TypeId::of::<T>())
            .map(|v| v.downcast::<T>().unwrap())
    }

    pub fn has<T: Send + Sync + 'static>(&self) -> bool {
        self.container.contains_key(&TypeId::of::<T>())
    }
}
