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
                }
            }
        }

        impl MemVar for Var<$t> {
            unsafe fn to_buffer(&self, buffer: *mut u8, subvalue: u8) -> u8 {
                *(buffer as *mut $t) = match subvalue {
                    0 => self.get(),
                    1 => self.value.get(),
                    2 => self.changed_value.get(),
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
                    false => 0,
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
                    _ => (),
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
                if (value != self.value.get()) {
                    self.value.set(value);

                    if self.subscribed.get() != SubscribeMode::Off {
                        let stored = self.changed_value.get();
                        let delta = self.min_delta.get();
                        if stored + delta <= value || stored - delta >= value {
                            match self.subscribed.get() {
                                SubscribeMode::Sticky => {
                                    if !self.dirty.get() && stored != value {
                                        self.changed_value.set(value);
                                        self.dirty.set(true);
                                    }
                                }
                                SubscribeMode::Current => {
                                    if stored != value {
                                        self.changed_value.set(value);
                                        self.dirty.set(true);
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }

            fn subscribe(&self, value: SubscribeMode) {
                self.subscribed.set(value);
            }
        }

        impl VarChange for Var<$t> {
            type VarType = $t;
        
            fn get_value(&self) -> $t {
                self.value.get()
            }
        
            fn is_posedge(&self, snapshot: $t) -> bool {
                snapshot + self.min_delta.get() <= self.value.get()
            }
        
            fn is_negedge(&self, snapshot: $t) -> bool {
                snapshot - self.min_delta.get() >= self.value.get()
            }
        }

        impl NumVar<$t> for Var<$t> {
            fn inc(&self, add: $t) {
                match self.value.get().checked_add(add) {
                    Some(t) => self.value.set(t),
                    None => self
                        .value
                        .set(<$t>::MIN + (add - (<$t>::MAX - self.value.get()))),
                }
            }

            fn add(&self, add: $t) -> bool {
                match self.value.get().checked_add(add) {
                    Some(t) => {
                        self.value.set(t);
                        true
                    }
                    None => false,
                }
            }

            fn sub(&self, substract: $t) -> bool {
                match self.value.get().checked_sub(substract) {
                    Some(t) => {
                        self.value.set(t);
                        true
                    }
                    None => false,
                }
            }

            fn delta(&self, delta: $t) {
                self.min_delta.set(delta);
            }
        }
    };
}
