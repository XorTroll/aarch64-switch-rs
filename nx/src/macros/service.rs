#![macro_use]

#[macro_export]
macro_rules! session_object_define {
    ($name:ident) => {
        pub struct $name {
            session: $crate::ipc::Session,
        }
        
        impl $crate::service::SessionObject for $name {
            fn new(session: $crate::ipc::Session) -> Self {
                Self { session: session }
            }

            fn get_session(&self) -> $crate::ipc::Session {
                self.session
            }

            fn convert_current_object_to_domain(&mut self) -> Result<()> {
                self.session.convert_current_object_to_domain()
            }
            fn query_pointer_buffer_size(&mut self) -> Result<u16> {
                self.session.query_pointer_buffer_size()
            }
        
            fn close(&mut self) {
                self.session.close()
            }
        }

        impl $crate::service::SharedSessionObject for $name {
            fn shared(session: $crate::ipc::Session) -> $crate::mem::SharedObject<Self> {
                $crate::mem::make_shared(Self::new(session))
            }
        }
        
        impl core::ops::Drop for $name {
            fn drop(&mut self) {
                self.session.close();
            }
        }
    };
}