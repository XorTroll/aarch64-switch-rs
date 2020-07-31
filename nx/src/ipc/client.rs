use crate::ipc;
use crate::result::*;
use core::mem;

#[inline(always)]
pub fn write_command_on_ipc_buffer(ctx: &mut ipc::CommandContext, command_type: ipc::CommandType, data_size: u32) {
    unsafe {
        let mut ipc_buf = ipc::get_ipc_buffer();
    
        let has_special_header = ctx.in_params.send_process_id || ctx.in_params.copy_handle_count > 0 || ctx.in_params.move_handle_count > 0;
        let data_word_count = (data_size + 3) / 4;
        let command_header = ipc_buf as *mut ipc::CommandHeader;
        *command_header = ipc::CommandHeader::new(command_type, ctx.send_static_count as u32, ctx.send_buffer_count as u32, ctx.receive_buffer_count as u32, ctx.exchange_buffer_count as u32, data_word_count, ctx.receive_static_count as u32, has_special_header);
        ipc_buf = command_header.offset(1) as *mut u8;

        if has_special_header {
            let special_header = ipc_buf as *mut ipc::CommandSpecialHeader;
            *special_header = ipc::CommandSpecialHeader::new(ctx.in_params.send_process_id, ctx.in_params.copy_handle_count as u32, ctx.in_params.move_handle_count as u32);
            ipc_buf = special_header.offset(1) as *mut u8;

            if ctx.in_params.send_process_id {
                ipc_buf = ipc_buf.offset(mem::size_of::<u64>() as isize);
            }

            ipc_buf = ipc::write_array_to_buffer(ipc_buf, ctx.in_params.copy_handle_count as u32, &ctx.in_params.copy_handles);
            ipc_buf = ipc::write_array_to_buffer(ipc_buf, ctx.in_params.move_handle_count as u32, &ctx.in_params.move_handles);
        }

        ipc_buf = ipc::write_array_to_buffer(ipc_buf, ctx.send_static_count as u32, &ctx.send_statics);
        ipc_buf = ipc::write_array_to_buffer(ipc_buf, ctx.send_buffer_count as u32, &ctx.send_buffers);
        ipc_buf = ipc::write_array_to_buffer(ipc_buf, ctx.receive_buffer_count as u32, &ctx.receive_buffers);
        ipc_buf = ipc::write_array_to_buffer(ipc_buf, ctx.exchange_buffer_count as u32, &ctx.exchange_buffers);
        ctx.in_params.data_words_offset = ipc_buf;
        ipc_buf = ipc_buf.offset((mem::size_of::<u32>() * data_word_count as usize) as isize);
        /* ipc_buf = */ ipc::write_array_to_buffer(ipc_buf, ctx.receive_static_count as u32, &ctx.receive_statics);
    }
}

#[inline(always)]
pub fn read_command_response_from_ipc_buffer(ctx: &mut ipc::CommandContext) {
    unsafe {
        let mut ipc_buf = ipc::get_ipc_buffer();

        let command_header = ipc_buf as *mut ipc::CommandHeader;
        ipc_buf = command_header.offset(1) as *mut u8;

        let mut copy_handle_count: u32 = 0;
        let mut move_handle_count: u32 = 0;
        if (*command_header).get_has_special_header() {
            let special_header = ipc_buf as *mut ipc::CommandSpecialHeader;
            copy_handle_count = (*special_header).get_copy_handle_count();
            move_handle_count = (*special_header).get_move_handle_count();
            ipc_buf = special_header.offset(1) as *mut u8;
            if (*special_header).get_send_process_id() {
                ctx.out_params.process_id = *(ipc_buf as *mut u64);
                ipc_buf = ipc_buf.offset(mem::size_of::<u64>() as isize);
            }
        }

        ipc_buf = ipc::read_array_from_buffer(ipc_buf, copy_handle_count, &mut ctx.out_params.copy_handles);
        ctx.out_params.copy_handle_count = copy_handle_count as usize;
        ipc_buf = ipc::read_array_from_buffer(ipc_buf, move_handle_count, &mut ctx.out_params.move_handles);
        ctx.out_params.move_handle_count = move_handle_count as usize;

        ipc_buf = ipc_buf.offset((mem::size_of::<ipc::SendStaticDescriptor>() * (*command_header).get_send_static_count() as usize) as isize);
        ctx.out_params.data_words_offset = ipc_buf;
    }
}

