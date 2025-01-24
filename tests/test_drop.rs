#![cfg(test)]

use single_thread_cell::{SingleThreadCell, SingleThreadRefCell};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct TestDrop {
    dropped: Arc<AtomicBool>,
}

impl Drop for TestDrop {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

fn create_test_case() -> (Arc<AtomicBool>, TestDrop) {
    let x = Arc::new(AtomicBool::new(false));
    let td = TestDrop {
        dropped: x.clone(),
    };
    (x, td)
}

#[test]
fn test_drop() {
    {
        let (x, td) = create_test_case();
        {
            let _ = SingleThreadCell::new(td);
        }
        assert!(x.load(Ordering::SeqCst));
    }
    {
        let (x, td) = create_test_case();
        {
            let _ = SingleThreadRefCell::new(td);
        }
        assert!(x.load(Ordering::SeqCst));
    }
    {
        let (x, td) = create_test_case();
        {
            let x = SingleThreadRefCell::new(td);
            // We are safe to drop in the other thread.
            std::thread::spawn(|| {
                let _ = x;
            }).join().unwrap();
        }
        assert!(x.load(Ordering::SeqCst));
    }
    {
        let (x, td) = create_test_case();
        {
            let x = SingleThreadCell::new(td);
            // We are safe to drop in the other thread.
            std::thread::spawn(|| {
                let _ = x;
            }).join().unwrap();
        }
        assert!(x.load(Ordering::SeqCst));
    }
}