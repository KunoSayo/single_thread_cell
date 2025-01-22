use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::thread::ThreadId;

type BorrowFlag = isize;
const UNUSED: BorrowFlag = 0;

#[inline(always)]
fn is_writing(x: BorrowFlag) -> bool {
    x < UNUSED
}

#[inline(always)]
fn is_reading(x: BorrowFlag) -> bool {
    x > UNUSED
}

#[track_caller]
#[cold]
fn panic_already_borrowed() -> ! {
    panic!("already borrowed")
}

#[track_caller]
#[cold]
fn panic_already_mutably_borrowed() -> ! {
    panic!("already mutably borrowed")
}

pub trait SingleThreadType {
    fn get_owner_thread_id(&self) -> ThreadId;

    /// Check the current thread or panic(abort).
    #[inline]
    fn check_thread_panic(&self) {
        let current_id = std::thread::current().id();
        if current_id != self.get_owner_thread_id() {
            panic!("Access single thread cell with different thread id {:?}", current_id);
        }
    }
}

/// A mutable memory location. Can only be accessed by the owner thread.
///
/// If you access the cell from a different thread, the process will be aborted.
pub struct SingleThreadCell<T> {
    value: UnsafeCell<T>,
    owner_thread: ThreadId,
}

impl<T> SingleThreadType for SingleThreadCell<T> {
    fn get_owner_thread_id(&self) -> ThreadId {
        self.owner_thread
    }
}


impl<T> SingleThreadCell<T> {
    pub fn new(val: T) -> Self {
        Self {
            value: UnsafeCell::new(val),
            owner_thread: std::thread::current().id(),
        }
    }

    /// Set the contained value
    ///
    /// # Panics
    /// This function will panic if access from different thread
    #[inline]
    pub fn set(&self, value: T) {
        self.check_thread_panic();
        // SAFETY: We checked the thread.
        unsafe { *self.value.get() = value; }
    }
}

impl<T: Copy> SingleThreadCell<T> {
    /// Returns a copy of the contained value.
    #[inline]
    pub fn get(&self) -> T {
        self.check_thread_panic();
        // SAFETY: We checked the thread.
        unsafe { *self.value.get() }
    }
}

impl<T> Drop for SingleThreadCell<T> {
    fn drop(&mut self) {
        if cfg!(debug_assertions) || std::mem::needs_drop::<T>() {
            self.check_thread_panic();
        }
    }
}

unsafe impl<T> Send for SingleThreadCell<T> {}
unsafe impl<T> Sync for SingleThreadCell<T> {}
unsafe impl<T> Send for SingleThreadRefCell<T> {}
unsafe impl<T> Sync for SingleThreadRefCell<T> {}


pub struct SingleThreadRefCell<T> {
    borrow: UnsafeCell<BorrowFlag>,
    value: UnsafeCell<T>,
    owner_thread: ThreadId,
}

impl<T> SingleThreadRefCell<T> {
    pub fn new(val: T) -> Self {
        Self {
            borrow: UnsafeCell::new(UNUSED),
            value: UnsafeCell::new(val),
            owner_thread: std::thread::current().id(),
        }
    }
}

impl<T> SingleThreadRefCell<T> {
    /// Immutably borrows the wrapped value, returning an error if the value is currently mutably borrowed.
    ///
    /// The borrow lasts until the returned Ref exits scope. Multiple immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    /// This function will panic if access from different thread, or already borrowed
    pub fn borrow(&self) -> SingleThreadRef<'_, T> {
        self.check_thread_panic();

        // We checked the thread.
        match unsafe { BorrowRef::new(&self.borrow) } {
            Some(b) => {
                let value = unsafe { NonNull::new_unchecked(self.value.get()) };
                SingleThreadRef { value, _borrow: b, marker: Default::default() }
            }
            None => {
                panic_already_mutably_borrowed()
            }
        }
    }

    /// Mutably borrows the wrapped value, returning none if the value is currently borrowed.
    ///
    /// # Panics
    /// This function will panic if access from different thread, or already borrowed
    pub fn borrow_mut(&self) -> SingleThreadRefMut<'_, T> {
        self.check_thread_panic();
        // We checked the thread.
        match unsafe { BorrowRefMut::new(&self.borrow) } {
            Some(b) => {
                // SAFETY: `BorrowRefMut` guarantees unique access.
                let value = unsafe { NonNull::new_unchecked(self.value.get()) };
                SingleThreadRefMut { value, _borrow: b, marker: PhantomData }
            }
            None => {
                panic_already_borrowed();
            }
        }
    }
}

impl<T> SingleThreadType for SingleThreadRefCell<T> {
    fn get_owner_thread_id(&self) -> ThreadId {
        self.owner_thread
    }
}

struct BorrowRef<'a> {
    borrow: &'a UnsafeCell<BorrowFlag>,
}

impl<'b> BorrowRef<'b> {
    #[inline]
    /// Outside should keep the borrow in the same thread.
    unsafe fn new(borrow: &'b UnsafeCell<BorrowFlag>) -> Option<BorrowRef<'b>> {
        let b = (*borrow.get()).wrapping_add(1);
        if !is_reading(b) {
            // Writing or overflow.
            None
        } else {
            *borrow.get() = b;
            Some(BorrowRef { borrow })
        }
    }
}

struct BorrowRefMut<'b> {
    borrow: &'b UnsafeCell<BorrowFlag>,
    // Mark this is not send or sync
    _marker: PhantomData<Cell<()>>,
}

impl<'b> BorrowRefMut<'b> {
    // Outside should keep the borrow in the same thread.
    #[inline]
    unsafe fn new(borrow: &'b UnsafeCell<BorrowFlag>) -> Option<BorrowRefMut<'b>> {
        // NOTE: Unlike BorrowRefMut::clone, new is called to create the initial
        // mutable reference, and so there must currently be no existing
        // references. Thus, while clone increments the mutable refcount, here
        // we explicitly only allow going from UNUSED to UNUSED - 1.
        match *borrow.get() {
            UNUSED => {
                *borrow.get() = UNUSED - 1;
                Some(BorrowRefMut { borrow: borrow, _marker: Default::default() })
            }
            _ => None,
        }
    }
}

pub struct SingleThreadRef<'a, T: 'a> {
    value: NonNull<T>,
    _borrow: BorrowRef<'a>,
    // Mark this is not send or sync
    marker: PhantomData<Cell<()>>,
}
pub struct SingleThreadRefMut<'b, T: ?Sized + 'b> {
    value: NonNull<T>,
    _borrow: BorrowRefMut<'b>,
    marker: PhantomData<&'b mut T>,
}


impl Drop for BorrowRef<'_> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: we should new with thread check.
        // This is not send nor sync.
        unsafe {
            let borrow = *self.borrow.get();
            debug_assert!(is_reading(borrow));
            *self.borrow.get() = borrow - 1;
        }
    }
}

impl Drop for BorrowRefMut<'_> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: we should new with thread check.
        // This is not send nor sync.
        unsafe {
            let borrow = *self.borrow.get();
            debug_assert!(is_writing(borrow));
            *self.borrow.get() = borrow + 1;
        }
    }
}


impl<T> Deref for SingleThreadRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}

impl<T> Deref for SingleThreadRefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}

impl<T> DerefMut for SingleThreadRefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.value.as_mut() }
    }
}

impl<T: Default> Default for SingleThreadCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T: Default> Default for SingleThreadRefCell<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}