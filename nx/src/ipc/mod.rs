use crate::result::*;
use crate::results;
use crate::svc;
use crate::thread;
use core::ptr;
use core::mem;
use core::fmt;
use enumflags2::BitFlags;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ControlRequestId {
    ConvertCurrentObjectToDomain = 0,
    CopyFromCurrentDomain = 1,
    CloneCurrentObject = 2,
    QueryPointerBufferSize = 3,
    CloneCurrentObjectEx = 4
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Session {
    pub handle: svc::Handle,
    pub object_id: u32,
    pub owns_handle: bool
}

impl Session {
    pub const fn new() -> Self {
        Self { handle: 0, object_id: 0, owns_handle: false }
    }

    pub const fn from_handle(handle: svc::Handle) -> Self {
        Self { handle: handle, object_id: 0, owns_handle: true }
    }

    pub const fn from_object_id(parent_handle: svc::Handle, object_id: u32) -> Self {
        Self { handle: parent_handle, object_id: object_id, owns_handle: false }
    }

    pub const fn is_valid(&self) -> bool {
        self.handle != 0
    }

    pub const fn is_domain(&self) -> bool {
        self.object_id != 0
    }

    pub fn convert_current_object_to_domain(&mut self) -> Result<()> {
        ipc_client_session_send_control_command!([*self; ControlRequestId::ConvertCurrentObjectToDomain; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                object_id: u32 => self.object_id
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(())
    }

    pub fn query_pointer_buffer_size(&mut self) -> Result<u16> {
        let pointer_buf_size: u16;
        ipc_client_session_send_control_command!([*self; ControlRequestId::QueryPointerBufferSize; false] => {
            In {};
            InHandles {};
            InObjects {};
            InSessions {};
            Buffers {};
            Out {
                ptr_buf_size: u16 => pointer_buf_size
            };
            OutHandles {};
            OutObjects {};
            OutSessions {};
        });
        Ok(pointer_buf_size)
    }

    pub fn close(&mut self) {
        if self.is_valid() {
            if self.is_domain() {
                let mut ctx = CommandContext::new(*self);
                client::write_request_command_on_ipc_buffer(&mut ctx, None, DomainCommandType::Close);
                let _ = svc::send_sync_request(self.handle);
            }
            else if self.owns_handle {
                let mut ctx = CommandContext::new(*self);
                client::write_close_command_on_ipc_buffer(&mut ctx);
                let _ = svc::send_sync_request(self.handle);
            }
            if self.owns_handle {
                let _ = svc::close_handle(self.handle);
            }
            *self = Self::new();
        }
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ handle: {} (owned: {}), object ID: {} ]", self.handle, self.owns_handle, self.object_id)
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
    pub in_object_count: u8,
    pub data_size: u16,
    pub object_id: u32,
    pub pad: u32,
    pub token: u32,
}

impl DomainInDataHeader {
    pub const fn empty() -> Self {
        Self { command_type: DomainCommandType::Invalid, in_object_count: 0, data_size: 0, object_id: 0, pad: 0, token: 0 }
    }

    pub const fn new(command_type: DomainCommandType, in_object_count: u8, data_size: u16, object_id: u32, token: u32) -> Self {
        Self { command_type: command_type, in_object_count: in_object_count, data_size: data_size, object_id: object_id, pad: 0, token: token }
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

#[derive(BitFlags, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum BufferAttribute {
    In = 0b1,
    Out = 0b10,
    MapAlias = 0b100,
    Pointer = 0b1000,
    FixedSize = 0b10000,
    AutoSelect = 0b100000,
    MapTransferAllowsNonSecure = 0b1000000,
    MapTransferAllowsNonDevice = 0b10000000,
}

const MAX_COUNT: usize = 8;

pub struct CommandIn {
    pub send_process_id: bool,
    pub process_id: u64,
    pub data_size: u32,
    pub data_offset: *mut u8,
    pub data_words_offset: *mut u8,
    pub objects_offset: *mut u8,
    copy_handles: [svc::Handle; MAX_COUNT],
    copy_handle_count: usize,
    move_handles: [svc::Handle; MAX_COUNT],
    move_handle_count: usize,
    objects: [u32; MAX_COUNT],
    object_count: usize,
    out_pointer_sizes: [u16; MAX_COUNT],
    out_pointer_size_count: usize,
}

impl CommandIn {
    pub const fn empty() -> Self {
        Self { send_process_id: false, process_id: 0, data_size: 0, data_offset: ptr::null_mut(), data_words_offset: ptr::null_mut(), objects_offset: ptr::null_mut(), copy_handles: [0; MAX_COUNT], copy_handle_count: 0, move_handles: [0; MAX_COUNT], move_handle_count: 0, objects: [0; MAX_COUNT], object_count: 0, out_pointer_sizes: [0; MAX_COUNT], out_pointer_size_count: 0 }
    }
    
    pub fn add_copy_handle(&mut self, handle: svc::Handle) {
        if self.copy_handle_count < MAX_COUNT {
            self.copy_handles[self.copy_handle_count] = handle;
            self.copy_handle_count += 1;
        }
    }

    pub fn add_move_handle(&mut self, handle: svc::Handle) {
        if self.move_handle_count < MAX_COUNT {
            self.move_handles[self.move_handle_count] = handle;
            self.move_handle_count += 1;
        }
    }

    pub fn add_handle(&mut self, handle: svc::Handle, mode: HandleMode) {
        match mode {
            HandleMode::Copy => self.add_copy_handle(handle),
            HandleMode::Move => self.add_move_handle(handle),
        }
    }

    pub fn add_object(&mut self, object_id: u32) {
        if self.object_count < MAX_COUNT {
            self.objects[self.object_count] = object_id;
            self.object_count += 1;
        }
    }

    pub fn add_session(&mut self, session: Session) {
        if session.is_domain() {
            self.add_object(session.object_id);
        }
    }

    pub fn add_out_pointer_size(&mut self, pointer_size: u16) {
        if self.out_pointer_size_count < MAX_COUNT {
            self.out_pointer_sizes[self.out_pointer_size_count] = pointer_size;
            self.out_pointer_size_count += 1;
        }
    }

    pub fn get_data_at<T: Copy>(&self, offset: usize) -> Result<T> {
        result_return_if!((offset + mem::size_of::<T>()) as u32 > self.data_size, 0xBE3F);

        unsafe {
            let ptr = self.data_offset.offset(offset as isize) as *const T;
            Ok(ptr.read_volatile())
        }
    }
}

pub struct CommandOut {
    pub send_process_id: bool,
    pub process_id: u64,
    pub data_size: u32,
    pub data_offset: *mut u8,
    pub data_words_offset: *mut u8,
    copy_handles: [svc::Handle; MAX_COUNT],
    copy_handle_count: usize,
    move_handles: [svc::Handle; MAX_COUNT],
    move_handle_count: usize,
    objects: [u32; MAX_COUNT],
    object_count: usize,
}

impl CommandOut {
    pub const fn empty() -> Self {
        Self { send_process_id: false, process_id: 0, data_size: 0, data_offset: ptr::null_mut(), data_words_offset: ptr::null_mut(), copy_handles: [0; MAX_COUNT], copy_handle_count: 0, move_handles: [0; MAX_COUNT], move_handle_count: 0, objects: [0; MAX_COUNT], object_count: 0 }
    }
    
    pub fn pop_copy_handle(&mut self) -> Result<svc::Handle> {
        if self.copy_handle_count > 0 {
            self.copy_handle_count -= 1;
            return Ok(self.copy_handles[self.copy_handle_count]);
        }
        Err(results::cmif::ResultInvalidOutObjectCount::make())
    }

    pub fn pop_move_handle(&mut self) -> Result<svc::Handle> {
        if self.move_handle_count > 0 {
            self.move_handle_count -= 1;
            return Ok(self.move_handles[self.move_handle_count]);
        }
        Err(results::cmif::ResultInvalidOutObjectCount::make())
    }

    pub fn pop_handle(&mut self, mode: HandleMode) -> Result<svc::Handle> {
        match mode {
            HandleMode::Copy => self.pop_copy_handle(),
            HandleMode::Move => self.pop_move_handle(),
        }
    }

    pub fn pop_object(&mut self) -> Result<u32> {
        if self.object_count > 0 {
            self.object_count -= 1;
            return Ok(self.objects[self.object_count]);
        }
        Err(results::cmif::ResultInvalidOutObjectCount::make())
    }

    pub fn set_data_at<T: Copy>(&self, offset: usize, t: T) -> Result<()> {
        result_return_if!((offset + mem::size_of::<T>()) as u32 > self.data_size, 0xBE3F);

        unsafe {
            let ptr = self.data_offset.offset(offset as isize) as *mut T;
            *ptr = t;
            Ok(())
        }
    }
}

#[repr(C)]
pub struct CommandContext {
    pub session: Session,
    pub in_params: CommandIn,
    pub out_params: CommandOut,
    send_statics: [SendStaticDescriptor; MAX_COUNT],
    send_static_count: usize,
    receive_statics: [ReceiveStaticDescriptor; MAX_COUNT],
    receive_static_count: usize,
    send_buffers: [BufferDescriptor; MAX_COUNT],
    send_buffer_count: usize,
    receive_buffers: [BufferDescriptor; MAX_COUNT],
    receive_buffer_count: usize,
    exchange_buffers: [BufferDescriptor; MAX_COUNT],
    exchange_buffer_count: usize
}

impl CommandContext {
    pub const fn empty() -> Self {
        Self { session: Session::new(), in_params: CommandIn::empty(), out_params: CommandOut::empty(), send_statics: [SendStaticDescriptor::empty(); MAX_COUNT], send_static_count: 0, receive_statics: [ReceiveStaticDescriptor::empty(); MAX_COUNT], receive_static_count: 0, send_buffers: [BufferDescriptor::empty(); MAX_COUNT], send_buffer_count: 0, receive_buffers: [BufferDescriptor::empty(); MAX_COUNT], receive_buffer_count: 0, exchange_buffers: [BufferDescriptor::empty(); MAX_COUNT], exchange_buffer_count: 0 }
    }

    pub const fn new(session: Session) -> Self {
        let mut ctx = Self::empty();
        ctx.session = session;
        ctx
    }

    pub fn add_send_static(&mut self, send_static: SendStaticDescriptor) {
        if self.send_static_count < MAX_COUNT {
            self.send_statics[self.send_static_count] = send_static;
            self.send_static_count += 1;
        }
    }

    pub fn add_receive_static(&mut self, receive_static: ReceiveStaticDescriptor) {
        if self.receive_static_count < MAX_COUNT {
            self.receive_statics[self.receive_static_count] = receive_static;
            self.receive_static_count += 1;
        }
    }

    pub fn add_send_buffer(&mut self, send_buffer: BufferDescriptor) {
        if self.send_buffer_count < MAX_COUNT {
            self.send_buffers[self.send_buffer_count] = send_buffer;
            self.send_buffer_count += 1;
        }
    }

    pub fn add_receive_buffer(&mut self, receive_buffer: BufferDescriptor) {
        if self.receive_buffer_count < MAX_COUNT {
            self.receive_buffers[self.receive_buffer_count] = receive_buffer;
            self.receive_buffer_count += 1;
        }
    }

    pub fn add_exchange_buffer(&mut self, exchange_buffer: BufferDescriptor) {
        if self.exchange_buffer_count < MAX_COUNT {
            self.exchange_buffers[self.exchange_buffer_count] = exchange_buffer;
            self.exchange_buffer_count += 1;
        }
    }

    pub fn add_buffer(&mut self, buffer: *const u8, buffer_size: usize, buffer_attribute: BitFlags<BufferAttribute>) -> Result<()> {
        let is_in = buffer_attribute.contains(BufferAttribute::In);
        let is_out = buffer_attribute.contains(BufferAttribute::Out);

        if buffer_attribute.contains(BufferAttribute::AutoSelect) {
            let pointer_buf_size = self.session.query_pointer_buffer_size()?;
            let buffer_in_static = (pointer_buf_size > 0) && (buffer_size <= pointer_buf_size as usize);
            if is_in {
                if buffer_in_static {
                    self.add_send_buffer(BufferDescriptor::new(ptr::null(), 0, BufferFlags::Normal));
                    self.add_send_static(SendStaticDescriptor::new(buffer, buffer_size, self.send_static_count as u32));
                }
                else {
                    self.add_send_buffer(BufferDescriptor::new(buffer, buffer_size, BufferFlags::Normal));
                    self.add_send_static(SendStaticDescriptor::new(ptr::null(), 0, self.send_static_count as u32));
                }
            }
            else if is_out {
                if buffer_in_static {
                    self.add_receive_buffer(BufferDescriptor::new(ptr::null(), 0, BufferFlags::Normal));
                    self.add_receive_static(ReceiveStaticDescriptor::new(buffer, buffer_size));
                    self.in_params.add_out_pointer_size(buffer_size as u16);
                }
                else {
                    self.add_receive_buffer(BufferDescriptor::new(buffer, buffer_size, BufferFlags::Normal));
                    self.add_receive_static(ReceiveStaticDescriptor::new(ptr::null(), 0));
                    self.in_params.add_out_pointer_size(0);
                }
            }
        }
        else if buffer_attribute.contains(BufferAttribute::Pointer) {
            if is_in {
                self.add_send_static(SendStaticDescriptor::new(buffer, buffer_size, self.send_static_count as u32));
            }
            else if is_out {
                self.add_receive_static(ReceiveStaticDescriptor::new(buffer, buffer_size));
                if !buffer_attribute.contains(BufferAttribute::FixedSize) {
                    self.in_params.add_out_pointer_size(buffer_size as u16);
                }
            }
        }
        else if buffer_attribute.contains(BufferAttribute::MapAlias) {
            let mut flags = BufferFlags::Normal;
            if buffer_attribute.contains(BufferAttribute::MapTransferAllowsNonSecure) {
                flags = BufferFlags::NonSecure;
            }
            else if buffer_attribute.contains(BufferAttribute::MapTransferAllowsNonDevice){
                flags = BufferFlags::NonDevice;
            }
            let buf_desc = BufferDescriptor::new(buffer, buffer_size, flags);
            if is_in && is_out {
                self.add_exchange_buffer(buf_desc);
            }
            else if is_in {
                self.add_send_buffer(buf_desc);
            }
            else if is_out {
                self.add_receive_buffer(buf_desc);
            }
        }

        Ok(())
    }

    pub fn pop_session(&mut self) -> Result<Session> {
        let session: Session;
        if self.session.is_domain() {
            let object = self.out_params.pop_object()?;
            session = Session::from_object_id(self.session.handle, object);
        }
        else {
            let handle = self.out_params.pop_handle(HandleMode::Move)?;
            session = Session::from_handle(handle);
        }
        Ok(session)
    }
}

#[inline(always)]
pub fn get_ipc_buffer() -> *mut u8 {
    unsafe {
        &mut (*thread::get_thread_local_storage()).ipc_buffer as *mut _ as *mut u8
    }
}

#[inline(always)]
pub fn read_array_from_buffer<T: Copy>(buffer: *mut u8, count: u32, array: &mut [T; MAX_COUNT]) -> *mut u8 {
    unsafe {
        let tmp_buffer = buffer as *mut T;
        if (count > 0) && (count as usize <= MAX_COUNT) {
            for i in 0..count {
                let cur = tmp_buffer.offset(i as isize);
                array[i as usize] = *cur;
            }
        }
        tmp_buffer.offset(count as isize) as *mut u8
    }
}

#[inline(always)]
pub fn write_array_to_buffer<T: Copy>(buffer: *mut u8, count: u32, array: &[T; MAX_COUNT]) -> *mut u8 {
    unsafe {
        let temp_buffer = buffer as *mut T;
        if (count > 0) && (count as usize <= MAX_COUNT) {
            for i in 0..count {
                let cur = temp_buffer.offset(i as isize);
                *cur = array[i as usize];
            }
        }
        temp_buffer.offset(count as isize) as *mut u8
    }
}

#[inline(always)]
pub const fn get_aligned_data_offset(data_words_offset: *mut u8, base_offset: *mut u8) -> *mut u8 {
    unsafe {
        let data_offset = (data_words_offset as usize - base_offset as usize + 15) & !15;
        (data_offset + base_offset as usize) as *mut u8
    }
}

// IPC command argument types

pub struct Buffer<const A: BufferAttribute> {
    pub buf: *const u8,
    pub size: usize
}

impl<const A: BufferAttribute> Buffer<A> {
    pub const fn from_const<T>(buf: *const T, size: usize) -> Self {
        Self { buf: buf as *const u8, size: size }
    }

    pub const fn from_mut<T>(buf: *mut T, size: usize) -> Self {
        Self { buf: buf as *const u8, size: size }
    }

    pub const fn from_var<T>(var: &T) -> Self {
        Self { buf: var as *const T as *const u8, size: mem::size_of::<T>() }
    }

    pub const fn get_as<T>(&self) -> &'static T {
        unsafe {
            &*(self.buf as *const T)
        }
    }
}

pub struct Handle<const M: HandleMode> {
    handle: svc::Handle
}

impl<const M: HandleMode> Handle<M> {
    pub const fn from(handle: svc::Handle) -> Self {
        Self { handle: handle }
    }

    pub const fn get_value(&self) -> svc::Handle {
        self.handle
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum CommandParameter {
    Raw,
    Buffer,
    Handle,
    Session
}

pub trait CommandParameterDeserialize<T> {
    fn deserialize() -> CommandParameter;
}

pub struct CommandParameterDeserializer<T> {
    phantom: core::marker::PhantomData<T>
}

impl<T> CommandParameterDeserialize<T> for CommandParameterDeserializer<T> {
    default fn deserialize() -> CommandParameter {
        CommandParameter::Raw
    }
}

impl<const A: BufferAttribute> CommandParameterDeserialize<Buffer<A>> for CommandParameterDeserializer<Buffer<A>> {
    default fn deserialize() -> CommandParameter {
        CommandParameter::Buffer
    }
}

impl<const M: HandleMode> CommandParameterDeserialize<Handle<M>> for CommandParameterDeserializer<Handle<M>> {
    default fn deserialize() -> CommandParameter {
        CommandParameter::Handle
    }
}

impl CommandParameterDeserialize<Session> for CommandParameterDeserializer<Session> {
    default fn deserialize() -> CommandParameter {
        CommandParameter::Session
    }
}

pub struct DataWalker {
    ptr: *mut u8,
    cur_offset: isize
}

impl DataWalker {
    pub fn new(ptr: *mut u8) -> Self {
        Self { ptr: ptr, cur_offset: 0 }
    }

    pub fn advance<T: Copy>(&mut self) {
        let align_of_type = core::mem::align_of::<T>() as isize;
        self.cur_offset += align_of_type - 1;
        self.cur_offset -= self.cur_offset % align_of_type;
        self.cur_offset += core::mem::size_of::<T>() as isize;
    }

    pub fn advance_get<T: Copy>(&mut self) -> T {
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

    pub fn advance_set<T: Copy>(&mut self, t: T) {
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

    pub fn get_offset(&self) -> isize {
        self.cur_offset
    }
}

pub mod client;

pub mod server;