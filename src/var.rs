use crate::sync::SyncCell;
use crate::var_impl;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SubscribeMode {
  Off,
  Sticky,
  Current
}

pub trait PilotBindings {
    type BindType;

    fn set_from_pilot_bindings(&self, plc_mem: &Self::BindType);

    fn write_to_pilot_bindings(&self, plc_mem: &mut Self::BindType);
}

#[derive(Debug)]
#[repr(C)]
pub struct VariableInfo {
    pub name: &'static str,
    pub ty: &'static str,
    pub fields: &'static [VariableInfo], // for compound types
    pub field_number_offset: u16,        // field number adjustments for compound fields
}

pub trait TypeName {
    const TYPE_NAME: &'static str;
}

impl TypeName for u32 {
    const TYPE_NAME: &'static str = "u32";
}
impl TypeName for i32 {
    const TYPE_NAME: &'static str = "i32";
}
impl TypeName for u16 {
    const TYPE_NAME: &'static str = "u16";
}
impl TypeName for i16 {
    const TYPE_NAME: &'static str = "i16";
}
impl TypeName for u8 {
    const TYPE_NAME: &'static str = "u8";
}
impl TypeName for i8 {
    const TYPE_NAME: &'static str = "i8";
}
impl TypeName for bool {
    const TYPE_NAME: &'static str = "bool";
}

pub trait MemVar: Sync {
    unsafe fn to_buffer(&self, buffer: *mut u8, subvalue: u8) -> u8;
    unsafe fn from_buffer(&self, buffer: *const u8, subvalue: u8) -> u8;
    unsafe fn is_dirty(&self) -> bool;
    unsafe fn clear_dirty(&self);
    unsafe fn get_forced(&self) -> u8;
    unsafe fn set_forced(&self, value: u8);
    unsafe fn get_subscribed(&self) -> u8;
    unsafe fn set_subscribed(&self, value: u8);
}

pub trait VarProps<T> {
    fn get(&self) -> T;
    fn set(&self, value: T);
    fn subscribe(&self, value: SubscribeMode);
}

pub trait VarChange {
    fn pos(&self) -> bool;
    fn neg(&self) -> bool;
    fn changed(&self) -> bool;

    fn wait_pos(&self) -> WaitChange<'_, Self>
    where
        Self: Sized,
    {
        WaitChange {
            var: self,
            event: Event::Pos,
        }
    }

    fn wait_neg(&self) -> WaitChange<'_, Self>
    where
        Self: Sized,
    {
        WaitChange {
            var: self,
            event: Event::Neg,
        }
    }

    fn wait_changed(&self) -> WaitChange<'_, Self>
    where
        Self: Sized,
    {
        WaitChange {
            var: self,
            event: Event::Changed,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Event {
    Pos,
    Neg,
    Changed,
}

pub struct WaitChange<'a, V> {
    var: &'a V,
    event: Event,
}


impl<V> Future for WaitChange<'_, V>
where
    V: VarChange,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        crate::poll::poll_called();
        let &Self { var, event } = self.into_ref().get_ref();
        let finished = match event {
            Event::Pos => var.pos(),
            Event::Neg => var.neg(),
            Event::Changed => var.pos() || var.neg(),
        };
        if finished {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub struct Var<T: Default> {
    value: SyncCell<T>,
    changed_value: SyncCell<T>,
    forced_value: SyncCell<T>,
    forced: SyncCell<bool>,
    dirty: SyncCell<bool>,
    subscribed: SyncCell<SubscribeMode>,
    pos: SyncCell<bool>,
    neg: SyncCell<bool>,
}

impl<T: Default> VarChange for Var<T> {
    fn pos(&self) -> bool {
        self.pos.get()
    }

    fn neg(&self) -> bool {
        self.neg.get()
    }

    fn changed(&self) -> bool {
        self.pos.get() || self.neg.get()
    }
}

var_impl!(u64);
var_impl!(i64);

var_impl!(u32);
var_impl!(i32);

var_impl!(u16);
var_impl!(i16);

var_impl!(u8);
var_impl!(i8);

// ********** bool *********** //
impl Var<bool> {
    pub const fn new() -> Var<bool> {
        Var {
            value: SyncCell::new(false),
            forced_value: SyncCell::new(false),
            changed_value: SyncCell::new(false),
            forced: SyncCell::new(false),
            subscribed: SyncCell::new(SubscribeMode::Off),
            dirty: SyncCell::new(false),
            pos: SyncCell::new(false),
            neg: SyncCell::new(false),
        }
    }
}

impl crate::var::MemVar for crate::var::Var<bool> {
    unsafe fn to_buffer(&self, buffer: *mut u8, subvalue: u8) -> u8 {
        *(buffer as *mut u8) = match subvalue {
            0 => match self.get() {
              true => 1,
              false => 0,
            },
            1 => match self.value.get() {
              true => 1,
              false => 0,
            },
            2 => match self.changed_value.get() {
              true => 1,
              false => 0,
            },
            3 => match self.forced_value.get() {
              true => 1,
              false => 0,
            },
            _ => 0
        };
        1
    }

    unsafe fn from_buffer(&self, buffer: *const u8, subvalue: u8) -> u8 {
        match subvalue {
            0 => self.set(*(buffer as *mut u8) > 0),
            1 => self.value.set(*(buffer as *mut u8) > 0),
            2 => self.changed_value.set(*(buffer as *mut u8) > 0),
            3 => self.forced_value.set(*(buffer as *mut u8) > 0),
            _ => self.set(*(buffer as *mut u8) > 0),
        };
        1
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
        SubscribeMode::Current => 2
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

impl VarProps<bool> for Var<bool> {
    fn get(&self) -> bool {
        match self.forced.get() {
            true => self.forced_value.get(),
            false => self.value.get(),
        }
    }

    fn set(&self, value: bool) {
        if value == self.value.get() {
            self.pos.set(false);
            self.neg.set(false);
        } else {
            if value > self.value.get() {
                self.pos.set(true);
                self.neg.set(false);
            } else {
                self.pos.set(false);
                self.neg.set(true);
            }
            self.value.set(value);
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
