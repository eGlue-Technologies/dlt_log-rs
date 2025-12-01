use crate::libdlt;
use log::{Level, Metadata, Record};
use std::ffi::CString;

pub struct DltLoggerBuilder {
    context_name: String,
    context_description: String,
}
impl DltLoggerBuilder {
    pub fn new() -> Self {
        return Self {
            context_name: String::new(),
            context_description: String::new(),
        };
    }

    pub fn set_context_name(&mut self, context_name: String) {
        self.context_name = context_name;
    }

    pub fn set_context_description(&mut self, context_description: String) {
        self.context_description = context_description;
    }

    pub fn build(self) -> DltLogger {
        let c_context_id = CString::new(self.context_name).unwrap();
        let c_context_description = CString::new(self.context_description).unwrap();
        let mut ctx = Box::new(libdlt::DltContext::default());
        let _dlt_return_value = unsafe {
            libdlt::dlt_register_context(
                &mut *ctx,
                c_context_id.as_ptr(),
                c_context_description.as_ptr(),
            )
        };
        return DltLogger {
            context: Box::into_raw(ctx),
        };
    }
}

pub struct DltLogger {
    context: *mut libdlt::DltContext,
}
impl log::Log for DltLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let level = match record.level() {
            Level::Error => libdlt::DltLogLevelType::DLT_LOG_ERROR,
            Level::Warn => libdlt::DltLogLevelType::DLT_LOG_WARN,
            Level::Info => libdlt::DltLogLevelType::DLT_LOG_INFO,
            Level::Debug => libdlt::DltLogLevelType::DLT_LOG_DEBUG,
            Level::Trace => libdlt::DltLogLevelType::DLT_LOG_VERBOSE,
        };

        let text = format!(
            "[{}:{}] {}",
            record.file_static().unwrap().rsplit('/').next().unwrap(),
            record.line().unwrap(),
            record.args()
        );

        let c_text = match CString::new(text) {
            Ok(result) => result,
            Err(_error) => {
                CString::from(c"ERROR: NulError when converting log message from Rust to C.")
            }
        };

        let _dlt_return_value =
            unsafe { libdlt::dlt_log_string(self.context, level, c_text.as_ptr()) };
    }

    fn flush(&self) {}
}

// impl Drop for DltLogger {
//     fn drop(&mut self) {
//         let _dlt_return_value = unsafe { libdlt::dlt_unregister_context(self.context) };
//         let _dlt_return_value = unsafe { libdlt::dlt_unregister_app() };
//     }
// }

// The `DltLogger` struct is marked as `Send` and `Sync` because the underlying DLT library is
// thread-safe, see https://github.com/COVESA/dlt-daemon/blob/master/doc/dlt_design_specification.md.
unsafe impl Send for DltLogger {}
unsafe impl Sync for DltLogger {}

#[cfg(test)]
mod tests {
    use std::ptr::null_mut;

    use log::Log;

    use super::*;

    #[test]
    fn test_flush() {
        let logger = DltLogger {
            context: null_mut(),
        };
        // does nothing
        logger.flush();
    }

    #[test]
    fn test_enabled() {
        let logger = DltLogger {
            context: null_mut(),
        };

        // always needs be enabled as this is handled by DLT
        let metadata = Metadata::builder().level(Level::max()).build();
        assert!(logger.enabled(&metadata));
    }
}