const PADDING: u32 = 16;

#[inline(always)]
pub fn write_request_command_on_ipc_buffer(ctx: &mut ipc::CommandContext, request_id: Option<u32>, domain_command_type: ipc::DomainCommandType) {
    unsafe {
        let ipc_buf = ipc::get_ipc_buffer();

        let has_data_header = request_id.is_some();
        let mut data_size = PADDING + ctx.in_params.data_size;
        if has_data_header {
            data_size += mem::size_of::<ipc::DataHeader>() as u32;
        }

        if ctx.session.is_domain() {
            data_size += (mem::size_of::<ipc::DomainInDataHeader>() + mem::size_of::<u32>() * ctx.in_params.object_count) as u32;
        }

        data_size = (data_size + 1) & !1;
        let out_pointer_sizes_offset = data_size;
        data_size += (mem::size_of::<u16>() * ctx.in_params.out_pointer_size_count) as u32;

        write_command_on_ipc_buffer(ctx, ipc::CommandType::Request, data_size);
        let mut data_offset = ipc::get_aligned_data_offset(ctx.in_params.data_words_offset, ipc_buf);

        let out_pointer_sizes = ctx.in_params.data_words_offset.offset(out_pointer_sizes_offset as isize);
        ipc::write_array_to_buffer(out_pointer_sizes, ctx.in_params.out_pointer_size_count as u32, &ctx.in_params.out_pointer_sizes);

        let mut data_header = data_offset as *mut ipc::DataHeader;
        if ctx.session.is_domain() {
            let domain_header = data_offset as *mut ipc::DomainInDataHeader;
            let left_data_size = mem::size_of::<ipc::DataHeader>() as u32 + ctx.in_params.data_size;
            *domain_header = ipc::DomainInDataHeader::new(domain_command_type, ctx.in_params.object_count as u8, left_data_size as u16, ctx.session.object_id, 0);
            data_offset = data_offset.offset(mem::size_of::<ipc::DomainInDataHeader>() as isize);
            ctx.in_params.objects_offset = data_offset.offset(left_data_size as isize);
            data_header = data_offset as *mut ipc::DataHeader;
        }

        if has_data_header {
            *data_header = ipc::DataHeader::new(ipc::IN_DATA_HEADER_MAGIC, 0, request_id.unwrap(), 0);
            data_offset = data_offset.offset(mem::size_of::<ipc::DataHeader>() as isize);
        }

        ctx.in_params.data_offset = data_offset;
    }
}

#[inline(always)]
pub fn read_request_command_response_from_ipc_buffer(ctx: &mut ipc::CommandContext) -> Result<()> {
    unsafe {
        let ipc_buf = ipc::get_ipc_buffer();
        read_command_response_from_ipc_buffer(ctx);

        let mut data_offset = ipc::get_aligned_data_offset(ctx.out_params.data_words_offset, ipc_buf);
        let mut data_header = data_offset as *mut ipc::DataHeader;
        if ctx.session.is_domain() {
            let domain_header = data_offset as *mut ipc::DomainOutDataHeader;
            data_offset = data_offset.offset(mem::size_of::<ipc::DomainOutDataHeader>() as isize);
            let objects_offset = data_offset.offset((mem::size_of::<ipc::DataHeader>() + ctx.out_params.data_size as usize) as isize);
            let object_count = (*domain_header).out_object_count;
            let _ = ipc::read_array_from_buffer(objects_offset, object_count, &mut ctx.out_params.objects);
            ctx.out_params.object_count = object_count as usize;
            data_header = data_offset as *mut ipc::DataHeader;
        }

        data_offset = data_offset.offset(mem::size_of::<ipc::DataHeader>() as isize);
        result_return_unless!((*data_header).magic == ipc::OUT_DATA_HEADER_MAGIC, 0xBEEF);
        result_try!(ResultCode::new((*data_header).value));

        ctx.out_params.data_offset = data_offset;
        Ok(())
    }
}

