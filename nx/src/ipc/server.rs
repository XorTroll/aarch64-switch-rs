use crate::result::*;
use crate::results;
use crate::svc;
use crate::thread;
use crate::ipc::sf::IObject;
use crate::ipc::sf::hipc::IHipcManager;
use crate::ipc::sf::hipc::IMitmQueryServer;
use crate::service;
use crate::service::sm;
use crate::service::sm::IUserInterface;
use crate::mem;
use super::*;

extern crate alloc;
use alloc::vec::Vec;
use core::mem as cmem;

pub struct ServerContext {
    pub ctx: CommandContext,
    pub raw_data_walker: DataWalker,
    pub new_sessions: Vec<ServerHolder>
}

impl ServerContext {
    pub fn new(ctx: CommandContext, raw_data_walker: DataWalker) -> Self {
        Self { ctx: ctx, raw_data_walker: raw_data_walker, new_sessions: Vec::new() }
    }
}

#[inline(always)]
pub fn read_command_from_ipc_buffer(ctx: &mut CommandContext) -> CommandType {
    unsafe {
        let mut ipc_buf = get_ipc_buffer();

        let command_header = ipc_buf as *mut CommandHeader;
        ipc_buf = command_header.offset(1) as *mut u8;

        let command_type = (*command_header).get_command_type();
        let data_size = (*command_header).get_data_word_count() * 4;
        ctx.in_params.data_size = data_size;

        if (*command_header).get_has_special_header() {
            let special_header = ipc_buf as *mut CommandSpecialHeader;
            ipc_buf = special_header.offset(1) as *mut u8;

            ctx.in_params.send_process_id = (*special_header).get_send_process_id();
            if ctx.in_params.send_process_id {
                let process_id_ptr = ipc_buf as *mut u64;
                ctx.in_params.process_id = *process_id_ptr;
                ipc_buf = process_id_ptr.offset(1) as *mut u8;
            }

            ctx.in_params.copy_handle_count = (*special_header).get_copy_handle_count() as usize;
            ipc_buf = read_array_from_buffer(ipc_buf, ctx.in_params.copy_handle_count as u32, &mut ctx.in_params.copy_handles);
            ctx.in_params.move_handle_count = (*special_header).get_move_handle_count() as usize;
            ipc_buf = read_array_from_buffer(ipc_buf, ctx.in_params.move_handle_count as u32, &mut ctx.in_params.move_handles);
        }

        ctx.send_static_count = (*command_header).get_send_static_count() as usize;
        ipc_buf = read_array_from_buffer(ipc_buf, ctx.send_static_count as u32, &mut ctx.send_statics);
        ctx.send_buffer_count = (*command_header).get_send_buffer_count() as usize;
        ipc_buf = read_array_from_buffer(ipc_buf, ctx.send_buffer_count as u32, &mut ctx.send_buffers);
        ctx.receive_buffer_count = (*command_header).get_receive_buffer_count() as usize;
        ipc_buf = read_array_from_buffer(ipc_buf, ctx.receive_buffer_count as u32, &mut ctx.receive_buffers);
        ctx.exchange_buffer_count = (*command_header).get_exchange_buffer_count() as usize;
        ipc_buf = read_array_from_buffer(ipc_buf, ctx.exchange_buffer_count as u32, &mut ctx.exchange_buffers);

        ctx.in_params.data_words_offset = ipc_buf;
        ipc_buf = ipc_buf.offset(data_size as isize);

        ctx.receive_static_count = (*command_header).get_receive_static_count() as usize;
        /* ipc_buf = */ read_array_from_buffer(ipc_buf, ctx.receive_static_count as u32, &mut ctx.receive_statics);

        command_type
    }
}

