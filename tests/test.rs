#![cfg(test)]

use single_thread_cell::{SingleThreadCell, SingleThreadRefCell};

#[test]
fn test_single_thread_cell() {
    let cell = SingleThreadCell::new(0);
    assert_eq!(cell.get(), 0);
    cell.set(1);
    assert_eq!(cell.get(), 1);
}

#[test]
fn test_single_thread_ref_cell() {
    let cell = SingleThreadRefCell::new(0);
    assert_eq!(*cell.borrow(), 0);
    *cell.borrow_mut() += 1;
    assert_eq!(*cell.borrow(), 1);

    *cell.borrow_mut() += 1;
    assert_eq!(*cell.borrow(), 2);
    *cell.borrow_mut() += 1;
    assert_eq!(*cell.borrow(), 3);
    assert_eq!(*cell.borrow(), 3);

    {
        let b1 = cell.borrow();
        let b2 = cell.borrow();
        assert_eq!(*b1, 3);
        assert_eq!(*b2, 3);

    }
}

#[test]
#[should_panic]
fn test_single_thread_ref_cell_panic_twice_mut() {
    let cell = SingleThreadRefCell::new(0);
    let _b = cell.borrow_mut();
    let _ab = cell.borrow_mut();
}

#[test]
#[should_panic]
fn test_single_thread_ref_cell_panic_mixed() {
    let cell = SingleThreadRefCell::new(0);
    let _b = cell.borrow();
    let _ab = cell.borrow_mut();
}

#[test]
#[should_panic]
fn test_single_thread_ref_cell_panic_mixed2() {
    let cell = SingleThreadRefCell::new(0);
    let _ab = cell.borrow_mut();
    let _b = cell.borrow();
}

#[test]
fn test_different_thread_borrow() {
    let cell = std::sync::Arc::new(SingleThreadRefCell::new(0));

    *cell.borrow_mut() = 1;
    assert_eq!(*cell.borrow(), 1);

    let cloned = cell.clone();
    let result = std::thread::spawn(move || {
        cloned.borrow();
    }).join();
    assert!(result.is_err());

    let cell = std::sync::Arc::new(SingleThreadCell::new(0));

    cell.set(1);
    assert_eq!(cell.get(), 1);
    let cloned = cell.clone();
    let result = std::thread::spawn(move || {
        cloned.get();
    }).join();
    assert!(result.is_err());
}

#[test]
fn test_different_thread_borrow_mut() {
    let cell = std::sync::Arc::new(SingleThreadRefCell::new(0));

    *cell.borrow_mut() = 1;
    assert_eq!(*cell.borrow(), 1);

    let cloned = cell.clone();
    let result = std::thread::spawn(move || {
       *cloned.borrow_mut() = 2;
    }).join();
    assert!(result.is_err());

    let cell = std::sync::Arc::new(SingleThreadCell::new(0));

    cell.set(1);
    assert_eq!(cell.get(), 1);

    let cloned = cell.clone();
    let result = std::thread::spawn(move || {
        cloned.set(2);
    }).join();
    assert!(result.is_err());
}



// How to test abort?