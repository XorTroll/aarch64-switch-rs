#![macro_use]

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

            let mut ctx = $crate::ipc::CommandContext::new($session);
            ctx.in_params.send_process_id = $send_pid;
            ctx.in_params.data_size = in_size as u32;
            $( ctx.add_buffer($buf, $buf_size, $buf_attr)?; ),*
            $( ctx.in_params.add_handle($in_handle, $in_handle_mode); ),*
            $( ctx.in_params.add_object($in_object); ),*
            $( ctx.in_params.add_object($in_session.object_id); ),*

            $crate::ipc::client::write_request_command_on_ipc_buffer(&mut ctx, Some($rq_id), $crate::ipc::DomainCommandType::SendMessage);
            
            if in_size > 0 {
                unsafe {
                    core::ptr::copy(in_ref, ctx.in_params.data_offset, in_size);
                }
            }

            $crate::svc::send_sync_request($session.handle)?;

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

            $crate::ipc::client::read_request_command_response_from_ipc_buffer(&mut ctx)?;

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

            let mut ctx = $crate::ipc::CommandContext::new($session);
            ctx.in_params.send_process_id = $send_pid;
            ctx.in_params.data_size = in_size as u32;
            $( ctx.add_buffer($buf, $buf_size, $buf_attr)?; ),*
            $( ctx.in_params.add_handle($in_handle, $in_handle_mode); ),*
            $( ctx.in_params.add_object($in_object); ),*
            $( ctx.in_params.add_object($in_session.object_id); ),*

            $crate::ipc::client::write_control_command_on_ipc_buffer(&mut ctx, $control_rq_id);
            
            if in_size > 0 {
                unsafe {
                    core::ptr::copy(in_ref, ctx.in_params.data_offset, in_size);
                }
            }

            $crate::svc::send_sync_request($session.handle)?;

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

            $crate::ipc::client::read_control_command_response_from_ipc_buffer(&mut ctx)?;

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