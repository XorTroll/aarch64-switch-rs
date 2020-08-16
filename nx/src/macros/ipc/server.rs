#![macro_use]

#[macro_export]
macro_rules! ipc_server_make_command_table {
    ($( $name:ident: $id:expr ),*) => {
        paste::paste! {
            vec![ $( $crate::ipc::server::CommandMetadata::new($id, unsafe { core::mem::transmute(Self::[<$name _impl>] as fn(&mut Self, &mut $crate::ipc::CommandContext) -> $crate::result::Result<()>) }) ),* ]
        }
    };
}