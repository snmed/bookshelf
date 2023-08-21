// Copyright © 2023 Sandro Dallo
//
// Use of this source code is governed by an BSD-style
// license that can be found in the LICENSE file.

use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

struct InnerPool<T: Send + ?Sized>(Arc<Mutex<Vec<Box<T>>>>, usize);

impl<T: Send + ?Sized> InnerPool<T> {
    fn acquire(&self) -> Result<Box<T>, bool> {
        let mut v = self.0.lock().unwrap();
        v.pop().ok_or(false)
    }

    fn relase(&self, item: Box<T>) {
        let mut v = self.0.lock().unwrap();
        if v.len() < self.1 {
            v.push(item)
        }
    }
}

pub struct PoolManager<T: Send + ?Sized> {
    creator: fn() -> Box<T>,
    pool: InnerPool<T>,
}


impl<T: Send + ?Sized> PoolManager<T> {
    pub fn new(min_pool: usize, creator: fn() -> Box<T>) -> PoolManager<T> {
        let mut conns: Vec<Box<T>> = Vec::new();
        for _ in 0..min_pool {
            conns.push(creator());
        }

        Self {
            creator,
            pool: InnerPool(Arc::new(Mutex::new(conns)), min_pool),
        }
    }

    pub fn get_pool_item(&self) -> PoolItem<T> {
        match self.pool.acquire() {
            Ok(p) => PoolItem(Some(p), InnerPool(Arc::clone(&self.pool.0), self.pool.1)),
            Err(_) => PoolItem(
                Some((self.creator)()),
                InnerPool(Arc::clone(&self.pool.0), self.pool.1),
            ),
        }
    }

    pub fn available_items(&self) -> usize {
        self.pool.0.lock().unwrap().len()
    }
}

pub struct PoolItem<T: Send + ?Sized>(Option<Box<T>>, InnerPool<T>);

impl<T: Send + ?Sized> Deref for PoolItem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap().as_ref()
    }
}

impl<T: Send + ?Sized> DerefMut for PoolItem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap().as_mut()
    }
}

impl<T: Send + ?Sized> Drop for PoolItem<T> {
    fn drop(&mut self) {
        self.1.relase(self.0.take().unwrap())
    }
}

#[cfg(test)] 
mod tests {
    use std::{thread, time::Duration, sync::Arc};
    use super::PoolManager;

    #[test]
    fn pool_test() {
        let pool = Arc::new(PoolManager::new(5, || {            
            Box::new("Just a test")
        }));
      
        let mut handles = Vec::new();
        for i in 0..15 {
            let p = pool.clone();
            handles.push(thread::spawn(move || {
                // Can't use `let _ = p.get_pool_item()` because value will be dropped immediately, therefore silence linter for now.
                #[allow(unused)]                
                let s = p.get_pool_item();                                            
                match i {
                    0..=3 => assert!(p.available_items() > 0),
                    _ => assert!(p.available_items() == 0),
                }

                thread::sleep(Duration::from_millis(500));
            }));
        }


        handles.drain(..).for_each(|h| {
            let _ = h.join();
        });
        assert!(pool.available_items() == 5);
    }
    
}