use crate::result::*;
use crate::results;
use crate::svc;
use crate::thread;
use crate::service;
use crate::service::sm;
use crate::service::sm::IUserInterface;
use super::*;

extern crate alloc;
use alloc::vec::Vec;
use alloc::boxed::Box;

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
                ipc_buf = ipc_buf.offset(mem::size_of::<u64>() as isize);
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

        // out pointer

        let data_header = data_offset as *mut DataHeader;
        data_offset = data_header.offset(1) as *mut u8;

        // domain

        result_return_unless!((*data_header).magic == IN_DATA_HEADER_MAGIC, results::cmif::ResultInvalidInputHeader);
        let request_id = (*data_header).value;

        ctx.in_params.data_offset = data_offset;
        ctx.in_params.data_size -= DATA_PADDING + mem::size_of::<DataHeader>() as u32;
        Ok(request_id)
    }
}

#[inline(always)]
pub fn write_request_command_response_on_ipc_buffer(ctx: &mut CommandContext, result: ResultCode) {
    unsafe {
        let ipc_buf = get_ipc_buffer();
        let mut data_size = DATA_PADDING + mem::size_of::<DataHeader>() as u32 + ctx.out_params.data_size;
        // domain size
        data_size = (data_size + 1) & !1;
        // out pointer

        write_command_response_on_ipc_buffer(ctx, CommandType::Request, data_size);
        let mut data_offset = get_aligned_data_offset(ctx.out_params.data_words_offset, ipc_buf);

        // out pointer

        let data_header = data_offset as *mut DataHeader;
        data_offset = data_header.offset(1) as *mut u8;

        // domain

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
        let control_request_id: ControlRequestId = mem::transmute((*data_header).value);

        ctx.in_params.data_offset = data_offset;
        ctx.in_params.data_size -= DATA_PADDING + mem::size_of::<DataHeader>() as u32;
        Ok(control_request_id)
    }
}

