use std::thread::ThreadId;

pub trait SingleThreadType {
    fn get_owner_thread_id(&self) -> ThreadId;

    /// Check the current thread and panic if not same.
    #[inline]
    fn check_thread_panic(&self) {
        let current_id = std::thread::current().id();
        if current_id != self.get_owner_thread_id() {
            panic!("Access single thread cell with different thread id {:?}", current_id);
        }
    }

    #[inline]
    fn check_same_thread(&self) -> bool {
        let current_id = std::thread::current().id();
        current_id == self.get_owner_thread_id()
    }
}
