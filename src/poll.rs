pub static mut POLL_CALLED: bool = false;

#[macro_export]
macro_rules! plc_loop {
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