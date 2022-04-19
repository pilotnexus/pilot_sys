//Simple Type implementation Macro
#[macro_export]
macro_rules! var_impl {
  ($t:ty) => {
    impl Var<$t> {
        pub const fn new() -> Var<$t> {
            Var {
                value: SyncCell::new(0),
                forced_value: SyncCell::new(0),
                changed_value: SyncCell::new(0),
                forced: SyncCell::new(false),
                min_delta: SyncCell::new(1),
                subscribed: SyncCell::new(SubscribeMode::Off),
                dirty: SyncCell::new(false),
                pos: SyncCell::new(false),
                neg: SyncCell::new(false),
            }
        }
    }
    
    impl MemVar for Var<$t> {
        unsafe fn to_buffer(&self, buffer: *mut u8, subvalue: u8) -> u8 {
            *(buffer as *mut $t) = match subvalue {
                0 => self.get(),
                1 => self.value.get(),
                2 => {
                  self.changed_value.get()
                },
                3 => self.forced_value.get(),
                _ => self.get(),
            };
            core::mem::size_of::<$t>() as u8
        }
    
        unsafe fn from_buffer(&self, buffer: *const u8, subvalue: u8) -> u8 {
            match subvalue {
                0 => self.set(*(buffer as *const $t)),
                1 => self.value.set(*(buffer as *const $t)),
                2 => self.changed_value.set(*(buffer as *const $t)),
                3 => self.forced_value.set(*(buffer as *const $t)),
                _ => self.set(*(buffer as *const $t)),
            };
            core::mem::size_of::<$t>() as u8
        }
    
        unsafe fn is_dirty(&self) -> bool {
            self.dirty.get()
        }
    
        unsafe fn clear_dirty(&self) {
          self.dirty.set(false);
        }
    
        unsafe fn get_forced(&self) -> u8 {
          match self.forced.get() {
            true => 1,
            false => 0
          }
        }
    
        unsafe fn set_forced(&self, value: u8) {
          if value > 0 {
            self.forced.set(true);
          } else {
            self.forced.set(false);
          }
        }
    
        unsafe fn get_subscribed(&self) -> u8 {
          match self.subscribed.get() {
            SubscribeMode::Off => 0,
            SubscribeMode::Sticky => 1,
            SubscribeMode::Current => 2,
          }
        }
    
        unsafe fn set_subscribed(&self, value: u8) {
          match value {
            0 => self.subscribed.set(SubscribeMode::Off),
            1 => self.subscribed.set(SubscribeMode::Sticky),
            2 => self.subscribed.set(SubscribeMode::Current),
            _ => ()
          }
        }
    }
    
    impl VarProps<$t> for Var<$t> {
        fn get(&self) -> $t {
            match self.forced.get() {
                true => self.forced_value.get(),
                false => self.value.get(),
            }
        }
    
        fn set(&self, value: $t) {
            if match (self.value.get(), self.min_delta.get()) {
                (v, _) if value == v => false, // unchanged
                (v, d) if value >= v+d => {    // pos change
                    self.value.set(value);
                    self.pos.set(true);
                    self.neg.set(false);
                    true
                },
                (v, d) if value <= v-d => {   // neg change
                    self.value.set(value);
                    self.pos.set(false);
                    self.neg.set(true);
                    true
                },
                (_, _) => {
                    self.value.set(value);
                    self.pos.set(false);
                    self.neg.set(false);
                    false
                }
            } {
                // pos or neg change
                match self.subscribed.get() {
                  SubscribeMode::Sticky => {
                    if !self.dirty.get() && self.changed_value.get() != value {
                        self.changed_value.set(value);
                        self.dirty.set(true);
                    }
                  },
                  SubscribeMode::Current => self.changed_value.set(value),
                  _ => ()
                }
            }
        }

        fn subscribe(&self, value: SubscribeMode) {
          self.subscribed.set(value);
        }
    }

    impl NumVar<$t> for Var<$t> {
        fn inc(&self, add: $t) {
            match self.value.get().checked_add(add) {
                Some(t) => self.value.set(t),
                None => self.value.set(<$t>::MIN + (add - (<$t>::MAX-self.value.get())))
            }
        }

        fn add(&self, add: $t) -> bool {
            match self.value.get().checked_add(add) {
                Some(t) => { self.value.set(t); true }
                None => false
            }
        }

        fn sub(&self, substract: $t) -> bool {
            match self.value.get().checked_sub(substract) {
                Some(t) => { self.value.set(t); true }
                None => false
            }
        }

        fn delta(&self, delta: $t) {
            self.min_delta.set(delta);
        }
    }

  }
}
