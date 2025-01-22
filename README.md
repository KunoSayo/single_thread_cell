## Introduction

This is a helper library to mark the cell as only being accessed by the owner thread.

> Still in development, the API may change in the future.

## Quick Start

```rust
use single_thread_cell::{SingleThreadCell, SingleThreadRefCell};

let cell = SingleThreadCell::new(0);
assert_eq!(cell.get(), 0);
cell.set(1);
assert_eq!(cell.get(), 1);

let ref_cell = SingleThreadRefCell::new(0);
assert_eq!(*ref_cell.borrow(), 0);
*ref_cell.borrow_mut() += 1;
assert_eq!(*ref_cell.borrow(), 1);
```