pub static mut POLL_CALLED: bool = false;

/// async loop, avoids blocking and lets other async 
/// tasks make progess.
/// This macro requires double-curly braces.
///
/// # Example
/// 
/// ```
/// loop_async! {{
///   //your async task code
/// }}
/// ```

#[macro_export]
macro_rules! loop_async {
    {$body:block} => { 
        loop {
            $body

            if pilot_sys::poll::await_next_cycle_needed() { 
                pilot_sys::time::wait_next_cycle().await;
            }
        } 
    }
}

#[inline(always)]
pub fn await_next_cycle_needed() -> bool {
    unsafe {
        if !POLL_CALLED {
            return true;
        }
        POLL_CALLED = false;
    }
    false
}

#[inline(always)]
pub fn poll_called() {
    unsafe {
        POLL_CALLED = true;
    }
}