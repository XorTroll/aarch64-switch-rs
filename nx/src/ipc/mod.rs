use crate::result::*;
use crate::results;
use crate::svc;
use crate::thread;
use core::ptr;
use core::mem;
use arrayvec::ArrayVec;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ControlRequestId {
    ConvertCurrentObjectToDomain = 0,
    CopyFromCurrentDomain = 1,
    CloneCurrentObject = 2,
    QueryPointerBufferSize = 3,
    CloneCurrentObjectEx = 4
}

pub type DomainObjectId = u32;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ObjectInfo {
    pub handle: svc::Handle,
    pub domain_object_id: DomainObjectId,
    pub owns_handle: bool
}

impl ObjectInfo {
    pub const fn new() -> Self {
        Self { handle: 0, domain_object_id: 0, owns_handle: false }
    }

    pub const fn from_handle(handle: svc::Handle) -> Self {
        Self { handle: handle, domain_object_id: 0, owns_handle: true }
    }

    pub const fn from_domain_object_id(parent_handle: svc::Handle, domain_object_id: DomainObjectId) -> Self {
        Self { handle: parent_handle, domain_object_id: domain_object_id, owns_handle: false }
    }

    pub const fn is_valid(&self) -> bool {
        self.handle != 0
    }

    pub const fn is_domain(&self) -> bool {
        self.domain_object_id != 0
    }

    pub fn convert_current_object_to_domain(&mut self) -> Result<DomainObjectId> {
        ipc_client_send_control_command!([*self; ControlRequestId::ConvertCurrentObjectToDomain] () => (domain_object_id: DomainObjectId))
    }