#[inline(always)]
pub fn write_command_response_on_ipc_buffer(ctx: &mut CommandContext, command_type: CommandType, data_size: u32) {
    unsafe {
        let mut ipc_buf = get_ipc_buffer();
        
        let command_header = ipc_buf as *mut CommandHeader;
        ipc_buf = command_header.offset(1) as *mut u8;

        let data_word_count = (data_size + 3) / 4;
        let has_special_header = ctx.out_params.send_process_id || (ctx.out_params.copy_handle_count > 0) || (ctx.out_params.move_handle_count > 0);
        *command_header = CommandHeader::new(command_type, ctx.send_static_count as u32, ctx.send_buffer_count as u32, ctx.receive_buffer_count as u32, ctx.exchange_buffer_count as u32, data_word_count, ctx.receive_static_count as u32, has_special_header);

        if has_special_header {
            let special_header = ipc_buf as *mut CommandSpecialHeader;
            ipc_buf = special_header.offset(1) as *mut u8;

            *special_header = CommandSpecialHeader::new(ctx.out_params.send_process_id, ctx.out_params.copy_handle_count as u32, ctx.out_params.move_handle_count as u32);
            if ctx.out_params.send_process_id {
                ipc_buf = ipc_buf.offset(cmem::size_of::<u64>() as isize);
            }

            ipc_buf = write_array_to_buffer(ipc_buf, ctx.out_params.copy_handle_count as u32, &ctx.out_params.copy_handles);
            ipc_buf = write_array_to_buffer(ipc_buf, ctx.out_params.move_handle_count as u32, &ctx.out_params.move_handles);
        }

        ipc_buf = write_array_to_buffer(ipc_buf, ctx.send_static_count as u32, &ctx.send_statics);
        ipc_buf = write_array_to_buffer(ipc_buf, ctx.send_buffer_count as u32, &ctx.send_buffers);
        ipc_buf = write_array_to_buffer(ipc_buf, ctx.receive_buffer_count as u32, &ctx.receive_buffers);
        ipc_buf = write_array_to_buffer(ipc_buf, ctx.exchange_buffer_count as u32, &ctx.exchange_buffers);
        ctx.out_params.data_words_offset = ipc_buf;

        ipc_buf = ipc_buf.offset((data_word_count * 4) as isize);
        /* ipc_buf = */ write_array_to_buffer(ipc_buf, ctx.receive_static_count as u32, &ctx.receive_statics);
    }
}

#[inline(always)]
pub fn read_request_command_from_ipc_buffer(ctx: &mut CommandContext) -> Result<u32> {
    unsafe {
        let ipc_buf = get_ipc_buffer();
        let mut data_offset = get_aligned_data_offset(ctx.in_params.data_words_offset, ipc_buf);

        // TODO: out pointer

        let data_header = data_offset as *mut DataHeader;
        data_offset = data_header.offset(1) as *mut u8;

        // TODO: domain

        result_return_unless!((*data_header).magic == IN_DATA_HEADER_MAGIC, results::cmif::ResultInvalidInputHeader);
        let request_id = (*data_header).value;

        ctx.in_params.data_offset = data_offset;
        ctx.in_params.data_size -= DATA_PADDING + cmem::size_of::<DataHeader>() as u32;
        Ok(request_id)
    }
}

#[inline(always)]
pub fn write_request_command_response_on_ipc_buffer(ctx: &mut CommandContext, result: ResultCode) {
    unsafe {
        let ipc_buf = get_ipc_buffer();
        let mut data_size = DATA_PADDING + cmem::size_of::<DataHeader>() as u32 + ctx.out_params.data_size;
        // TODO: domain size
        data_size = (data_size + 1) & !1;
        // TODO: out pointer

        write_command_response_on_ipc_buffer(ctx, CommandType::Request, data_size);
        let mut data_offset = get_aligned_data_offset(ctx.out_params.data_words_offset, ipc_buf);

        // TODO: out pointer

        let data_header = data_offset as *mut DataHeader;
        data_offset = data_header.offset(1) as *mut u8;

        // TODO: domain

        *data_header = DataHeader::new(OUT_DATA_HEADER_MAGIC, 0, result.get_value(), 0);
        ctx.out_params.data_offset = data_offset;
    }
}