#[inline(always)]
pub fn write_control_command_response_on_ipc_buffer(ctx: &mut CommandContext, result: ResultCode) {
    unsafe {
        let ipc_buf = get_ipc_buffer();
        let mut data_size = DATA_PADDING + mem::size_of::<DataHeader>() as u32 + ctx.out_params.data_size;
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

pub type CommandFn = fn(&mut dyn Server, &mut CommandContext) -> Result<()>;
pub type CommandOrigFn<T> = fn(&mut T, &mut CommandContext) -> Result<()>;

pub struct CommandMetadata {
    pub id: u32,
    pub func: CommandFn
}

impl CommandMetadata {
    pub fn new(id: u32, func: CommandFn) -> Self {
        Self { id: id, func: func }
    }
}

pub trait Server {
    fn new() -> Self where Self: Sized;
    fn get_command_table(&self) -> Vec<CommandMetadata>;

    fn call_self_command(&mut self, command: CommandFn, ctx: &mut CommandContext) -> Result<()> {
        let original_fn: CommandOrigFn<Self> = unsafe {
            core::mem::transmute(command)
        };
        (original_fn)(self, ctx)
    }
}

fn new_server_impl<S: Server + 'static>() -> Box<dyn Server> {
    Box::new(S::new())
}

pub type NewServerFn = fn() -> Box<dyn Server>;

pub enum WaitHandleType {
    Server,
    Session
}

pub struct WaitHandle {
    pub handle: svc::Handle,
    pub wait_type: WaitHandleType
}

impl WaitHandle {
    pub const fn new(handle: svc::Handle, wait_type: WaitHandleType) -> Self {
        Self { handle: handle, wait_type: wait_type }
    }
}

pub struct ServerObject {
    pub server: Option<Box<dyn Server>>,
    pub new_server_fn: Option<NewServerFn>,
    pub handle: WaitHandle,
    pub forward_handle: svc::Handle,
    pub is_mitm_service: bool,
    pub service_name: sm::ServiceName
}

impl ServerObject {
    pub fn empty() -> Self {
        Self { server: None, new_server_fn: None, handle: WaitHandle::new(0, WaitHandleType::Server), forward_handle: 0, is_mitm_service: false, service_name: sm::ServiceName::empty() } 
    }

    pub fn new_session<S: Server + 'static>(handle: svc::Handle) -> Self {
        Self { server: Some(Box::new(S::new())), new_server_fn: Some(new_server_impl::<S>), handle: WaitHandle::new(handle, WaitHandleType::Session), forward_handle: 0, is_mitm_service: false, service_name: sm::ServiceName::empty() } 
    }
    
    pub fn new_server<S: Server + 'static>(handle: svc::Handle, service_name: sm::ServiceName, is_mitm_service: bool) -> Self {
        Self { server: Some(Box::new(S::new())), new_server_fn: Some(new_server_impl::<S>), handle: WaitHandle::new(handle, WaitHandleType::Server), forward_handle: 0, is_mitm_service: is_mitm_service, service_name: service_name } 
    }

    pub fn make_new_session(&self, handle: svc::Handle, forward_handle: svc::Handle) -> Result<Self> {
        let new_fn = self.get_new_server_fn()?;
        Ok(Self { server: Some((new_fn)()), new_server_fn: Some(new_fn), handle: WaitHandle::new(handle, WaitHandleType::Session), forward_handle: forward_handle, is_mitm_service: false, service_name: sm::ServiceName::empty() })
    }

    pub fn get_server(&mut self) -> Result<&mut Box<dyn Server>> {
        match &mut self.server {
            Some(server) => Ok(server),
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

pub struct ServerContainer {
    pub servers: Vec<ServerObject>
}

impl ServerContainer {
    pub fn new(server: ServerObject) -> Self {
        Self { servers: vec![server] } 
    }

    pub fn get_handle_index(&self, handle: svc::Handle) -> Option<usize> {
        let mut i: usize = 0;
        for server in &self.servers {
            if server.handle.handle == handle {
                return Some(i);
            }
            i += 1;
        }
        None
    }
    
    pub fn process_signaled_handle(&mut self, index: usize) -> Result<()> {
        let mut push_new_session = false;
        let mut new_session = ServerObject::empty();
        let mut should_close_session = false;

        match self.servers.get_mut(index) {
            None => return Err(results::hipc::ResultSessionClosed::make()),
            Some(server) => {
                match server.handle.wait_type {
                    WaitHandleType::Session => {
                        svc::reply_and_receive(&server.handle.handle as *const svc::Handle, 1, 0, -1)?;
                        let fwd_handle = server.forward_handle;
                        let is_mitm = fwd_handle != 0;

                        let mut ipc_buf_backup: [u8; 0x100] = [0; 0x100];
                        if is_mitm {
                            let ipc_buf = get_ipc_buffer();
                            unsafe {
                                core::ptr::copy(ipc_buf, ipc_buf_backup.as_mut_ptr(), ipc_buf_backup.len());
                            }
                        }

                        let session = Session::from_handle(server.handle.handle);

                        let mut ctx = CommandContext::new(session);
                        let command_type = read_command_from_ipc_buffer(&mut ctx);
                        match command_type {
                            CommandType::Request => {
                                match read_request_command_from_ipc_buffer(&mut ctx) {
                                    Ok(rq_id) => {
                                        match server.get_server() {
                                            Ok(server_box) => {
                                                // Nothing done on success here, as if the command succeeds it will automatically respond by itself.
                                                let mut command_found = false;
                                                for command in (*server_box).get_command_table() {
                                                    if command.id == rq_id {
                                                        command_found = true;
                                                        if let Err(rc) = (*server_box).call_self_command(command.func, &mut ctx) {
                                                            if is_mitm && results::sm::mitm::ResultShouldForwardToSession::matches(rc) {
                                                                let ipc_buf = get_ipc_buffer();
                                                                unsafe {
                                                                    core::ptr::copy(ipc_buf_backup.as_ptr(), ipc_buf, ipc_buf_backup.len());
                                                                }
                                                                // Let the original service take care of the command for us.
                                                                if let Err(rc) = svc::send_sync_request(fwd_handle) {
                                                                    write_request_command_response_on_ipc_buffer(&mut ctx, rc);
                                                                }
                                                            }
                                                            else {
                                                                write_request_command_response_on_ipc_buffer(&mut ctx, rc);
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
                                                        if let Err(rc) = svc::send_sync_request(server.forward_handle) {
                                                            write_request_command_response_on_ipc_buffer(&mut ctx, rc);
                                                        }
                                                    }
                                                    else {
                                                        write_request_command_response_on_ipc_buffer(&mut ctx, results::cmif::ResultInvalidCommandRequestId::make());
                                                    }
                                                }
                                            },
                                            Err(rc) => write_request_command_response_on_ipc_buffer(&mut ctx, rc),
                                        }
                                    }
                                    Err(rc) => write_request_command_response_on_ipc_buffer(&mut ctx, rc),
                                };
                            },
                            CommandType::Control => {
                                write_control_command_response_on_ipc_buffer(&mut ctx, results::hipc::ResultSessionClosed::make());
                            }
                            CommandType::Close => {
                                should_close_session = true;
                                write_close_command_response_on_ipc_buffer(&mut ctx);
                            },
                            _ => {
                                todo!();
                            }
                        }
                        
                        match svc::reply_and_receive(&server.handle.handle as *const svc::Handle, 0, server.handle.handle, 0) {
                            Err(rc) => {
                                if !results::os::ResultTimeout::matches(rc) {
                                    return Err(rc);
                                }
                            },
                            _ => {}
                        };
                    },
                    WaitHandleType::Server => {
                        let new_handle = svc::accept_session(server.handle.handle)?;
                        let mut forward_handle: svc::Handle = 0;

                        if server.is_mitm_service {
                            let mut sm = service::new_named_port_object::<sm::UserInterface>()?;
                            let (_info, session_handle) = sm.atmosphere_acknowledge_mitm_session(server.service_name)?;
                            forward_handle = session_handle;
                        }

                        push_new_session = true;
                        new_session = server.make_new_session(new_handle, forward_handle)?;
                    },
                };
            }
        };

        if push_new_session {
            self.servers.push(new_session);
        }
        if should_close_session {
            self.servers.remove(index);
        }

        Ok(())
    }
}

pub trait Service: Server {
    fn get_name() -> &'static str;
    fn get_max_sesssions() -> i32;
}

pub trait MitmService: Server {
    fn get_name() -> &'static str;
    fn should_mitm(info: sm::MitmProcessInfo) -> bool;
}

pub trait NamedPort: Server {
    fn get_port_name() -> &'static str;
    fn get_max_sesssions() -> i32;
}

pub trait IMitmQueryServer {
    ipc_server_interface_define_command!(should_mitm: (info: sm::MitmProcessInfo) => (should_mitm: bool));
}

pub struct MitmQueryServer<S: MitmService> {
    phantom: core::marker::PhantomData<S>
}

impl<S: MitmService> IMitmQueryServer for MitmQueryServer<S> {
    fn should_mitm(&mut self, info: sm::MitmProcessInfo) -> Result<bool> {
        Ok(S::should_mitm(info))
    }
}

impl<S: MitmService> Server for MitmQueryServer<S> {
    fn new() -> Self {
        Self { phantom: core::marker::PhantomData }
    }

    fn get_command_table(&self) -> Vec<CommandMetadata> {
        ipc_server_make_command_metadata!(
            should_mitm: 65000
        )
    }
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
        for server_container in &self.server_containers {
            for server in &server_container.servers {
                if server.handle.handle != 0 {
                    if i < 0x40 {
                        self.wait_handles[i] = server.handle.handle;
                        i += 1;
                    }
                }
            }
        }
        (self.wait_handles.as_mut_ptr(), i as u32)
    }

    pub fn register_container(&mut self, container: ServerContainer) {
        self.server_containers.push(container);
    }
    
    pub fn register_server<S: Server + 'static>(&mut self, handle: svc::Handle, service_name: sm::ServiceName, is_mitm_service: bool) {
        let server = ServerObject::new_server::<S>(handle, service_name, is_mitm_service);
        let container = ServerContainer::new(server);
        self.server_containers.push(container);
    }
    
    pub fn register_session<S: Server + 'static>(&mut self, handle: svc::Handle) {
        let server = ServerObject::new_session::<S>(handle);
        let container = ServerContainer::new(server);
        self.server_containers.push(container);
    }

    pub fn register_mitm_query_session<S: MitmService + 'static>(&mut self, query_handle: svc::Handle) {
        let server = ServerObject::new_session::<MitmQueryServer<S>>(query_handle);
        let container = ServerContainer::new(server);
        self.mitm_query_sessions.push(container);
    }
    
    pub fn register_service_server<S: Service + 'static>(&mut self) -> Result<()> {
        let service_name = sm::ServiceName::new(S::get_name());
        
        let service_handle = {
            let mut sm = service::new_named_port_object::<sm::UserInterface>()?;
            sm.register_service(service_name, false, S::get_max_sesssions())?
        };

        self.register_server::<S>(service_handle, service_name, false);
        Ok(())
    }
    
    pub fn register_mitm_service_server<S: MitmService + 'static>(&mut self) -> Result<()> {
        let service_name = sm::ServiceName::new(S::get_name());

        let (mitm_handle, query_handle) = {
            let mut sm = service::new_named_port_object::<sm::UserInterface>()?;
            sm.atmosphere_install_mitm(service_name)?
        };

        self.register_server::<S>(mitm_handle, service_name, true);
        self.register_mitm_query_session::<S>(query_handle);
        Ok(())
    }

    pub fn register_named_port_server<S: NamedPort + 'static>(&mut self) -> Result<()> {
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