    pub fn query_pointer_buffer_size(&mut self) -> Result<u16> {
        ipc_client_send_control_command!([*self; ControlRequestId::QueryPointerBufferSize] () => (pointer_buffer_size: u16))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum HandleMode {
    Copy = 0,
    Move = 1
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum BufferFlags {
    Normal = 0,
    NonSecure = 1,
    Invalid = 2,
    NonDevice = 3
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BufferDescriptor {
    pub size_low: u32,
    pub address_low: u32,
    pub bits: u32,
}

impl BufferDescriptor {
    pub const fn empty() -> Self {
        Self { size_low: 0, address_low: 0, bits: 0 }
    }

    pub const fn new(buffer: *const u8, buffer_size: usize, flags: BufferFlags) -> Self {
        unsafe {
            let address_low = buffer as usize as u32;
            let address_mid = ((buffer as usize) >> 32) as u32;
            let address_high = ((buffer as usize) >> 36) as u32;
            let size_low = buffer_size as u32;
            let size_high = (buffer_size >> 32) as u32;

            let mut bits: u32 = 0;
            write_bits!(0, 1, bits, flags as u32);
            write_bits!(2, 23, bits, address_high);
            write_bits!(24, 27, bits, size_high);
            write_bits!(28, 31, bits, address_mid);

            Self { size_low: size_low, address_low: address_low, bits: bits }
        }
    }

    pub const fn get_address(&self) -> *mut u8 {
        let address_high = read_bits!(2, 23, self.bits);
        let address_mid = read_bits!(28, 31, self.bits);
        (self.address_low as usize | ((address_mid as usize) << 32) | ((address_high as usize) << 36)) as *mut u8
    }

    pub const fn get_size(&self) -> usize {
        let size_high = read_bits!(24, 27, self.bits);
        self.size_low as usize | ((size_high as usize) << 32)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SendStaticDescriptor {
    bits: u32,
    address_low: u32,
}

impl SendStaticDescriptor {
    pub const fn empty() -> Self {
        Self { bits: 0, address_low: 0 }
    }

    pub const fn new(buffer: *const u8, buffer_size: usize, index: u32) -> Self {
        unsafe {
            let address_low = buffer as usize as u32;
            let address_mid = ((buffer as usize) >> 32) as u32;
            let address_high = ((buffer as usize) >> 36) as u32;

            let mut bits: u32 = 0;
            write_bits!(0, 5, bits, index);
            write_bits!(6, 11, bits, address_high);
            write_bits!(12, 15, bits, address_mid);
            write_bits!(16, 31, bits, buffer_size as u32);

            Self { bits: bits, address_low: address_low }
        }
    }

    pub const fn get_address(&self) -> *mut u8 {
        let address_high = read_bits!(6, 11, self.bits);
        let address_mid = read_bits!(12, 15, self.bits);
        (self.address_low as usize | ((address_mid as usize) << 32) | ((address_high as usize) << 36)) as *mut u8
    }

    pub const fn get_size(&self) -> usize {
        read_bits!(16, 31, self.bits) as usize
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ReceiveStaticDescriptor {
    address_low: u32,
    bits: u32,
}

impl ReceiveStaticDescriptor {
    pub const fn empty() -> Self {
        Self { address_low: 0, bits: 0 }
    }

    pub const fn new(buffer: *const u8, buffer_size: usize) -> Self {
        unsafe {
            let address_low = buffer as usize as u32;
            let address_high = ((buffer as usize) >> 32) as u32;

            let mut bits: u32 = 0;
            write_bits!(0, 15, bits, address_high);
            write_bits!(16, 31, bits, buffer_size as u32);

            Self { address_low: address_low, bits: bits }
        }
    }

    pub const fn get_address(&self) -> *mut u8 {
        let address_high = read_bits!(0, 15, self.bits);
        (self.address_low as usize | ((address_high as usize) << 32)) as *mut u8
    }

    pub const fn get_size(&self) -> usize {
        read_bits!(16, 31, self.bits) as usize
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum CommandType {
    Invalid = 0,
    LegacyRequest = 1,
    Close = 2,
    LegacyControl = 3,
    Request = 4,
    Control = 5,
    RequestWithContext = 6,
    ControlWithContext = 7
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CommandHeader {
    bits_1: u32,
    bits_2: u32,
}

impl CommandHeader {
    pub const fn empty() -> Self {
        Self { bits_1: 0, bits_2: 0 }
    }

    pub const fn encode_receive_static_type(receive_static_count: u32) -> u32 {
        let mut static_type: u32 = 0;
        if receive_static_count > 0 {
            static_type += 2;
            if receive_static_count != 0xFF {
                static_type += receive_static_count;
            }
        }
        static_type
    }

    pub const fn decode_receive_static_type(receive_static_type: u32) -> u32 {
        let mut count: u32 = 0;
        if receive_static_type > 0 {
            if receive_static_type == 2 {
                count = 0xFF;
            }
            else if receive_static_type > 2 {
                count = receive_static_type - 2;
            }
        }
        count
    }

    pub const fn new(command_type: CommandType, send_static_count: u32, send_buffer_count: u32, receive_buffer_count: u32, exchange_buffer_count: u32, data_word_count: u32, receive_static_count: u32, has_special_header: bool) -> Self {
        let mut bits_1: u32 = 0;
        write_bits!(0, 15, bits_1, command_type as u32);
        write_bits!(16, 19, bits_1, send_static_count);
        write_bits!(20, 23, bits_1, send_buffer_count);
        write_bits!(24, 27, bits_1, receive_buffer_count);
        write_bits!(28, 31, bits_1, exchange_buffer_count);

        let mut bits_2: u32 = 0;
        write_bits!(0, 9, bits_2, data_word_count);
        write_bits!(10, 13, bits_2, Self::encode_receive_static_type(receive_static_count));
        write_bits!(31, 31, bits_2, has_special_header as u32);

        Self { bits_1: bits_1, bits_2: bits_2 }
    }

    pub const fn get_command_type(&self) -> CommandType {
        let raw_type = read_bits!(0, 15, self.bits_1);
        unsafe {
            mem::transmute(raw_type as u16)
        }
    }

    pub const fn get_send_static_count(&self) -> u32 {
        read_bits!(16, 19, self.bits_1)
    }

    pub const fn get_send_buffer_count(&self) -> u32 {
        read_bits!(20, 23, self.bits_1)
    }

    pub const fn get_receive_buffer_count(&self) -> u32 {
        read_bits!(24, 27, self.bits_1)
    }

    pub const fn get_exchange_buffer_count(&self) -> u32 {
        read_bits!(28, 31, self.bits_1)
    }

    pub const fn get_data_word_count(&self) -> u32 {
        read_bits!(0, 9, self.bits_2)
    }

    pub const fn get_receive_static_count(&self) -> u32 {
        Self::decode_receive_static_type(read_bits!(10, 13, self.bits_2))
    }

    pub const fn get_has_special_header(&self) -> bool {
        read_bits!(31, 31, self.bits_2) != 0
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CommandSpecialHeader {
    bits: u32,
}

impl CommandSpecialHeader {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn new(send_process_id: bool, copy_handle_count: u32, move_handle_count: u32) -> Self {
        let mut bits: u32 = 0;
        write_bits!(0, 0, bits, send_process_id as u32);
        write_bits!(1, 4, bits, copy_handle_count);
        write_bits!(5, 8, bits, move_handle_count);

        Self { bits: bits }
    }

    pub const fn get_send_process_id(&self) -> bool {
        read_bits!(0, 0, self.bits) != 0
    }

    pub const fn get_copy_handle_count(&self) -> u32 {
        read_bits!(1, 4, self.bits)
    }

    pub const fn get_move_handle_count(&self) -> u32 {
        read_bits!(5, 8, self.bits)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DataHeader {
    pub magic: u32,
    pub version: u32,
    pub value: u32,
    pub token: u32,
}

impl DataHeader {
    pub const fn empty() -> Self {
        Self { magic: 0, version: 0, value: 0, token: 0 }
    }

    pub const fn new(magic: u32, version: u32, value: u32, token: u32) -> Self {
        Self { magic: magic, version: version, value: value, token: token }
    }
}

pub const DATA_PADDING: u32 = 16;

pub const IN_DATA_HEADER_MAGIC: u32 = 0x49434653;
pub const OUT_DATA_HEADER_MAGIC: u32 = 0x4F434653;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum DomainCommandType {
    Invalid = 0,
    SendMessage = 1,
    Close = 2
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DomainInDataHeader {
    pub command_type: DomainCommandType,
    pub object_count: u8,
    pub data_size: u16,
    pub domain_object_id: DomainObjectId,
    pub pad: u32,
    pub token: u32,
}

impl DomainInDataHeader {
    pub const fn empty() -> Self {
        Self { command_type: DomainCommandType::Invalid, object_count: 0, data_size: 0, domain_object_id: 0, pad: 0, token: 0 }
    }

    pub const fn new(command_type: DomainCommandType, object_count: u8, data_size: u16, domain_object_id: DomainObjectId, token: u32) -> Self {
        Self { command_type: command_type, object_count: object_count, data_size: data_size, domain_object_id: domain_object_id, pad: 0, token: token }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DomainOutDataHeader {
    pub out_object_count: u32,
    pub pad: [u32; 3],
}

impl DomainOutDataHeader {
    pub const fn empty() -> Self {
        Self { out_object_count: 0, pad: [0; 3] }
    }

    pub const fn new(out_object_count: u32) -> Self {
        let mut header = Self::empty();
        header.out_object_count = out_object_count;
        header
    }
}

bit_enum! {
    BufferAttribute (u8) {
        In = bit!(0),
        Out = bit!(1),
        MapAlias = bit!(2),
        Pointer = bit!(3),
        FixedSize = bit!(4),
        AutoSelect = bit!(5),
        MapTransferAllowsNonSecure = bit!(6),
        MapTransferAllowsNonDevice = bit!(7)
    }
}

const MAX_COUNT: usize = 8;

#[derive(Copy, Clone)]
pub struct DataWalker {
    ptr: *mut u8,
    cur_offset: isize
}

impl DataWalker {
    pub fn empty() -> Self {
        Self { ptr: ptr::null_mut(), cur_offset: 0 }
    }

    pub fn new(ptr: *mut u8) -> Self {
        Self { ptr: ptr, cur_offset: 0 }
    }

    pub fn advance<T>(&mut self) {
        let align_of_type = core::mem::align_of::<T>() as isize;
        self.cur_offset += align_of_type - 1;
        self.cur_offset -= self.cur_offset % align_of_type;
        self.cur_offset += core::mem::size_of::<T>() as isize;
    }

    pub fn advance_get<T>(&mut self) -> T {
        unsafe {
            let align_of_type = core::mem::align_of::<T>() as isize;
            self.cur_offset += align_of_type - 1;
            self.cur_offset -= self.cur_offset % align_of_type;
            let offset = self.cur_offset;
            self.cur_offset += core::mem::size_of::<T>() as isize;

            let data_ref = self.ptr.offset(offset) as *const T;
            data_ref.read_volatile()
        }
    }

    pub fn advance_set<T>(&mut self, t: T) {
        unsafe {
            let align_of_type = core::mem::align_of::<T>() as isize;
            self.cur_offset += align_of_type - 1;
            self.cur_offset -= self.cur_offset % align_of_type;
            let offset = self.cur_offset;
            self.cur_offset += core::mem::size_of::<T>() as isize;

            let data_ref = self.ptr.offset(offset) as *mut T;
            data_ref.write_volatile(t);
        }
    }

    pub fn reset(&mut self) {
        self.cur_offset = 0;
    }

    pub fn reset_with(&mut self, ptr: *mut u8) {
        self.reset();
        self.ptr = ptr;
    }

    pub fn get_offset(&self) -> isize {
        self.cur_offset
    }
}

pub struct CommandIn {
    pub send_process_id: bool,
    pub process_id: u64,
    pub data_size: u32,
    pub data_offset: *mut u8,
    pub data_words_offset: *mut u8,
    pub objects_offset: *mut u8,
    copy_handles: ArrayVec<[svc::Handle; MAX_COUNT]>,
    move_handles: ArrayVec<[svc::Handle; MAX_COUNT]>,
    objects: ArrayVec<[DomainObjectId; MAX_COUNT]>,
    out_pointer_sizes: ArrayVec<[u16; MAX_COUNT]>,
}

impl CommandIn {
    pub fn empty() -> Self {
        Self { send_process_id: false, process_id: 0, data_size: 0, data_offset: ptr::null_mut(), data_words_offset: ptr::null_mut(), objects_offset: ptr::null_mut(), copy_handles: ArrayVec::new(), move_handles: ArrayVec::new(), objects: ArrayVec::new(), out_pointer_sizes: ArrayVec::new() }
    }
    
    pub fn add_copy_handle(&mut self, handle: svc::Handle) -> Result<()> {
        match self.copy_handles.try_push(handle) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_move_handle(&mut self, handle: svc::Handle) -> Result<()> {
        match self.move_handles.try_push(handle) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_handle<const M: HandleMode>(&mut self, handle: sf::Handle<M>) -> Result<()> {
        match M {
            HandleMode::Copy => self.add_copy_handle(handle.handle),
            HandleMode::Move => self.add_move_handle(handle.handle)
        }
    }

    pub fn add_domain_object(&mut self, domain_object_id: DomainObjectId) -> Result<()> {
        match self.objects.try_push(domain_object_id) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_object(&mut self, object_info: ObjectInfo) -> Result<()> {
        if object_info.is_domain() {
            self.add_domain_object(object_info.domain_object_id)
        }
        else {
            Err(ResultCode::new(0xCCC))
        }
    }

    pub fn add_out_pointer_size(&mut self, pointer_size: u16) -> Result<()> {
        match self.out_pointer_sizes.try_push(pointer_size) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }
}

pub struct CommandOut {
    pub send_process_id: bool,
    pub process_id: u64,
    pub data_size: u32,
    pub data_offset: *mut u8,
    pub data_words_offset: *mut u8,
    copy_handles: ArrayVec<[svc::Handle; MAX_COUNT]>,
    move_handles: ArrayVec<[svc::Handle; MAX_COUNT]>,
    objects: ArrayVec<[DomainObjectId; MAX_COUNT]>
}

impl CommandOut {
    pub fn empty() -> Self {
        Self { send_process_id: false, process_id: 0, data_size: 0, data_offset: ptr::null_mut(), data_words_offset: ptr::null_mut(), copy_handles: ArrayVec::new(), move_handles: ArrayVec::new(), objects: ArrayVec::new() }
    }
    
    pub fn pop_copy_handle(&mut self) -> Result<svc::Handle> {
        match self.copy_handles.pop_at(0) {
            Some(handle) => Ok(handle),
            None => Err(results::cmif::ResultInvalidOutObjectCount::make())
        }
    }

    pub fn pop_move_handle(&mut self) -> Result<svc::Handle> {
        match self.move_handles.pop_at(0) {
            Some(handle) => Ok(handle),
            None => Err(results::cmif::ResultInvalidOutObjectCount::make())
        }
    }

    pub fn pop_handle<const M: HandleMode>(&mut self) -> Result<sf::Handle<M>> {
        let handle = match M {
            HandleMode::Copy => sf::Handle::from(self.pop_copy_handle()?),
            HandleMode::Move => sf::Handle::from(self.pop_move_handle()?),
        };
        Ok(handle)
    }

    pub fn push_copy_handle(&mut self, handle: svc::Handle) -> Result<()> {
        match self.copy_handles.try_push(handle) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn push_move_handle(&mut self, handle: svc::Handle) -> Result<()> {
        match self.move_handles.try_push(handle) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn push_handle<const M: HandleMode>(&mut self, handle: sf::Handle<M>) -> Result<()> {
        match M {
            HandleMode::Copy => self.push_copy_handle(handle.handle),
            HandleMode::Move => self.push_move_handle(handle.handle)
        }
    }

    pub fn pop_domain_object(&mut self) -> Result<DomainObjectId> {
        match self.objects.pop_at(0) {
            Some(handle) => Ok(handle),
            None => Err(results::cmif::ResultInvalidOutObjectCount::make())
        }
    }

    pub fn push_domain_object(&mut self, domain_object_id: DomainObjectId) -> Result<()> {
        match self.objects.try_push(domain_object_id) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }
}

pub struct CommandContext {
    pub object_info: ObjectInfo,
    pub in_params: CommandIn,
    pub out_params: CommandOut,
    send_statics: ArrayVec<[SendStaticDescriptor; MAX_COUNT]>,
    receive_statics: ArrayVec<[ReceiveStaticDescriptor; MAX_COUNT]>,
    send_buffers: ArrayVec<[BufferDescriptor; MAX_COUNT]>,
    receive_buffers: ArrayVec<[BufferDescriptor; MAX_COUNT]>,
    exchange_buffers: ArrayVec<[BufferDescriptor; MAX_COUNT]>
}

impl CommandContext {
    pub fn empty() -> Self {
        Self { object_info: ObjectInfo::new(), in_params: CommandIn::empty(), out_params: CommandOut::empty(), send_statics: ArrayVec::new(), receive_statics: ArrayVec::new(), send_buffers: ArrayVec::new(), receive_buffers: ArrayVec::new(), exchange_buffers: ArrayVec::new() }
    }

    pub fn new(object_info: ObjectInfo) -> Self {
        let mut ctx = Self::empty();
        ctx.object_info = object_info;
        ctx
    }

    pub fn add_send_static(&mut self, send_static: SendStaticDescriptor) -> Result<()> {
        match self.send_statics.try_push(send_static) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_receive_static(&mut self, receive_static: ReceiveStaticDescriptor) -> Result<()> {
        match self.receive_statics.try_push(receive_static) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_send_buffer(&mut self, send_buffer: BufferDescriptor) -> Result<()> {
        match self.send_buffers.try_push(send_buffer) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_receive_buffer(&mut self, receive_buffer: BufferDescriptor) -> Result<()> {
        match self.receive_buffers.try_push(receive_buffer) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_exchange_buffer(&mut self, exchange_buffer: BufferDescriptor) -> Result<()> {
        match self.exchange_buffers.try_push(exchange_buffer) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_buffer<const A: BufferAttribute>(&mut self, buffer: sf::Buffer<A>) -> Result<()> {
        let is_in = A.contains(BufferAttribute::In());
        let is_out = A.contains(BufferAttribute::Out());

        if A.contains(BufferAttribute::AutoSelect()) {
            let pointer_buf_size = self.object_info.query_pointer_buffer_size()?;
            let buffer_in_static = (pointer_buf_size > 0) && (buffer.size <= pointer_buf_size as usize);
            if is_in {
                if buffer_in_static {
                    self.add_send_buffer(BufferDescriptor::new(ptr::null(), 0, BufferFlags::Normal))?;
                    self.add_send_static(SendStaticDescriptor::new(buffer.buf, buffer.size, self.send_statics.len() as u32))?;
                }
                else {
                    self.add_send_buffer(BufferDescriptor::new(buffer.buf, buffer.size, BufferFlags::Normal))?;
                    self.add_send_static(SendStaticDescriptor::new(ptr::null(), 0, self.send_statics.len() as u32))?;
                }
            }
            else if is_out {
                if buffer_in_static {
                    self.add_receive_buffer(BufferDescriptor::new(ptr::null(), 0, BufferFlags::Normal))?;
                    self.add_receive_static(ReceiveStaticDescriptor::new(buffer.buf, buffer.size))?;
                    self.in_params.add_out_pointer_size(buffer.size as u16)?;
                }
                else {
                    self.add_receive_buffer(BufferDescriptor::new(buffer.buf, buffer.size, BufferFlags::Normal))?;
                    self.add_receive_static(ReceiveStaticDescriptor::new(ptr::null(), 0))?;
                    self.in_params.add_out_pointer_size(0)?;
                }
            }
        }
        else if A.contains(BufferAttribute::Pointer()) {
            if is_in {
                self.add_send_static(SendStaticDescriptor::new(buffer.buf, buffer.size, self.send_statics.len() as u32))?;
            }
            else if is_out {
                self.add_receive_static(ReceiveStaticDescriptor::new(buffer.buf, buffer.size))?;
                if !A.contains(BufferAttribute::FixedSize()) {
                    self.in_params.add_out_pointer_size(buffer.size as u16)?;
                }
            }
        }
        else if A.contains(BufferAttribute::MapAlias()) {
            let mut flags = BufferFlags::Normal;
            if A.contains(BufferAttribute::MapTransferAllowsNonSecure()) {
                flags = BufferFlags::NonSecure;
            }
            else if A.contains(BufferAttribute::MapTransferAllowsNonDevice()) {
                flags = BufferFlags::NonDevice;
            }
            let buf_desc = BufferDescriptor::new(buffer.buf, buffer.size, flags);
            if is_in && is_out {
                self.add_exchange_buffer(buf_desc)?;
            }
            else if is_in {
                self.add_send_buffer(buf_desc)?;
            }
            else if is_out {
                self.add_receive_buffer(buf_desc)?;
            }
        }

        Ok(())
    }

    pub fn pop_send_static(&mut self) -> Result<SendStaticDescriptor> {
        match self.send_statics.pop_at(0) {
            Some(send_static) => Ok(send_static),
            None => Err(ResultCode::new(0xBB))
        }
    }

    pub fn pop_receive_static(&mut self) -> Result<ReceiveStaticDescriptor> {
        match self.receive_statics.pop_at(0) {
            Some(receive_static) => Ok(receive_static),
            None => Err(ResultCode::new(0xBB))
        }
    }

    pub fn pop_send_buffer(&mut self) -> Result<BufferDescriptor> {
        match self.send_buffers.pop_at(0) {
            Some(send_buffer) => Ok(send_buffer),
            None => Err(ResultCode::new(0xBB))
        }
    }

    pub fn pop_receive_buffer(&mut self) -> Result<BufferDescriptor> {
        match self.receive_buffers.pop_at(0) {
            Some(receive_buffer) => Ok(receive_buffer),
            None => Err(ResultCode::new(0xBB))
        }
    }

    pub fn pop_exchange_buffer(&mut self) -> Result<BufferDescriptor> {
        match self.exchange_buffers.pop_at(0) {
            Some(exchange_buffer) => Ok(exchange_buffer),
            None => Err(ResultCode::new(0xBB))
        }
    }

    pub fn pop_buffer<const A: BufferAttribute>(&mut self) -> Result<sf::Buffer<A>> {
        let is_in = A.contains(BufferAttribute::In());
        let is_out = A.contains(BufferAttribute::Out());

        if A.contains(BufferAttribute::AutoSelect()) {
            if is_in {
                if let Ok(static_desc) = self.pop_send_static() {
                    if let Ok(send_desc) = self.pop_send_buffer() {
                        if !static_desc.get_address().is_null() && (static_desc.get_size() > 0) {
                            return Ok(sf::Buffer::from_mut(static_desc.get_address(), static_desc.get_size()));
                        }
                        if !send_desc.get_address().is_null() && (send_desc.get_size() > 0) {
                            return Ok(sf::Buffer::from_mut(send_desc.get_address(), send_desc.get_size()));
                        }
                    }
                }
            }
            else if is_out {
                if let Ok(static_desc) = self.pop_receive_static() {
                    if let Ok(recv_desc) = self.pop_receive_buffer() {
                        if !static_desc.get_address().is_null() && (static_desc.get_size() > 0) {
                            return Ok(sf::Buffer::from_mut(static_desc.get_address(), static_desc.get_size()));
                        }
                        if !recv_desc.get_address().is_null() && (recv_desc.get_size() > 0) {
                            return Ok(sf::Buffer::from_mut(recv_desc.get_address(), recv_desc.get_size()));
                        }
                    }
                }
            }
        }
        else if A.contains(BufferAttribute::Pointer()) {
            if is_in {
                if let Ok(static_desc) = self.pop_send_static() {
                    return Ok(sf::Buffer::from_mut(static_desc.get_address(), static_desc.get_size()));
                }
            }
            else if is_out {
                if let Ok(static_desc) = self.pop_receive_static() {
                    return Ok(sf::Buffer::from_mut(static_desc.get_address(), static_desc.get_size()));
                }
            }
        }
        else if A.contains(BufferAttribute::MapAlias()) {
            if is_in && is_out {
                if let Ok(exch_desc) = self.pop_exchange_buffer() {
                    return Ok(sf::Buffer::from_mut(exch_desc.get_address(), exch_desc.get_size()));
                }
            }
            else if is_in {
                if let Ok(send_desc) = self.pop_send_buffer() {
                    return Ok(sf::Buffer::from_mut(send_desc.get_address(), send_desc.get_size()));
                }
            }
            else if is_out {
                if let Ok(recv_desc) = self.pop_receive_buffer() {
                    return Ok(sf::Buffer::from_mut(recv_desc.get_address(), recv_desc.get_size()));
                }
            }
        }

        Err(ResultCode::new(0xBABE))
    }

    pub fn pop_object(&mut self) -> Result<ObjectInfo> {
        let object_info: ObjectInfo;
        if self.object_info.is_domain() {
            let domain_object_id = self.out_params.pop_domain_object()?;
            object_info = ObjectInfo::from_domain_object_id(self.object_info.handle, domain_object_id);
        }
        else {
            let handle: sf::MoveHandle = self.out_params.pop_handle()?;
            object_info = ObjectInfo::from_handle(handle.handle);
        }
        Ok(object_info)
    }
}

#[inline(always)]
pub fn get_ipc_buffer() -> *mut u8 {
    unsafe {
        &mut (*thread::get_thread_local_storage()).ipc_buffer as *mut _ as *mut u8
    }
}

#[inline(always)]
pub fn read_array_from_buffer<T: Copy>(buffer: *mut u8, count: u32, array: &mut ArrayVec<[T; MAX_COUNT]>) -> *mut u8 {
    unsafe {
        let tmp_buffer = buffer as *mut T;
        array.clear();
        let _ = array.try_extend_from_slice(core::slice::from_raw_parts(tmp_buffer, count as usize));
        tmp_buffer.offset(count as isize) as *mut u8
    }
}

#[inline(always)]
pub fn write_array_to_buffer<T: Copy>(buffer: *mut u8, count: u32, array: &ArrayVec<[T; MAX_COUNT]>) -> *mut u8 {
    unsafe {
        let tmp_buffer = buffer as *mut T;
        core::ptr::copy(array.as_ptr(), tmp_buffer, count as usize);
        tmp_buffer.offset(count as isize) as *mut u8
    }
}

#[inline(always)]
pub const fn get_aligned_data_offset(data_words_offset: *mut u8, base_offset: *mut u8) -> *mut u8 {
    unsafe {
        let align = DATA_PADDING as usize - 1;
        let data_offset = (data_words_offset as usize - base_offset as usize + align) & !align;
        (data_offset + base_offset as usize) as *mut u8
    }
}

pub mod client;

pub mod server;

pub mod sf;