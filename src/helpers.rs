use core::ops::{Deref, DerefMut};

use alloc::{borrow::Cow, string::String};

pub struct AssertThreadSafe<T: ?Sized>(T);

unsafe impl<T: ?Sized> Sync for AssertThreadSafe<T> {}

impl<T> AssertThreadSafe<T> {
    pub const unsafe fn new_unchecked(val: T) -> Self {
        Self(val)
    }

    pub fn into_innner(this: Self) -> T {
        this.0
    }
}

impl<T: ?Sized> AssertThreadSafe<T> {
    pub const fn get(this: &Self) -> &T {
        &this.0
    }

    pub const fn get_mut(this: &mut Self) -> &mut T {
        &mut this.0
    }
}

impl<T: ?Sized> Deref for AssertThreadSafe<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::get(self)
    }
}

impl<T: ?Sized> DerefMut for AssertThreadSafe<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::get_mut(self)
    }
}

pub trait SplitOnceOwned: Sized {
    fn split_once_take(&mut self, p: &str) -> Option<Self>;

    fn split_once_owned(mut self, p: &str) -> Result<(Self, Self), Self> {
        match self.split_once_take(p) {
            Some(val) => Ok((self, val)),
            None => Err(self),
        }
    }
}

impl SplitOnceOwned for String {
    fn split_once_take(&mut self, p: &str) -> Option<Self> {
        let n = self.find(p)?;

        let k = self.split_off(n + p.len());

        self.drain(n..);

        Some(k)
    }
}

impl<'a> SplitOnceOwned for Cow<'a, str> {
    fn split_once_take(&mut self, p: &str) -> Option<Self> {
        match self {
            Cow::Borrowed(n) => {
                let (a, b) = (*n).split_once(p)?;
                *n = a;
                Some(Cow::Borrowed(b))
            }
            Cow::Owned(n) => {
                let b = n.split_once_take(p)?;

                Some(Cow::Owned(b))
            }
        }
    }
}