#[inline(always)]
pub fn read_control_command_from_ipc_buffer(ctx: &mut CommandContext) -> Result<ControlRequestId> {
    unsafe {
        let ipc_buf = get_ipc_buffer();
        let mut data_offset = get_aligned_data_offset(ctx.in_params.data_words_offset, ipc_buf);

        let data_header = data_offset as *mut DataHeader;
        data_offset = data_header.offset(1) as *mut u8;

        result_return_unless!((*data_header).magic == IN_DATA_HEADER_MAGIC, results::cmif::ResultInvalidInputHeader);
        let control_request_id: ControlRequestId = cmem::transmute((*data_header).value);

        ctx.in_params.data_offset = data_offset;
        ctx.in_params.data_size -= DATA_PADDING + cmem::size_of::<DataHeader>() as u32;
        Ok(control_request_id)
    }
}

#[inline(always)]
pub fn write_control_command_response_on_ipc_buffer(ctx: &mut CommandContext, result: ResultCode) {
    unsafe {
        let ipc_buf = get_ipc_buffer();
        let mut data_size = DATA_PADDING + cmem::size_of::<DataHeader>() as u32 + ctx.out_params.data_size;
        data_size = (data_size + 1) & !1;

        write_command_response_on_ipc_buffer(ctx, CommandType::Control, data_size);
        let mut data_offset = get_aligned_data_offset(ctx.out_params.data_words_offset, ipc_buf);

        let data_header = data_offset as *mut DataHeader;
        data_offset = data_header.offset(1) as *mut u8;

        *data_header = DataHeader::new(OUT_DATA_HEADER_MAGIC, 0, result.get_value(), 0);
        ctx.out_params.data_offset = data_offset;
    }
}

#[inline(always)]
pub fn write_close_command_response_on_ipc_buffer(ctx: &mut CommandContext) {
    write_command_response_on_ipc_buffer(ctx, CommandType::Close, 0);
}

pub trait CommandParameter<O> {
    fn after_request_read(ctx: &mut ServerContext) -> Result<O>;
    fn before_response_write(var: &Self, ctx: &mut ServerContext) -> Result<()>;
    fn after_response_write(var: &Self, ctx: &mut ServerContext) -> Result<()>;
}

impl<T: Copy> CommandParameter<T> for T {
    default fn after_request_read(ctx: &mut ServerContext) -> Result<Self> {
        Ok(ctx.raw_data_walker.advance_get())
    }

    default fn before_response_write(_raw: &Self, ctx: &mut ServerContext) -> Result<()> {
        ctx.raw_data_walker.advance::<Self>();
        Ok(())
    }

    default fn after_response_write(raw: &Self, ctx: &mut ServerContext) -> Result<()> {
        ctx.raw_data_walker.advance_set(*raw);
        Ok(())
    }
}

// TODO: support these

impl<const A: BufferAttribute> CommandParameter<sf::Buffer<A>> for sf::Buffer<A> {
    fn after_request_read(ctx: &mut ServerContext) -> Result<Self> {
        ctx.ctx.pop_buffer()
    }

    fn before_response_write(_buffer: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }

    fn after_response_write(_buffer: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }
}

impl<const M: HandleMode> CommandParameter<sf::Handle<M>> for sf::Handle<M> {
    fn after_request_read(_ctx: &mut ServerContext) -> Result<Self> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }

    fn before_response_write(_handle: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }

    fn after_response_write(_handle: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }
}

impl CommandParameter<sf::ProcessId> for sf::ProcessId {
    fn after_request_read(ctx: &mut ServerContext) -> Result<Self> {
        if ctx.ctx.in_params.send_process_id {
            // TODO: is this really how process ID works? (is the in raw u64 just placeholder data?)
            let _ = ctx.raw_data_walker.advance_get::<u64>();
            Ok(sf::ProcessId::from(ctx.ctx.in_params.process_id)) 
        }
        else {
            Err(results::hipc::ResultUnsupportedOperation::make())
        }
    }

    fn before_response_write(_process_id: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Ok(())
    }

