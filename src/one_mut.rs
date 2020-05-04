use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, self};

pub struct OneMut<T> {
    flag: AtomicBool, // if there exists a MutGuard
    inner: UnsafeCell<T>,
}

impl<T> OneMut<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            flag: AtomicBool::new(false),
            inner: UnsafeCell::new(inner),
        }
    }

    pub fn get_mut<'s>(&'s self) -> Option<MutGuard<'s, T>> {
        let prev = self.flag.compare_and_swap(false, true, atomic::Ordering::SeqCst); // maybe another ordering
        if prev {
            // failed
            return None;
        } else {
            unsafe {
                Some(MutGuard {
                    flag: &self.flag,
                    inner: &mut *self.inner.get(),
                })
            }
        }
    }
}

unsafe impl<T: Sync> Sync for OneMut<T> {}
unsafe impl<T: Send> Send for OneMut<T> {}

pub struct MutGuard<'s, T: 's> {
    inner: &'s mut T,
    flag: &'s AtomicBool,
}

impl<T> Deref for MutGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T> DerefMut for MutGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<T> Drop for MutGuard<'_, T> {
    fn drop(&mut self) {
        self.flag.store(false, atomic::Ordering::SeqCst); // maybe another ordering
    }
}

#[test]
fn test_one_mut() {
    const N: usize = 16;

    static DATA_BUF: OneMut<[u8; N]> = OneMut::new([0; N]);

    let mut guard: MutGuard<'static, [u8; N]> = DATA_BUF.get_mut().unwrap();

    let buf: &mut [u8] = &mut *guard;
    assert_eq!(buf, &mut [0; N]);

    std::thread::spawn(|| {
        let opt = DATA_BUF.get_mut();
        assert!(opt.is_none());
    })
    .join()
    .unwrap();
}
