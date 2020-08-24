use nx::result::*;
use nx::ipc::sf;
use nx::arm;
use nx::mem;
use nx::sync;
use nx::service;
use nx::service::fspsrv;
use nx::service::fspsrv::IFile;
use nx::service::fspsrv::IFileSystem;
use nx::service::fspsrv::IFileSystemProxy;
use alloc::string::String;

const BASE_LOG_DIR: &'static str = "/lm-rs";

static mut G_LOCK: sync::Mutex = sync::Mutex::new(false);
static mut G_FSP_SERVICE: mem::Shared<fspsrv::FileSystemProxy> = mem::Shared::empty();
static mut G_SD_FS: mem::Shared<fspsrv::FileSystem> = mem::Shared::empty();
static mut G_INITIALIZED: bool = false;
static mut G_ENABLED: bool = true;

pub fn initialize() -> Result<()> {
    unsafe {
        let _ = sync::ScopedLock::new(&mut G_LOCK);
        if !G_INITIALIZED {
            G_FSP_SERVICE = service::new_service_object::<fspsrv::FileSystemProxy>()?;
            G_SD_FS = G_FSP_SERVICE.get().open_sd_card_filesystem()?.to::<fspsrv::FileSystem>();
            
            let base_log_path = fspsrv::Path::from(BASE_LOG_DIR)?;
            let base_log_path_buf = sf::Buffer::from_var(&base_log_path);
            let _ = G_SD_FS.get().delete_directory_recursively(base_log_path_buf);

            G_INITIALIZED = true;
        }
    }
    Ok(())
}

pub fn exit() {
    unsafe {
        let _ = sync::ScopedLock::new(&mut G_LOCK);
        if G_INITIALIZED {
            G_SD_FS.reset();
            G_FSP_SERVICE.reset();
            G_INITIALIZED = false;
        }
    }
}

pub fn set_log_enabled(enabled: bool) {
    unsafe {
        let _ = sync::ScopedLock::new(&mut G_LOCK);
        G_ENABLED = enabled;
    }
}

fn log_packet_buf_impl(packet_buf: *const u8, buf_size: usize, log_dir: String, log_buf_file: String) -> Result<()> {
    unsafe {
        let _ = sync::ScopedLock::new(&mut G_LOCK);
        if G_INITIALIZED && G_ENABLED {
            let base_log_path = fspsrv::Path::from(BASE_LOG_DIR)?;
            let base_log_path_buf = sf::Buffer::from_var(&base_log_path);
            let _ = G_SD_FS.get().create_directory(base_log_path_buf);

            let log_dir_path = fspsrv::Path::from(&log_dir[..])?;
            let log_dir_path_buf = sf::Buffer::from_var(&log_dir_path);
            let _ = G_SD_FS.get().create_directory(log_dir_path_buf);

            let log_buf_path = fspsrv::Path::from(&log_buf_file[..])?;
            let log_buf_path_buf = sf::Buffer::from_var(&log_buf_path);
            let _ = G_SD_FS.get().delete_file(log_dir_path_buf);
            let _ = G_SD_FS.get().create_file(fspsrv::FileAttribute::None(), 0, log_buf_path_buf);

            {
                let log_file = G_SD_FS.get().open_file(fspsrv::FileOpenMode::Write() | fspsrv::FileOpenMode::Append(), log_buf_path_buf)?.to::<fspsrv::File>();
                log_file.get().write(fspsrv::FileWriteOption::Flush(), 0, buf_size, sf::Buffer::from_const(packet_buf, buf_size))?;
            }
        }
    }
    Ok(())
}

pub fn log_packet_buf(packet_buf: *const u8, buf_size: usize, program_id: u64) {
    let log_timestamp = arm::get_system_tick();
    let process_log_dir = format!("{}/0x{:016X}", BASE_LOG_DIR, program_id);
    let log_buf_path = format!("{}/0x{:016X}.bin", process_log_dir, log_timestamp);

    let _ = log_packet_buf_impl(packet_buf, buf_size, process_log_dir, log_buf_path);
}