    fn after_response_write(_process_id: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Ok(())
    }
}

/*
impl CommandParameter<mem::Shared<dyn sf::IObject>> for mem::Shared<dyn sf::IObject> {
    fn after_request_read(_ctx: &mut ServerContext) -> Result<mem::Shared<dyn sf::IObject>> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }

    fn before_response_write(session: &Self, ctx: &mut ServerContext) -> Result<()> {
        if ctx.ctx.object_info.is_domain() {
            Err(results::hipc::ResultUnsupportedOperation::make())
        }
        else {
            let (server_handle, client_handle) = svc::create_session(false, 0)?;
            let handle: sf::MoveHandle = sf::Handle::from(client_handle);
            ctx.ctx.out_params.push_handle(handle)?;
            ctx.new_sessions.push(ServerHolder::new_session(server_handle, session));
            Ok(())
        }
    }

    fn after_response_write(_session: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Ok(())
    }
}
*/

impl CommandParameter<mem::Shared<dyn sf::IObject>> for mem::Shared<dyn sf::IObject> {
    fn after_request_read(_ctx: &mut ServerContext) -> Result<Self> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }

    fn before_response_write(session: &Self, ctx: &mut ServerContext) -> Result<()> {
        if ctx.ctx.object_info.is_domain() {
            Err(results::hipc::ResultUnsupportedOperation::make())
        }
        else {
            let (server_handle, client_handle) = svc::create_session(false, 0)?;
            let handle: sf::MoveHandle = sf::Handle::from(client_handle);
            ctx.ctx.out_params.push_handle(handle)?;
            ctx.new_sessions.push(ServerHolder::new_session(server_handle, session.clone()));
            Ok(())
        }
    }

    fn after_response_write(_session: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Ok(())
    }
}

/*
impl<S: IServer + 'static> CommandParameter<mem::Shared<dyn sf::IObject>> for mem::Shared<S> {
    fn after_request_read(_ctx: &mut ServerContext) -> Result<mem::Shared<dyn sf::IObject>> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }

    fn before_response_write(session: &Self, ctx: &mut ServerContext) -> Result<()> {
        if ctx.ctx.object_info.is_domain() {
            Err(results::hipc::ResultUnsupportedOperation::make())
        }
        else {
            let (server_handle, client_handle) = svc::create_session(false, 0)?;
            let handle: sf::MoveHandle = sf::Handle::from(client_handle);
            ctx.ctx.out_params.push_handle(handle)?;
            ctx.new_sessions.push(ServerHolder::new_session(server_handle, session.clone()));
            Ok(())
        }
    }

    fn after_response_write(_session: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Ok(())
    }
}
*/

pub trait IServerObject: sf::IObject {
    fn new(session: sf::Session) -> Self where Self: Sized;
}

fn create_server_object_impl<S: IServerObject + 'static>(object_info: ObjectInfo) -> mem::Shared<dyn sf::IObject> {
    mem::Shared::new(S::new(sf::Session::from(object_info)))
}

pub type NewServerFn = fn(ObjectInfo) -> mem::Shared<dyn sf::IObject>;

pub enum WaitHandleType {
    Server,
    Session
}

pub struct ServerHolder {
    pub server: Option<mem::Shared<dyn sf::IObject>>,
    pub new_server_fn: Option<NewServerFn>,
    pub handle_type: WaitHandleType,
    pub forward_handle: svc::Handle,
    pub is_mitm_service: bool,
    pub service_name: sm::ServiceName
}

impl ServerHolder {
    pub fn empty() -> Self {
        Self { server: None, new_server_fn: None, handle_type: WaitHandleType::Server, forward_handle: 0, is_mitm_service: false, service_name: sm::ServiceName::empty() } 
    }

