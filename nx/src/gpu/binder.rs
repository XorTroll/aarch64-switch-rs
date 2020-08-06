use crate::result::*;
use crate::svc;
use crate::gpu::parcel;
use crate::service::dispdrv;
use crate::service::dispdrv::IHOSBinderDriver;
use crate::mem;
use core::mem as cmem;
use super::*;

pub const INTERFACE_TOKEN: &str = "android.gui.IGraphicBufferProducer";

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(i32)]
pub enum ErrorCode {
    Success = 0,
    PermissionDenied = -1,
    NameNotFound = -2,
    WouldBlock = -11,
    NoMemory = -12,
    AlreadyExists = -17,
    NoInit = -19,
    BadValue = -22,
    DeadObject = -32,
    InvalidOperation = -38,
    NotEnoughData = -61,
    UnknownTransaction = -74,
    BadIndex = -75,
    TimeOut = -110,
    FdsNotAllowed = -2147483641,
    FailedTransaction = -2147483646,
    BadType = -2147483647,
}

pub const RESULT_SUBMODULE: u32 = 8;

result_lib_define_group!(RESULT_SUBMODULE => {
    ResultErrorCodeInvalid: 1,
    ResultErrorCodePermissionDenied: 2,
    ResultErrorCodeNameNotFound: 3,
    ResultErrorCodeWouldBlock: 4,
    ResultErrorCodeNoMemory: 5,
    ResultErrorCodeAlreadyExists: 6,
    ResultErrorCodeNoInit: 7,
    ResultErrorCodeBadValue: 8,
    ResultErrorCodeDeadObject: 9,
    ResultErrorCodeInvalidOperation: 10,
    ResultErrorCodeNotEnoughData: 11,
    ResultErrorCodeUnknownTransaction: 12,
    ResultErrorCodeBadIndex: 13,
    ResultErrorCodeTimeOut: 14,
    ResultErrorCodeFdsNotAllowed: 15,
    ResultErrorCodeFailedTransaction: 16,
    ResultErrorCodeBadType: 17
});

#[allow(unreachable_patterns)]
pub fn convert_error_code(err: ErrorCode) -> Result<()> {
    match err {
        ErrorCode::Success => Ok(()),
        ErrorCode::PermissionDenied => Err(ResultCode::from::<ResultErrorCodePermissionDenied>()),
        ErrorCode::NameNotFound => Err(ResultCode::from::<ResultErrorCodeNameNotFound>()),
        ErrorCode::WouldBlock => Err(ResultCode::from::<ResultErrorCodeWouldBlock>()),
        ErrorCode::NoMemory => Err(ResultCode::from::<ResultErrorCodeNoMemory>()),
        ErrorCode::AlreadyExists => Err(ResultCode::from::<ResultErrorCodeAlreadyExists>()),
        ErrorCode::NoInit => Err(ResultCode::from::<ResultErrorCodeNoInit>()),
        ErrorCode::BadValue => Err(ResultCode::from::<ResultErrorCodeBadValue>()),
        ErrorCode::DeadObject => Err(ResultCode::from::<ResultErrorCodeDeadObject>()),
        ErrorCode::InvalidOperation => Err(ResultCode::from::<ResultErrorCodeInvalidOperation>()),
        ErrorCode::NotEnoughData => Err(ResultCode::from::<ResultErrorCodeNotEnoughData>()),
        ErrorCode::UnknownTransaction => Err(ResultCode::from::<ResultErrorCodeUnknownTransaction>()),
        ErrorCode::BadIndex => Err(ResultCode::from::<ResultErrorCodeBadIndex>()),
        ErrorCode::TimeOut => Err(ResultCode::from::<ResultErrorCodeTimeOut>()),
        ErrorCode::FdsNotAllowed => Err(ResultCode::from::<ResultErrorCodeFdsNotAllowed>()),
        ErrorCode::FailedTransaction => Err(ResultCode::from::<ResultErrorCodeFailedTransaction>()),
        ErrorCode::BadType => Err(ResultCode::from::<ResultErrorCodeBadType>()),
        _ => Err(ResultCode::from::<ResultErrorCodeInvalid>()),
    }
}

pub struct Binder {
    handle: i32,
    hos_binder_driver: mem::SharedObject<dispdrv::HOSBinderDriver>
}

impl Binder {
    pub fn new(handle: i32, hos_binder_driver: mem::SharedObject<dispdrv::HOSBinderDriver>) -> Self {
        Self { handle: handle, hos_binder_driver: hos_binder_driver }
    }

    fn transact_parcel_begin(&self, parcel: &mut parcel::Parcel) -> Result<()> {
        parcel.write_interface_token(INTERFACE_TOKEN)
    }

    fn transact_parcel_check_err(&mut self, parcel: &mut parcel::Parcel) -> Result<()> {
        let err: ErrorCode = parcel.read()?;
        convert_error_code(err)?;
        Ok(())
    }

    fn transact_parcel_impl(&mut self, transaction_id: dispdrv::ParcelTransactionId, payload: parcel::ParcelPayload, payload_size: usize) -> Result<parcel::Parcel> {
        let response_payload = parcel::ParcelPayload::new();
        self.hos_binder_driver.borrow_mut().transact_parcel(self.handle, transaction_id, 0, &payload as *const _ as *const u8, payload_size, &response_payload as *const _ as *const u8, cmem::size_of::<parcel::ParcelPayload>())?;
        
        let mut parcel = parcel::Parcel::new();
        parcel.load_from(response_payload);
        Ok(parcel)
    }

