use anymap::AnyMap;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub trait Msg_Responder {
    type Msg_Data;
    type Resp_Data;

    fn send_message(&mut self, msg: Self::Msg_Data) -> Self::Resp_Data;
}

pub struct Msg_Dispatcher {
    responders: AnyMap,
}

impl Msg_Dispatcher {
    pub fn new() -> Msg_Dispatcher {
        Msg_Dispatcher {
            responders: AnyMap::new(),
        }
    }

    pub fn register<T>(&mut self, responder: Rc<RefCell<T>>)
    where
        T: Msg_Responder + 'static,
    {
        self.responders.insert::<Rc<RefCell<T>>>(responder);
    }

    pub fn borrow_mut<T>(&self) -> Option<RefMut<'_, T>>
    where
        T: Msg_Responder + 'static,
    {
        self.responders
            .get::<Rc<RefCell<T>>>()
            .map(|r| r.borrow_mut())
    }

    pub fn send_message<T>(&self, msg: T::Msg_Data) -> Option<T::Resp_Data>
    where
        T: Msg_Responder + 'static,
    {
        if let Some(responder) = self.responders.get::<Rc<RefCell<T>>>() {
            Some(responder.borrow_mut().send_message(msg))
        } else {
            None
        }
    }
}