    pub fn new_server_session<S: IServerObject + 'static>(handle: svc::Handle) -> Self {
        let session = sf::Session::from_handle(handle);
        Self { server: Some(mem::Shared::new(S::new(session))), new_server_fn: None, handle_type: WaitHandleType::Session, forward_handle: 0, is_mitm_service: false, service_name: sm::ServiceName::empty() } 
    }

    pub fn new_session(handle: svc::Handle, object: mem::Shared<dyn sf::IObject>) -> Self {
        *object.get().get_session() = sf::Session::from_handle(handle);
        Self { server: Some(object), new_server_fn: None, handle_type: WaitHandleType::Session, forward_handle: 0, is_mitm_service: false, service_name: sm::ServiceName::empty() } 
    }
    
    pub fn new_server<S: IServerObject + 'static>(handle: svc::Handle, service_name: sm::ServiceName, is_mitm_service: bool) -> Self {
        let session = sf::Session::from_handle(handle);
        Self { server: Some(mem::Shared::new(S::new(session))), new_server_fn: Some(create_server_object_impl::<S>), handle_type: WaitHandleType::Server, forward_handle: 0, is_mitm_service: is_mitm_service, service_name: service_name } 
    }

    pub fn make_new_session(&self, handle: svc::Handle, forward_handle: svc::Handle) -> Result<Self> {
        let new_fn = self.get_new_server_fn()?;
        let object_info = ObjectInfo::from_handle(handle);
        Ok(Self { server: Some((new_fn)(object_info)), new_server_fn: Some(new_fn), handle_type: WaitHandleType::Session, forward_handle: forward_handle, is_mitm_service: false, service_name: sm::ServiceName::empty() })
    }

    pub fn get_server(&mut self) -> Result<&mut mem::Shared<dyn sf::IObject>> {
        match &mut self.server {
            Some(server) => Ok(server),
            None => Err(results::hipc::ResultSessionClosed::make())
        }
    }

    pub fn get_server_info(&mut self) -> Result<ObjectInfo> {
        match &mut self.server {
            Some(server) => Ok(server.get().get_info()),
            None => Err(results::hipc::ResultSessionClosed::make())
        }
    }

    pub fn get_new_server_fn(&self) -> Result<NewServerFn> {
        match self.new_server_fn {
            Some(new_server_fn) => Ok(new_server_fn),
            None => Err(results::hipc::ResultSessionClosed::make())
        }
    }
}

#[allow(dead_code)]
pub struct HipcManager<'a> {
    session: sf::Session,
    server: &'a ServerHolder
}

impl<'a> HipcManager<'a> {
    pub fn new(server: &'a ServerHolder) -> Self {
        Self { session: sf::Session::new(), server: server }
    }
}

// TODO: implement control commands (currently stubbed calls)

impl<'a> IHipcManager for HipcManager<'a> {
    fn convert_current_object_to_domain(&mut self) -> Result<DomainObjectId> {
        Err(ResultCode::new(0xBAD))
    }

    fn copy_from_current_domain(&mut self, _domain_object_id: DomainObjectId) -> Result<sf::MoveHandle> {
        Err(ResultCode::new(0xBAD))
    }

    fn clone_current_object(&mut self) -> Result<sf::MoveHandle> {
        Err(ResultCode::new(0xBAD))
    }

    fn query_pointer_buffer_size(&mut self) -> Result<u16> {
        Ok(0)
    }

    fn clone_current_object_ex(&mut self, _tag: u32) -> Result<sf::MoveHandle> {
        Err(ResultCode::new(0xBAD))
    }
}

impl<'a> sf::IObject for HipcManager<'a> {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table!(
            convert_current_object_to_domain: 0,
            copy_from_current_domain: 1,
            clone_current_object: 2,
            query_pointer_buffer_size: 3,
            clone_current_object_ex: 4
        )
    }
}

pub struct ServerContainer {
    pub server_holders: Vec<ServerHolder>
}

impl ServerContainer {
    pub fn new(server_holder: ServerHolder) -> Self {
        Self { server_holders: vec![server_holder] } 
    }

    pub fn get_handle_index(&mut self, handle: svc::Handle) -> Option<usize> {
        let mut i: usize = 0;
        for server_holder in self.server_holders.iter_mut() {
            match server_holder.get_server_info() {
                Ok(object_info) => {
                    if object_info.handle == handle {
                        return Some(i);
                    }
                },
                _ => {}
            }
            i += 1;
        }
        None
    }
    
