use super::*;
use crate::svc;
use crate::ipc;
use crate::ipc::client;
use crate::ipc::server;
use core::mem;
use alloc::vec::Vec;

#[derive(Copy, Clone)]
pub struct Buffer<const A: BufferAttribute> {
    pub buf: *const u8,
    pub size: usize
}

impl<const A: BufferAttribute> Buffer<A> {
    pub const fn from_const<T>(buf: *const T, size: usize) -> Self {
        Self { buf: buf as *const u8, size: size }
    }

    pub const fn from_mut<T>(buf: *mut T, size: usize) -> Self {
        Self { buf: buf as *const u8, size: size }
    }

    pub const fn from_var<T>(var: &T) -> Self {
        Self::from_const(var as *const T, mem::size_of::<T>())
    }

    pub const fn from_array<T>(arr: &[T]) -> Self {
        Self::from_const(arr.as_ptr(), arr.len() * mem::size_of::<T>())
    }

    pub const fn get_as<T>(&self) -> &T {
        unsafe {
            &*(self.buf as *const T)
        }
    }

    pub fn get_mut_as<T>(&self) -> &mut T {
        unsafe {
            &mut *(self.buf as *mut T)
        }
    }
}

pub type InMapAliasBuffer = Buffer<{bit_group!{ BufferAttribute [In, MapAlias] }}>;
pub type OutMapAliasBuffer = Buffer<{bit_group!{ BufferAttribute [Out, MapAlias] }}>;
pub type InAutoSelectBuffer = Buffer<{bit_group!{ BufferAttribute [In, AutoSelect] }}>;
pub type OutAutoSelectBuffer = Buffer<{bit_group!{ BufferAttribute [Out, AutoSelect] }}>;
pub type InPointerBuffer = Buffer<{bit_group!{ BufferAttribute [In, Pointer] }}>;
pub type OutPointerBuffer = Buffer<{bit_group!{ BufferAttribute [Out, Pointer] }}>;

#[derive(Copy, Clone)]
pub struct Handle<const M: HandleMode> {
    pub handle: svc::Handle
}

impl<const M: HandleMode> Handle<M> {
    pub const fn from(handle: svc::Handle) -> Self {
        Self { handle: handle }
    }
}

pub type CopyHandle = Handle<{HandleMode::Copy}>;
pub type MoveHandle = Handle<{HandleMode::Move}>;

#[derive(Copy, Clone)]
pub struct ProcessId {
    pub process_id: u64
}

impl ProcessId {
    pub const fn from(process_id: u64) -> Self {
        Self { process_id: process_id }
    }

    pub const fn new() -> ProcessId {
        Self::from(0)
    }
}

pub struct Session {
    pub object_info: ObjectInfo
}

impl Session {
    pub fn new() -> Self  {
        Self { object_info: ObjectInfo::new() }
    }

    pub fn from(object_info: ObjectInfo) -> Self {
        Self { object_info: object_info }
    }
    
    pub fn from_handle(handle: svc::Handle) -> Self {
        Self::from(ObjectInfo::from_handle(handle))
    }

    pub fn convert_to_domain(&mut self) -> Result<()> {
        self.object_info.domain_object_id = self.object_info.convert_current_object_to_domain()?;
        Ok(())
    }

    pub fn get_info(&mut self) -> &mut ObjectInfo {
        &mut self.object_info
    }

    pub fn close(&mut self) {
        if self.object_info.is_valid() {
            if self.object_info.is_domain() {
                let mut ctx = CommandContext::new(self.object_info);
                client::write_request_command_on_ipc_buffer(&mut ctx, None, DomainCommandType::Close);
                let _ = svc::send_sync_request(self.object_info.handle);
            }
            else if self.object_info.owns_handle {
                let mut ctx = CommandContext::new(self.object_info);
                client::write_close_command_on_ipc_buffer(&mut ctx);
                let _ = svc::send_sync_request(self.object_info.handle);
            }
            if self.object_info.owns_handle {
                let _ = svc::close_handle(self.object_info.handle);
            }
            self.object_info = ObjectInfo::new();
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}

pub type CommandFn = fn(&mut dyn IObject, &mut server::ServerContext) -> Result<()>;
pub type CommandSpecificFn<T> = fn(&mut T, &mut server::ServerContext) -> Result<()>;

pub struct CommandMetadata {
    pub rq_id: u32,
    pub command_fn: CommandFn
}

pub type CommandMetadataTable = Vec<CommandMetadata>;

impl CommandMetadata {
    pub fn new(rq_id: u32, command_fn: CommandFn) -> Self {
        Self { rq_id: rq_id, command_fn: command_fn }
    }
}

// This trait is analogous to N's IServiceObject type - the base for any kind of IPC interface
// IClientObject (on service module) and IServerObject (on server module) are wrappers for some specific kind of objects

pub trait IObject {
    fn get_session(&mut self) -> &mut Session;
    fn get_command_table(&self) -> CommandMetadataTable;

    fn get_info(&mut self) -> ipc::ObjectInfo {
        self.get_session().object_info
    }

    fn convert_to_domain(&mut self) -> Result<()> {
        self.get_session().convert_to_domain()
    }

    fn query_pointer_buffer_size(&mut self) -> Result<u16> {
        self.get_info().query_pointer_buffer_size()
    }

    fn close_session(&mut self) {
        self.get_session().close()
    }

    fn is_valid(&mut self) -> bool {
        self.get_info().is_valid()
    }
    
    fn is_domain(&mut self) -> bool {
        self.get_info().is_domain()
    }

    fn call_self_command(&mut self, command_fn: CommandFn, ctx: &mut server::ServerContext) -> Result<()> {
        let original_fn: CommandSpecificFn<Self> = unsafe { mem::transmute(command_fn) };
        (original_fn)(self, ctx)
    }
}

pub mod sm;

pub mod psm;

pub mod applet;

pub mod lm;

pub mod fatal;

pub mod dispdrv;

pub mod fspsrv;

pub mod hid;

pub mod nv;

pub mod vi;

pub mod hipc;