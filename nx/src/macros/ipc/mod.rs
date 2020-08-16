#![macro_use]

pub mod client;

pub mod server;

#[macro_export]
macro_rules! ipc_interface_define_command {
    ($name:ident < $( $generic:ident ),* > : ( $( $in_param_name:ident: $in_param_type:ty ),* ) => ( $( $out_param_name:ident: $out_param_type:ty ),* )) => {
        #[allow(unused_parens)]
        fn $name< $( $generic: $crate::service::ISessionObject ),* >(&mut self, $( $in_param_name: $in_param_type ),* ) -> $crate::result::Result<( $( $out_param_type ),* )>;

        paste::paste! {
            #[allow(unused_assignments)]
            #[allow(unused_parens)]
            fn [<$name _impl>]< $( $generic: $crate::service::ISessionObject ),* >(&mut self, mut ctx: &mut $crate::ipc::CommandContext) -> $crate::result::Result<()> {
                let mut walker = $crate::ipc::DataWalker::new(ctx.in_params.data_offset);
                $( let $in_param_name = <$in_param_type as $crate::ipc::server::CommandParameter<_>>::after_request_read(&mut walker, &mut ctx)?; )*

                let ( $( $out_param_name ),* ) = self.$name::< $( $generic ),* >( $( $in_param_name ),* )?;

                walker = $crate::ipc::DataWalker::new(core::ptr::null_mut());
                $( $crate::ipc::server::CommandParameter::<_>::before_response_write(&$out_param_name, &mut walker, &mut ctx)?; )*
                ctx.out_params.data_size = walker.get_offset() as u32;

                $crate::ipc::server::write_request_command_response_on_ipc_buffer(ctx, $crate::result::ResultSuccess::make());

                walker = $crate::ipc::DataWalker::new(ctx.out_params.data_offset);
                $( $crate::ipc::server::CommandParameter::<_>::after_response_write(&$out_param_name, &mut walker, &mut ctx)?; )*

                Ok(())
            }
        }
    };

    ($name:ident: ( $( $in_param_name:ident: $in_param_type:ty ),* ) => ( $( $out_param_name:ident: $out_param_type:ty ),* )) => {
        ipc_interface_define_command!($name<>: ( $( $in_param_name: $in_param_type ),* ) => ( $( $out_param_name: $out_param_type ),* ));
    };
}