    pub fn process_signaled_handle(&mut self, index: usize) -> Result<()> {
        let mut new_sessions: Vec<ServerHolder> = Vec::new();
        let mut should_close_session = false;

        match self.server_holders.get_mut(index) {
            None => return Err(results::hipc::ResultSessionClosed::make()),
            Some(server_holder) => {
                let server_holder_info = server_holder.get_server_info()?;
                match server_holder.handle_type {
                    WaitHandleType::Session => {
                        svc::reply_and_receive(&server_holder_info.handle as *const svc::Handle, 1, 0, -1)?;
                        let fwd_handle = server_holder.forward_handle;
                        let is_mitm = fwd_handle != 0;

                        let mut ipc_buf_backup: [u8; 0x100] = [0; 0x100];
                        if is_mitm {
                            let ipc_buf = get_ipc_buffer();
                            unsafe {
                                core::ptr::copy(ipc_buf, ipc_buf_backup.as_mut_ptr(), ipc_buf_backup.len());
                            }
                        }

                        let mut ctx = ServerContext::new(CommandContext::new(server_holder_info), DataWalker::empty());
                        let command_type = read_command_from_ipc_buffer(&mut ctx.ctx);
                        match command_type {
                            CommandType::Request => {
                                match read_request_command_from_ipc_buffer(&mut ctx.ctx) {
                                    Ok(rq_id) => {
                                        match server_holder.get_server() {
                                            Ok(server) => {
                                                // Nothing done on success here, as if the command succeeds it will automatically respond by itself.
                                                let mut command_found = false;
                                                for command in server.get().get_command_table() {
                                                    if command.rq_id == rq_id {
                                                        command_found = true;
                                                        if let Err(rc) = server.get().call_self_command(command.command_fn, &mut ctx) {
                                                            if is_mitm && results::sm::mitm::ResultShouldForwardToSession::matches(rc) {
                                                                let ipc_buf = get_ipc_buffer();
                                                                unsafe {
                                                                    core::ptr::copy(ipc_buf_backup.as_ptr(), ipc_buf, ipc_buf_backup.len());
                                                                }
                                                                // Let the original service take care of the command for us.
                                                                if let Err(rc) = svc::send_sync_request(fwd_handle) {
                                                                    write_request_command_response_on_ipc_buffer(&mut ctx.ctx, rc);
                                                                }
                                                            }
                                                            else {
                                                                write_request_command_response_on_ipc_buffer(&mut ctx.ctx, rc);
                                                            }
                                                        }
                                                    }
                                                }
                                                if !command_found {
                                                    if is_mitm {
                                                        let ipc_buf = get_ipc_buffer();
                                                        unsafe {
                                                            core::ptr::copy(ipc_buf_backup.as_ptr(), ipc_buf, ipc_buf_backup.len());
                                                        }
                                                        // Let the original service take care of the command for us.
                                                        if let Err(rc) = svc::send_sync_request(fwd_handle) {
                                                            write_request_command_response_on_ipc_buffer(&mut ctx.ctx, rc);
                                                        }
                                                    }
                                                    else {
                                                        write_request_command_response_on_ipc_buffer(&mut ctx.ctx, results::cmif::ResultInvalidCommandRequestId::make());
                                                    }
                                                }
                                            },
                                            Err(rc) => write_request_command_response_on_ipc_buffer(&mut ctx.ctx, rc),
                                        }
                                    }
                                    Err(rc) => write_request_command_response_on_ipc_buffer(&mut ctx.ctx, rc),
                                };
                            },
                            CommandType::Control => {
                                match read_control_command_from_ipc_buffer(&mut ctx.ctx) {
                                    Ok(control_rq_id) => {
                                        let mut hipc_manager = HipcManager::new(server_holder);
                                        // Nothing done on success here, as if the command succeeds it will automatically respond by itself.
                                        let mut command_found = false;
                                        for command in hipc_manager.get_command_table() {
                                            if command.rq_id == control_rq_id as u32 {
                                                command_found = true;
                                                if let Err(rc) = hipc_manager.call_self_command(command.command_fn, &mut ctx) {
                                                    write_control_command_response_on_ipc_buffer(&mut ctx.ctx, rc);
                                                }
                                            }
                                        }
                                        if !command_found {
                                            write_control_command_response_on_ipc_buffer(&mut ctx.ctx, results::cmif::ResultInvalidCommandRequestId::make());
                                        }
                                    }
                                    Err(rc) => write_control_command_response_on_ipc_buffer(&mut ctx.ctx, rc),
                                };
                            }
                            CommandType::Close => {
                                should_close_session = true;
                                write_close_command_response_on_ipc_buffer(&mut ctx.ctx);
                            },
                            _ => {
                                todo!();
                            }
                        }
                        
                        match svc::reply_and_receive(&server_holder_info.handle as *const svc::Handle, 0, server_holder_info.handle, 0) {
                            Err(rc) => {
                                if !results::os::ResultTimeout::matches(rc) {
                                    return Err(rc);
                                }
                            },
                            _ => {}
                        };

                        new_sessions.append(&mut ctx.new_sessions);
                    },
                    WaitHandleType::Server => {
                        let new_handle = svc::accept_session(server_holder_info.handle)?;
                        let mut forward_handle: svc::Handle = 0;

                        if server_holder.is_mitm_service {
                            let sm = service::new_named_port_object::<sm::UserInterface>()?;
                            let (_info, session_handle) = sm.get().atmosphere_acknowledge_mitm_session(server_holder.service_name)?;
                            forward_handle = session_handle.handle;
                        }

                        new_sessions.push(server_holder.make_new_session(new_handle, forward_handle)?);
                    },
                };
            }
        };

        if should_close_session {
            self.server_holders.remove(index);
        }
        else {
            for session in new_sessions {
                self.server_holders.push(session);
            }
        }

        Ok(())
    }
}

