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

pub struct Session {
    pub session: ipc::Session
}

impl Session {
    pub fn from(session: ipc::Session) -> Self {
        Self { session: session }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.session.close();
    }
}

pub trait ISessionObject {
    fn new(session: Session) -> Self where Self: Sized;
    fn get_session(&mut self) -> &mut Session;

    fn get_inner_session(&mut self) -> ipc::Session {
        (*self.get_session()).session
    }

    fn convert_current_object_to_domain(&mut self) -> Result<()> {
        self.get_session().session.convert_current_object_to_domain()
    }

    fn query_pointer_buffer_size(&mut self) -> Result<u16> {
        self.get_session().session.query_pointer_buffer_size()
    }

    fn close_session(&mut self) {
        self.get_session().session.close()
    }

    fn is_valid(&mut self) -> bool {
        self.get_inner_session().is_valid()
    }
    
    fn is_domain(&mut self) -> bool {
        self.get_inner_session().is_domain()
    }
}

pub trait INamedPort {
    fn get_name() -> &'static str;
    fn post_initialize(&mut self) -> Result<()>;
}

pub trait IService {
    fn get_name() -> &'static str;
    fn as_domain() -> bool;
    fn post_initialize(&mut self) -> Result<()>;
}

pub fn new_named_port_object<T: ISessionObject + INamedPort + 'static>() -> Result<mem::Shared<T>> {
    let handle = svc::connect_to_named_port(T::get_name().as_ptr())?;
    let session = ipc::Session::from_handle(handle);
    let mut object = T::new(Session::from(session));
    object.post_initialize()?;
    Ok(mem::Shared::new(object))
}

pub fn new_service_object<T: ISessionObject + IService + 'static>() -> Result<mem::Shared<T>> {
    let sm = new_named_port_object::<sm::UserInterface>()?;
    let session_handle = sm.get().get_service(sm::ServiceName::new(T::get_name()))?;
    let session = ipc::Session::from_handle(session_handle.handle);
    let mut object = T::new(Session::from(session));
    object.post_initialize()?;
    if T::as_domain() {
        object.convert_current_object_to_domain()?;
    }
    Ok(mem::Shared::new(object))
}