    fn transact_parcel(&mut self, transaction_id: dispdrv::ParcelTransactionId, parcel: &mut parcel::Parcel) -> Result<parcel::Parcel> {
        let (payload, payload_size) = parcel.end_write()?;
        self.transact_parcel_impl(transaction_id, payload, payload_size)
    }

    pub fn get_handle(&self) -> i32 {
        self.handle
    }

    pub fn get_hos_binder_driver(&mut self) -> mem::SharedObject<dispdrv::HOSBinderDriver> {
        self.hos_binder_driver.clone()
    }

    pub fn increase_refcounts(&mut self) -> Result<()> {
        self.hos_binder_driver.borrow_mut().adjust_refcount(self.handle, 1, dispdrv::RefcountType::Weak)?;
        self.hos_binder_driver.borrow_mut().adjust_refcount(self.handle, 1, dispdrv::RefcountType::Strong)
    }

    pub fn decrease_refcounts(&mut self) -> Result<()> {
        self.hos_binder_driver.borrow_mut().adjust_refcount(self.handle, -1, dispdrv::RefcountType::Weak)?;
        self.hos_binder_driver.borrow_mut().adjust_refcount(self.handle, -1, dispdrv::RefcountType::Strong)
    }

    pub fn connect(&mut self, api: ConnectionApi, producer_controlled_by_app: bool) -> Result<QueueBufferOutput> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        let producer_listener: u32 = 0;
        parcel.write(producer_listener)?;
        parcel.write(api)?;
        parcel.write(producer_controlled_by_app as u32)?;

        let mut response_parcel = self.transact_parcel(dispdrv::ParcelTransactionId::Connect, &mut parcel)?;
        let qbo: QueueBufferOutput = response_parcel.read()?;

        self.transact_parcel_check_err(&mut response_parcel)?;
        Ok(qbo)
    }

    pub fn disconnect(&mut self, api: ConnectionApi, mode: DisconnectMode) -> Result<()> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        parcel.write(api)?;
        parcel.write(mode)?;

        let mut response_parcel = self.transact_parcel(dispdrv::ParcelTransactionId::Disconnect, &mut parcel)?;

        self.transact_parcel_check_err(&mut response_parcel)?;
        Ok(())
    }

    pub fn set_preallocated_buffer(&mut self, slot: i32, buf: GraphicBuffer) -> Result<()> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        parcel.write(slot)?;
        let has_input = true;
        parcel.write(has_input as u32)?;
        if has_input {
            parcel.write_sized(buf)?;
        }

        self.transact_parcel(dispdrv::ParcelTransactionId::SetPreallocatedBuffer, &mut parcel)?;
        Ok(())
    }
    
    pub fn request_buffer(&mut self, slot: i32) -> Result<(bool, GraphicBuffer)> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        parcel.write(slot)?;

        let mut response_parcel = self.transact_parcel(dispdrv::ParcelTransactionId::RequestBuffer, &mut parcel)?;
        let non_null_v: u32 = response_parcel.read()?;
        let non_null = non_null_v != 0;
        let mut gfx_buf: GraphicBuffer = unsafe { cmem::zeroed() };
        if non_null {
            gfx_buf = response_parcel.read_sized()?;
        }

        self.transact_parcel_check_err(&mut response_parcel)?;
        Ok((non_null, gfx_buf))
    }

    pub fn dequeue_buffer(&mut self, is_async: bool, width: u32, height: u32, get_frame_timestamps: bool, usage: BitFlags<GraphicsAllocatorUsage>) -> Result<(i32, bool, MultiFence)> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        parcel.write(is_async as u32)?;
        parcel.write(width)?;
        parcel.write(height)?;
        parcel.write(get_frame_timestamps as u32)?;
        parcel.write(usage)?;

        let mut response_parcel = self.transact_parcel(dispdrv::ParcelTransactionId::DequeueBuffer, &mut parcel)?;

        let slot: i32 = response_parcel.read()?;
        let has_fences_v: u32 = response_parcel.read()?;
        let has_fences = has_fences_v != 0;
        let mut fences: MultiFence = unsafe { cmem::zeroed() };
        if has_fences {
            fences = response_parcel.read_sized()?;
        }

        self.transact_parcel_check_err(&mut response_parcel)?;
        Ok((slot, has_fences, fences))
    }

    pub fn queue_buffer(&mut self, slot: i32, qbi: QueueBufferInput) -> Result<QueueBufferOutput> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        parcel.write(slot)?;
        parcel.write_sized(qbi)?;

        let mut response_parcel = self.transact_parcel(dispdrv::ParcelTransactionId::QueueBuffer, &mut parcel)?;

        let qbo = response_parcel.read()?;

        self.transact_parcel_check_err(&mut response_parcel)?;
        Ok(qbo)
    }

    pub fn get_native_handle(&mut self, unk: u32) -> Result<svc::Handle> {
        self.hos_binder_driver.borrow_mut().get_native_handle(self.handle, unk)
    }
}