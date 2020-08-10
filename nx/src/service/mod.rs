use crate::ipc;
use crate::mem;
use crate::svc;
use crate::result::*;

pub mod sm;
use crate::service::sm::IUserInterface;

pub mod psm;

pub mod fspsrv;

pub mod lm;

pub mod vi;

pub mod nv;

pub mod dispdrv;

pub mod fatal;

pub mod hid;

pub mod applet;

pub trait SessionObject {
    fn new(session: ipc::Session) -> Self;
    fn get_session(&self) -> ipc::Session;
    fn convert_current_object_to_domain(&mut self) -> Result<()>;
    fn query_pointer_buffer_size(&mut self) -> Result<u16>;
    fn close(&mut self);

    fn is_valid(&self) -> bool {
        self.get_session().is_valid()
    }
    
    fn is_domain(&self) -> bool {
        self.get_session().is_domain()
    }
}

pub trait SharedSessionObject {
    fn shared(session: ipc::Session) -> mem::SharedObject<Self>;
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

impl<T: SessionObject> SessionObject for mem::SharedObject<T> {
    fn new(session: ipc::Session) -> Self {
        mem::make_shared(T::new(session))
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
    let handle = svc::connect_to_named_port(T::get_name().as_ptr())?;
    let session = ipc::Session::from_handle(handle);
    let mut object = T::new(session);
    object.post_initialize()?;
    Ok(object)
}

pub fn new_shared_named_port_object<T: SessionObject + SharedSessionObject + NamedPort>() -> Result<mem::SharedObject<T>> {
    let object = new_named_port_object::<T>()?;
    let shared_object = mem::make_shared(object);
    Ok(shared_object)
}

pub fn new_service_object<T: SessionObject + Service>() -> Result<T> {
    let mut sm_session = new_named_port_object::<sm::UserInterface>()?;
    let session = sm_session.get_service(sm::ServiceName::new(T::get_name()))?;
    let mut object = T::new(session);
    object.post_initialize()?;
    if T::as_domain() {
        object.convert_current_object_to_domain()?;
    }
    Ok(object)
}

pub fn new_shared_service_object<T: SessionObject + SharedSessionObject + Service>() -> Result<mem::SharedObject<T>> {
    let object = new_service_object::<T>()?;
    let shared_object = mem::make_shared(object);
    Ok(shared_object)
}