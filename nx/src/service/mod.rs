use crate::ipc;
use crate::alloc;
use crate::svc;
use crate::result::*;

pub mod sm;
use crate::service::sm::IUserInterface;

pub mod psm;

pub mod fspsrv;

pub trait SessionObject {
    fn new(session: ipc::Session) -> Self;
    fn get_session(&self) -> ipc::Session;

    fn is_valid(&self) -> bool {
        self.get_session().is_valid()
    }
    
    fn is_domain(&self) -> bool {
        self.get_session().is_domain()
    }

    fn convert_current_object_to_domain(&mut self) -> Result<()>;
    fn query_pointer_buffer_size(&mut self) -> Result<u16>;
    fn close(&mut self);
}

pub trait SharedSessionObject {
    fn shared(session: ipc::Session) -> alloc::SharedObject<Self>;
}

pub trait NamedPort {
    fn get_name() -> &'static str;
    fn post_initialize(&mut self) -> Result<()>;
}

pub trait Service {
    fn get_name() -> &'static str;
    fn as_domain() -> bool;
    fn post_initialize(&mut self) -> Result<()>;
}

impl SessionObject for ipc::Session {
    fn new(session: ipc::Session) -> Self {
        session
    }

    fn get_session(&self) -> ipc::Session {
        *self
    }

    fn convert_current_object_to_domain(&mut self) -> Result<()> {
        self.convert_current_object_to_domain()
    }

    fn query_pointer_buffer_size(&mut self) -> Result<u16> {
        self.query_pointer_buffer_size()
    }

    fn close(&mut self) {
        self.close()
    }
}

impl<T: SessionObject> SessionObject for alloc::SharedObject<T> {
    fn new(session: ipc::Session) -> Self {
        alloc::make_shared(T::new(session))
    }

    fn get_session(&self) -> ipc::Session {
        self.borrow_mut().get_session()
    }

    fn convert_current_object_to_domain(&mut self) -> Result<()> {
        self.borrow_mut().convert_current_object_to_domain()
    }
    fn query_pointer_buffer_size(&mut self) -> Result<u16> {
        self.borrow_mut().query_pointer_buffer_size()
    }

    fn close(&mut self) {
        self.borrow_mut().close()
    }
}

pub fn new_named_port_object<T: SessionObject + NamedPort>() -> Result<T> {
    let name = [T::get_name(), "\0"].join("");
    let handle = svc::connect_to_named_port(name.as_ptr())?;
    let session = ipc::Session::from_handle(handle);
    let mut object = T::new(session);
    object.post_initialize()?;
    Ok(object)
}

pub fn new_shared_named_port_object<T: SessionObject + SharedSessionObject + NamedPort>() -> Result<alloc::SharedObject<T>> {
    let object = new_named_port_object::<T>()?;
    let shared_object = alloc::make_shared(object);
    Ok(shared_object)
}

pub fn new_service_object<T: SessionObject + Service>() -> Result<T> {
    let name = [T::get_name(), "\0"].join("");
    let mut sm_session = new_named_port_object::<sm::UserInterface>()?;
    let session = sm_session.get_service(sm::ServiceName::new(&name))?;
    let mut object = T::new(session);
    object.post_initialize()?;
    if T::as_domain() {
        object.convert_current_object_to_domain()?;
    }
    Ok(object)
}

pub fn new_shared_service_object<T: SessionObject + SharedSessionObject + Service>() -> Result<alloc::SharedObject<T>> {
    let object = new_service_object::<T>()?;
    let shared_object = alloc::make_shared(object);
    Ok(shared_object)
}

#[macro_export]
macro_rules! session_object_define {
    ($name:ident) => {
        pub struct $name {
            session: ipc::Session,
        }
        
        impl SessionObject for $name {
            fn new(session: ipc::Session) -> Self {
                Self { session: session }
            }

            fn get_session(&self) -> ipc::Session {
                self.session
            }

            fn convert_current_object_to_domain(&mut self) -> Result<()> {
                self.session.convert_current_object_to_domain()
            }
            fn query_pointer_buffer_size(&mut self) -> Result<u16> {
                self.session.query_pointer_buffer_size()
            }
        
            fn close(&mut self) {
                self.session.close()
            }
        }

        impl SharedSessionObject for $name {
            fn shared(session: ipc::Session) -> alloc::SharedObject<Self> {
                alloc::SharedObject::new(core::cell::RefCell::new(Self::new(session)))
            }
        }
        
        impl core::ops::Drop for $name {
            fn drop(&mut self) {
                self.session.close();
            }
        }
    };
}