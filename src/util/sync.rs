use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Clone)]
pub struct SharedBool(Arc<AtomicBool>);

impl SharedBool {
    pub fn new(val: bool) -> Self {
        Self(Arc::new(AtomicBool::new(val)))
    }

    pub fn load(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub fn store(&self, val: bool) {
        self.0.store(val, Ordering::Relaxed)
    }
}

#[derive(Clone, Default)]
pub struct SharedCell<T: Default + Clone>(Arc<Mutex<T>>);

impl<T: Default + Clone> SharedCell<T> {
    pub fn new(val: T) -> Self {
        Self(Arc::new(Mutex::new(val)))
    }

    pub fn load(&self) -> Result<T, ()> {
        if let Ok(val) = self.0.lock() {
            Ok((*val).clone())
        } else {
            Err(())
        }
    }

    pub fn store(&self, new_val: T) -> Result<(), ()> {
        if let Ok(mut val) = self.0.lock() {
            *val = new_val;
            Ok(())
        } else {
            Err(())
        }
    }
}
