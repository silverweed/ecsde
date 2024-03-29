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
            idx: obs.len() - 1,
            _pd: PhantomData,
        }
    }

    pub fn unsubscribe<E: 'static + Event>(&mut self, handle: Event_Subscription_Handle<E>) {
        let idx = handle.idx;
        if let Some(obs) = self.observers.get_mut::<Observers<E>>() {
            if idx < obs.len() {
                let _ = obs.remove(idx);
                return;
            }
        }

        fatal!("Tried to unsubscribe with invalid handle {:?}", handle);
    }

    pub fn raise<E: 'static + Event>(&mut self, args: &E::Args) {
        trace!("Event_Register::raise");

        if let Some(obs) = self.observers.get_mut::<Observers<E>>() {
            for (cb, cb_data) in obs {
                cb(args.clone(), cb_data.as_mut());
            }
        }
    }

    pub fn raise_batch<E: 'static + Event>(&mut self, args_batch: &[&E::Args]) {
        trace!("Event_Register::raise_batch");

        if args_batch.is_empty() {
            return;
        }

        if let Some(obs) = self.observers.get_mut::<Observers<E>>() {
            for (cb, cb_data) in obs {
                for args in args_batch {
                    cb((*args).clone(), cb_data.as_mut());
                }
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

        reg.raise::<Evt_Test>(&(0, 0));
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

        reg.raise::<Evt_Test>(&(42, -48));
        reg.raise::<Evt_Test>(&(48, -42));

        reg.unsubscribe(evt_h);

        reg.raise::<Evt_Test>(&(42, -48));

        assert_eq!(res.lock().unwrap().len(), 2);
    }

    #[test]
    fn use_with_cb_data() {
        let mut reg = Event_Register::new();
        reg.subscribe::<Evt_Test>(
            Box::new(|_, cb_data| {
                with_cb_data(cb_data.unwrap(), |data: &mut String| {
                    println!("{}", data);
                });
            }),
            Some(wrap_cb_data(String::from("Test"))),
        );

        reg.raise::<Evt_Test>(&(0, 0));
    }

    #[test]
    #[should_panic]
    fn use_with_cb_data_wrong() {
        let mut reg = Event_Register::new();
        reg.subscribe::<Evt_Test>(
            Box::new(|_, cb_data| {
                with_cb_data(cb_data.unwrap(), |data: &mut String| {
                    println!("{}", data);
                });
            }),
            Some(wrap_cb_data(42)),
        );

        reg.raise::<Evt_Test>(&(0, 0));
    }

    #[test]
    fn raise_batch() {
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

        let data = vec![&(42, -1), &(33, -48), &(42, -48), &(42, 42), &(0, 0)];
        reg.raise_batch::<Evt_Test>(&data);

        assert_eq!(res.lock().unwrap().len(), 5);
    }
}