enum_define!(ControlRequestId(u32) {
    ConvertCurrentObjectToDomain = 0,
    CopyFromCurrentDomain = 1,
    CloneCurrentObject = 2,
    QueryPointerBufferSize = 3,
    CloneCurrentObjectEx = 4
});

#[inline(always)]
pub fn write_control_command_on_ipc_buffer(ctx: &mut ipc::CommandContext, request_id: ControlRequestId) {
    unsafe {
        let ipc_buf = ipc::get_ipc_buffer();
        let data_size = PADDING + mem::size_of::<ipc::DataHeader>() as u32 + ctx.in_params.data_size;

        write_command_on_ipc_buffer(ctx, ipc::CommandType::Control, data_size);
        let mut data_offset = ipc::get_aligned_data_offset(ctx.in_params.data_words_offset, ipc_buf);

        let data_header = data_offset as *mut ipc::DataHeader;
        *data_header = ipc::DataHeader::new(ipc::IN_DATA_HEADER_MAGIC, 0, request_id as u32, 0);

        data_offset = data_offset.offset(mem::size_of::<ipc::DataHeader>() as isize);
        ctx.in_params.data_offset = data_offset;
    }
}

#[inline(always)]
pub fn read_control_command_response_from_ipc_buffer(ctx: &mut ipc::CommandContext) -> Result<()> {
    unsafe {
        let ipc_buf = ipc::get_ipc_buffer();

        read_command_response_from_ipc_buffer(ctx);
        let mut data_offset = ipc::get_aligned_data_offset(ctx.out_params.data_words_offset, ipc_buf);

        let data_header = data_offset as *mut ipc::DataHeader;
        
        data_offset = data_offset.offset(mem::size_of::<ipc::DataHeader>() as isize);
        result_return_unless!((*data_header).magic == ipc::OUT_DATA_HEADER_MAGIC, 0xBEEF);
        result_try!(ResultCode::new((*data_header).value));

        ctx.out_params.data_offset = data_offset;
        Ok(())
    }
}

#[inline(always)]
pub fn write_close_command_on_ipc_buffer(ctx: &mut ipc::CommandContext) {
    write_command_on_ipc_buffer(ctx, ipc::CommandType::Close, 0);
}

#[macro_export]
macro_rules! ipc_client_session_send_request_command {
    ([$session:expr; $rq_id:expr; $send_pid:expr] => { In { $( $in_name:ident: $in_ty:ty = $in_val:expr ),* }; InHandles { $( $in_handle:expr => $in_handle_mode:expr ),* }; InObjects { $( $in_object:expr ),* }; InSessions { $( $in_session:expr ),* }; Buffers { $( ($buf:expr, $buf_size:expr) => $buf_attr:expr ),* }; Out { $( $out_name:ident: $out_ty:ty => $out_val:ident ),* }; OutHandles { $( $out_handle:expr => $out_handle_mode:expr ),* }; OutObjects { $( $out_object:expr ),* }; OutSessions { $( $out_session:expr ),* }; }) => {
        {
            #[repr(C)]
            struct _In {
                $($in_name: $in_ty),*
            }
            let in_data = _In {
                $($in_name: $in_val),*
            };
            let in_ref = &in_data as *const _ as *const u8;
            let in_size = core::mem::size_of::<_In>();

            let mut ctx = ipc::CommandContext::new($session);
            ctx.in_params.send_process_id = $send_pid;
            ctx.in_params.data_size = in_size as u32;
            $( ctx.add_buffer($buf, $buf_size, $buf_attr)?; ),*
            $( ctx.in_params.add_handle($in_handle, $in_handle_mode); ),*
            $( ctx.in_params.add_object($in_object); ),*
            $( ctx.in_params.add_object($in_session.object_id); ),*

            write_request_command_on_ipc_buffer(&mut ctx, Some($rq_id), ipc::DomainCommandType::SendMessage);
            
            if in_size > 0 {
                unsafe {
                    core::ptr::copy(in_ref, ctx.in_params.data_offset, in_size);
                }
            }

            svc::send_sync_request($session.handle)?;

            #[repr(C)]
            struct _Out {
                $($out_name: $out_ty),*
            }
            let mut out_data: _Out = unsafe {
                core::mem::zeroed()
            };
            let out_ref = &mut out_data as *mut _ as *mut u8;
            let out_size = core::mem::size_of::<_Out>();

            ctx.out_params.data_size = out_size as u32;

            read_request_command_response_from_ipc_buffer(&mut ctx)?;

            if out_size > 0 {
                unsafe {
                    core::ptr::copy(ctx.out_params.data_offset, out_ref, out_size);
                }
            }

            $( $out_val = out_data.$out_name; ),*
            $( $out_handle = ctx.out_params.pop_handle($out_handle_mode)?; ),*
            $( $out_object = ctx.out_params.pop_object()?; ),*
            $( $out_session = ctx.pop_session()?; ),*
        }
    };
}