pub struct MitmQueryServer<S: MitmService> {
    session: sf::Session,
    phantom: core::marker::PhantomData<S>
}

impl<S: MitmService> IMitmQueryServer for MitmQueryServer<S> {
    fn should_mitm(&mut self, info: sm::MitmProcessInfo) -> Result<bool> {
        Ok(S::should_mitm(info))
    }
}

impl<S: MitmService> sf::IObject for MitmQueryServer<S> {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table!(
            should_mitm: 65000
        )
    }
}

impl<S: MitmService> IServerObject for MitmQueryServer<S> {
    fn new(session: sf::Session) -> Self {
        Self { session: session, phantom: core::marker::PhantomData }
    }
}

pub trait IService: IServerObject {
    fn get_name() -> &'static str;
    fn get_max_sesssions() -> i32;
}

pub trait MitmService: IServerObject {
    fn get_name() -> &'static str;
    fn should_mitm(info: sm::MitmProcessInfo) -> bool;
}

pub trait INamedPort: IServerObject {
    fn get_port_name() -> &'static str;
    fn get_max_sesssions() -> i32;
}

pub struct ServerManager {
    server_containers: Vec<ServerContainer>,
    wait_handles: [svc::Handle; 0x40],
    mitm_query_sessions: Vec<ServerContainer>,
    mitm_query_thread: thread::Thread,
}

impl ServerManager {
    pub fn new() -> Self {
        Self { server_containers: Vec::new(), wait_handles: [0; 0x40], mitm_query_sessions: Vec::new(), mitm_query_thread: thread::Thread::empty() }
    }

    fn mitm_query_thread_impl(query_sessions_arg: *mut u8) {
        unsafe {
            let query_sessions = query_sessions_arg as *mut Vec<ServerContainer>;

            let mut query_manager = ServerManager::new();
            query_manager.server_containers = Vec::from_raw_parts((*query_sessions).as_mut_ptr(), (*query_sessions).len(), (*query_sessions).len());
            let _ = query_manager.loop_process();
        }
    }
    
