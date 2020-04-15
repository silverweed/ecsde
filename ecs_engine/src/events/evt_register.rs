use anymap::AnyMap;
use std::any::Any;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

pub trait Event {
    type Args: Clone;
}

pub type Event_Callback<T> = Box<dyn FnMut(<T as Event>::Args, Option<&mut Event_Callback_Data>)>;
pub type Event_Callback_Data = Arc<Mutex<dyn Any>>;

type Observers<T> = Vec<(Event_Callback<T>, Option<Event_Callback_Data>)>;

pub struct Event_Subscription_Handle<T> {
    idx: usize,
    _pd: PhantomData<T>,
}

impl<T> Debug for Event_Subscription_Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Event_Subscription_Handle<{:?}>({})",
            std::any::type_name::<T>(),
            self.idx
        )
    }
}

pub struct Event_Register {
    observers: AnyMap,
}

impl Event_Register {
    pub fn new() -> Self {
        Self {
            observers: AnyMap::new(),
        }
    }

    pub fn subscribe<E: 'static + Event>(
        &mut self,
        cb: Event_Callback<E>,
        cb_data: Option<Event_Callback_Data>,
    ) -> Event_Subscription_Handle<E> {
        let obs = self
            .observers
            .entry::<Observers<E>>()
            .or_insert_with(Vec::default);
        obs.push((cb, cb_data));
        Event_Subscription_Handle {
            idx: obs.len(),
            _pd: PhantomData,
        }
    }

    pub fn unsubscribe<E: 'static + Event>(&mut self, handle: Event_Subscription_Handle<E>) {
        let idx = handle.idx;
        if let Some(obs) = self.observers.get_mut::<Observers<E>>() {
            if obs.len() < idx {
                let _ = obs.remove(idx);
                return;
            }
        }

        lerr!("Tried to unsubscribe with invalid handle {:?}", handle);
    }

    pub fn raise<E: 'static + Event>(&mut self, args: E::Args) {
        if let Some(obs) = self.observers.get_mut::<Observers<E>>() {
            for (cb, cb_data) in obs {
                cb(args.clone(), cb_data.as_mut());
            }
        }
    }
}

pub fn wrap_cb_data<T: 'static>(data: T) -> Event_Callback_Data {
    Arc::new(Mutex::new(data))
}

pub fn with_cb_data<T: 'static, R, F: FnMut(&mut T) -> R>(
    data: &mut Event_Callback_Data,
    mut cb: F,
) -> R {
    let mut locked = data.lock().unwrap_or_else(|err| {
        fatal!(
            "unwrap_cb_data<{:?}>: failed to lock mutex: {:?}.",
            std::any::type_name::<T>(),
            err
        )
    });
    let downcasted = locked.downcast_mut::<T>().unwrap_or_else(|| {
        fatal!(
            "unwrap_cb_data<{:?}>: unwrapped data is not of the expected type.",
            std::any::type_name::<T>()
        )
    });
    cb(downcasted)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Evt_Test;

    impl Event for Evt_Test {
        type Args = (u32, i32);
    }

    #[test]
    fn no_cb_data() {
        let mut reg = Event_Register::new();
        reg.subscribe::<Evt_Test>(
            Box::new(|(uns, sgn), _| {
                println!("{} {}", uns, sgn);
            }),
            None,
        );

        reg.raise::<Evt_Test>((0, 0));
    }

    #[test]
    fn sub_raise_unsub() {
        let mut reg = Event_Register::new();
        let res: Arc<Mutex<Vec<()>>> = Arc::new(Mutex::new(vec![]));

        let evt_h = reg.subscribe::<Evt_Test>(
            Box::new(|(uns, sgn), res| {
                let mut res = res.unwrap().lock().unwrap();
                let res = res.downcast_mut::<Vec<()>>().unwrap();
                if uns == 42 {
                    res.push(());
                }
                if sgn == -48 {
                    res.push(());
                }
            }),
            Some(res.clone()),
        );

        reg.raise::<Evt_Test>((42, -48));
        reg.raise::<Evt_Test>((48, -42));

        reg.unsubscribe(evt_h);

        reg.raise::<Evt_Test>((42, -48));

        assert_eq!(res.lock().unwrap().len(), 2);
    }
}
