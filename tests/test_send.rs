#![cfg(test)]

use std::marker::PhantomData;
use single_thread_cell::{SingleThreadCell, SingleThreadRef, SingleThreadRefCell, SingleThreadRefMut};

#[derive(Eq, PartialEq, Debug)]
struct True;
#[derive(Eq, PartialEq, Debug)]
struct False;

trait SendCheckTrait {
    const IS_SEND: False = False;
}

impl<T: ?Sized> SendCheckTrait for T {}


macro_rules! is_send {
    ($ty: ty) => {{
        struct Wrapper<T: ?Sized>(PhantomData<T>);

        #[allow(dead_code)]
        impl<U: ?Sized + Send> Wrapper<U> {
            const IS_SEND: True = True;
        }
        <Wrapper<$ty>>::IS_SEND
    }};
}


#[test]
fn test_send() {
    use std::rc::Rc;

    // check function correct.

    assert_eq!(is_send!(i32), True);
    assert_eq!(is_send!(Rc<i32>), False);


    assert_eq!(is_send!(SingleThreadCell<i32>), True);
    assert_eq!(is_send!(SingleThreadRefCell<i32>), True);

    assert_eq!(is_send!(SingleThreadCell<Rc<()>>), False);
    assert_eq!(is_send!(SingleThreadRefCell<Rc<()>>), False);

    assert_eq!(is_send!(SingleThreadRef<()>), False);
    assert_eq!(is_send!(SingleThreadRefMut<()>), False);
}