    fn prepare_wait_handles(&mut self) -> (*mut svc::Handle, u32) {
        let mut i: usize = 0;
        for server_container in self.server_containers.iter_mut() {
            for server in server_container.server_holders.iter_mut() {
                match server.get_server_info() {
                    Ok(object_info) => {
                        if object_info.handle != 0 {
                            if i < 0x40 {
                                self.wait_handles[i] = object_info.handle;
                                i += 1;
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        (self.wait_handles.as_mut_ptr(), i as u32)
    }

    pub fn register_container(&mut self, container: ServerContainer) {
        self.server_containers.push(container);
    }
    
    pub fn register_server<S: IServerObject + 'static>(&mut self, handle: svc::Handle, service_name: sm::ServiceName, is_mitm_service: bool) {
        let server = ServerHolder::new_server::<S>(handle, service_name, is_mitm_service);
        let container = ServerContainer::new(server);
        self.server_containers.push(container);
    }
    
    pub fn register_session<S: IServerObject + 'static>(&mut self, handle: svc::Handle) {
        let server = ServerHolder::new_server_session::<S>(handle);
        let container = ServerContainer::new(server);
        self.server_containers.push(container);
    }

    pub fn register_mitm_query_session<S: MitmService + 'static>(&mut self, query_handle: svc::Handle) {
        let server = ServerHolder::new_server_session::<MitmQueryServer<S>>(query_handle);
        let container = ServerContainer::new(server);
        self.mitm_query_sessions.push(container);
    }
    
    pub fn register_service_server<S: IService + 'static>(&mut self) -> Result<()> {
        let service_name = sm::ServiceName::new(S::get_name());
        
        let service_handle = {
            let sm = service::new_named_port_object::<sm::UserInterface>()?;
            sm.get().register_service(service_name, false, S::get_max_sesssions())?
        };

        self.register_server::<S>(service_handle.handle, service_name, false);
        Ok(())
    }
    
    pub fn register_mitm_service_server<S: MitmService + 'static>(&mut self) -> Result<()> {
        let service_name = sm::ServiceName::new(S::get_name());

        let (mitm_handle, query_handle) = {
            let sm = service::new_named_port_object::<sm::UserInterface>()?;
            sm.get().atmosphere_install_mitm(service_name)?
        };

        self.register_server::<S>(mitm_handle.handle, service_name, true);
        self.register_mitm_query_session::<S>(query_handle.handle);
        Ok(())
    }

    pub fn register_named_port_server<S: INamedPort + 'static>(&mut self) -> Result<()> {
        let port_handle = svc::manage_named_port(S::get_port_name().as_ptr(), S::get_max_sesssions())?;

        self.register_server::<S>(port_handle, sm::ServiceName::empty(), false);
        Ok(())
    }

    fn process_impl(&mut self) -> Result<()> {
        let (handles, handle_count) = self.prepare_wait_handles();
        let index = svc::wait_synchronization(handles, handle_count, -1)?;

        let signaled_handle = self.wait_handles[index as usize];
        for server_container in self.server_containers.iter_mut() {
            if let Some(handle_index) = server_container.get_handle_index(signaled_handle) {
                server_container.process_signaled_handle(handle_index)?;
            }
        }

        Ok(())
    }

    pub fn loop_process(&mut self) -> Result<()> {
        if !self.mitm_query_sessions.is_empty() {
            self.mitm_query_thread = thread::Thread::new(Self::mitm_query_thread_impl, &mut self.mitm_query_sessions as *mut _ as *mut u8, ptr::null_mut(), 0x4000, "MitmQueryThread")?;
            self.mitm_query_thread.create_and_start(20, -2)?;
        }

        loop {
            match self.process_impl() {
                Err(rc) => {
                    // TODO: handle results properly here
                    if results::os::ResultOperationCanceled::matches(rc) {
                        break;
                    }
                },
                _ => {}
            }
        }

        Ok(())
    }
}