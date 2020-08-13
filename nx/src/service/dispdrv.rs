use crate::result::*;
use crate::ipc;
use crate::svc;
use crate::service;
use crate::service::SessionObject;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum RefcountType {
    Weak,
    Strong,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ParcelTransactionId {
    RequestBuffer = 1,
    SetBufferCount = 2,
    DequeueBuffer = 3,
    DetachBuffer = 4,
    DetachNextBuffer = 5,
    AttachBuffer = 6,
    QueueBuffer = 7,
    CancelBuffer = 8,
    Query = 9,
    Connect = 10,
    Disconnect = 11,
    SetSidebandStream = 12,
    AllocateBuffers = 13,
    SetPreallocatedBuffer = 14,
}

pub trait IHOSBinderDriver {
    fn transact_parcel(&mut self, binder_handle: i32, transaction_id: ParcelTransactionId, flags: u32, in_parcel_buf: *const u8, in_parcel_size: usize, out_parcel_buf: *const u8, out_parcel_size: usize) -> Result<()>;
    fn adjust_refcount(&mut self, binder_handle: i32, add_value: i32, refcount_type: RefcountType) -> Result<()>;
    fn get_native_handle(&mut self, binder_handle: i32, unk_type: u32) -> Result<svc::Handle>;
    fn transact_parcel_auto(&mut self, binder_handle: i32, transaction_id: ParcelTransactionId, flags: u32, in_parcel_buf: *const u8, in_parcel_size: usize, out_parcel_buf: *const u8, out_parcel_size: usize) -> Result<()>;
}

session_object_define!(HOSBinderDriver);

impl service::Service for HOSBinderDriver {
    fn get_name() -> &'static str {
        nul!("dispdrv")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl IHOSBinderDriver for HOSBinderDriver {
    fn transact_parcel(&mut self, binder_handle: i32, transaction_id: ParcelTransactionId, flags: u32, in_parcel_buf: *const u8, in_parcel_size: usize, out_parcel_buf: *const u8, out_parcel_size: usize) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 0; false] => {
            In {
                binder_handle: i32 = binder_handle,
                transaction_id: ParcelTransactionId = transaction_id,
                flags: u32 = flags
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (in_parcel_buf, in_parcel_size) => ipc::BufferAttribute::In | ipc::BufferAttribute::MapAlias,
                (out_parcel_buf, out_parcel_size) => ipc::BufferAttribute::Out | ipc::BufferAttribute::MapAlias
            };
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }

    fn adjust_refcount(&mut self, binder_handle: i32, add_value: i32, refcount_type: RefcountType) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 1; false] => {
            In {
                binder_handle: i32 = binder_handle,
                add_value: i32 = add_value,
                refcount_type: RefcountType = refcount_type
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }

    fn get_native_handle(&mut self, binder_handle: i32, unk_type: u32) -> Result<svc::Handle> {
        let handle: svc::Handle;
        ipc_client_session_send_request_command!([self.session; 2; false] => {
            In {
                binder_handle: i32 = binder_handle,
                unk_type: u32 = unk_type
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {};
            OutHandles {
                handle => ipc::HandleMode::Copy
            };
            OutObjects {};
            OutSessions {};
        });
        Ok(handle)
    }

    fn transact_parcel_auto(&mut self, binder_handle: i32, transaction_id: ParcelTransactionId, flags: u32, in_parcel_buf: *const u8, in_parcel_size: usize, out_parcel_buf: *const u8, out_parcel_size: usize) -> Result<()> {
        ipc_client_session_send_request_command!([self.session; 3; false] => {
            In {
                binder_handle: i32 = binder_handle,
                transaction_id: ParcelTransactionId = transaction_id,
                flags: u32 = flags
            };
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {
                (in_parcel_buf, in_parcel_size) => ipc::BufferAttribute::In | ipc::BufferAttribute::AutoSelect,
                (out_parcel_buf, out_parcel_size) => ipc::BufferAttribute::Out | ipc::BufferAttribute::AutoSelect
            };
            Out {};
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }
}