#[macro_export]
macro_rules! ipc_client_session_send_control_command {
    ([$session:expr; $control_rq_id:expr; $send_pid:expr] => { In { $( $in_name:ident: $in_ty:ty = $in_val:expr ),* }; InHandles { $( $in_handle:expr => $in_handle_mode:expr ),* }; InObjects { $( $in_object:expr ),* }; InSessions { $( $in_session:expr ),* }; Buffers { $( ($buf:expr, $buf_size:expr) => $buf_attr:expr ),* }; Out { $( $out_name:ident: $out_ty:ty => $out_val:expr ),* }; OutHandles { $( $out_handle:expr => $out_handle_mode:expr ),* }; OutObjects { $( $out_object:expr ),* }; OutSessions { $( $out_session:expr ),* }; }) => {
        {
            #[repr(C)]
            struct _In {
                $($in_name: $in_ty),*
            }
            let in_data = _In {
                $($in_name: $in_val),*
            };
            let in_ref = &in_data as *const _ as *const u8;
            let in_size = core::mem::size_of::<_In>();

            let mut ctx = CommandContext::new($session);
            ctx.in_params.send_process_id = $send_pid;
            ctx.in_params.data_size = in_size as u32;
            $( ctx.add_buffer($buf, $buf_size, $buf_attr)?; ),*
            $( ctx.in_params.add_handle($in_handle, $in_handle_mode); ),*
            $( ctx.in_params.add_object($in_object); ),*
            $( ctx.in_params.add_object($in_session.object_id); ),*

            write_control_command_on_ipc_buffer(&mut ctx, $control_rq_id);
            
            if in_size > 0 {
                unsafe {
                    core::ptr::copy(in_ref, ctx.in_params.data_offset, in_size);
                }
            }

            svc::send_sync_request($session.handle)?;

            #[repr(C)]
            struct _Out {
                $($out_name: $out_ty),*
            }
            let mut out_data: _Out = unsafe {
                core::mem::zeroed()
            };
            let out_ref = &mut out_data as *mut _ as *mut u8;
            let out_size = core::mem::size_of::<_Out>();
            ctx.out_params.data_size = out_size as u32;

            read_control_command_response_from_ipc_buffer(&mut ctx)?;

            if out_size > 0 {
                unsafe {
                    core::ptr::copy(ctx.out_params.data_offset, out_ref, out_size);
                }
            }

            $( $out_val = out_data.$out_name; ),*
            $( $out_handle = ctx.out_params.pop_handle($out_handle_mode)?; ),*
            $( $out_object = ctx.out_params.pop_object()?; ),*
            $( $out_session = ctx.pop_session()?; ),*
        }
    };
}