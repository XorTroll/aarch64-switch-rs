#![macro_use]

// TODO: buffer, handle, session

#[macro_export]
macro_rules! ipc_server_interface_define_command {
    ($name:ident: ( $( $in_param_name:ident: $in_param_type:ty ),* ) => ( $( $out_param_name:ident: $out_param_type:ty ),* )) => {
        #[allow(unused_parens)]
        fn $name(&mut self, $( $in_param_name: $in_param_type ),* ) -> $crate::result::Result<( $( $out_param_type ),* )>;

        paste::paste! {
            #[allow(unused_assignments)]
            #[allow(unused_parens)]
            fn [<$name _impl>](&mut self, ctx: &mut $crate::ipc::CommandContext) -> $crate::result::Result<()> {
                let mut walker = $crate::ipc::DataWalker::new(ctx.in_params.data_offset);
                $(
                    let $in_param_name: $in_param_type = match $crate::ipc::CommandParameterDeserializer::<$in_param_type>::deserialize() {
                        $crate::ipc::CommandParameter::Raw => {
                            walker.advance_get()
                        }
                        _ => todo!(),
                    };
                )*

                let ( $( $out_param_name ),* ) = self.$name( $( $in_param_name ),* )?;

                walker = $crate::ipc::DataWalker::new(core::ptr::null_mut());
                $(
                    match $crate::ipc::CommandParameterDeserializer::<$out_param_type>::deserialize() {
                        $crate::ipc::CommandParameter::Raw => {
                            walker.advance::<$out_param_type>();
                        }
                        _ => todo!(),
                    };
                )*

                ctx.out_params.data_size = walker.get_offset() as u32;
                $crate::ipc::server::write_request_command_response_on_ipc_buffer(ctx, $crate::result::ResultSuccess::make());
                walker = $crate::ipc::DataWalker::new(ctx.out_params.data_offset);
                $(
                    match $crate::ipc::CommandParameterDeserializer::<$out_param_type>::deserialize() {
                        $crate::ipc::CommandParameter::Raw => {
                            walker.advance_set($out_param_name);
                        }
                        _ => todo!(),
                    };
                )*

                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! ipc_server_make_command_metadata {
    ($( $name:ident: $id:expr ),*) => {
        paste::paste! {
            vec![ $( $crate::ipc::server::CommandMetadata::new($id, unsafe { core::mem::transmute(Self::[<$name _impl>] as fn(&mut Self, &mut $crate::ipc::CommandContext) -> $crate::result::Result<()>) }) ),* ]
        }
    };
}