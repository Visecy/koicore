use koicore::WriterConfig;
use koicore::writer::{FloatFormat, FormatterOptions, NumberFormat};
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char};
use std::ptr;

/// Transparent configuration struct for FFI
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KoiFormatterOptions {
    pub indent: usize,
    pub use_tabs: bool,
    pub newline_before: bool,
    pub newline_after: bool,
    pub compact: bool,
    pub force_quotes_for_vars: bool,
    pub number_format: *const c_char,
    pub float_format: *const c_char,
    pub newline_before_param: bool,
    pub newline_after_param: bool,
    pub should_override: bool,
}

impl From<KoiFormatterOptions> for FormatterOptions {
    fn from(opt: KoiFormatterOptions) -> Self {
        let float_format = if opt.float_format.is_null() {
            FloatFormat::Default
        } else {
            let cstr = unsafe { CStr::from_ptr(opt.float_format) };
            FloatFormat::Custom(cstr.to_string_lossy().to_string())
        };
        Self {
            indent: opt.indent,
            use_tabs: opt.use_tabs,
            newline_before: opt.newline_before,
            newline_after: opt.newline_after,
            compact: opt.compact,
            force_quotes_for_vars: opt.force_quotes_for_vars,
            number_format: if opt.number_format.is_null() {
                NumberFormat::Unknown
            } else {
                let number_format = unsafe { CStr::from_ptr(opt.number_format) };
                NumberFormat::from(number_format.to_string_lossy().to_string())
            },
            float_format,
            newline_before_param: opt.newline_before_param,
            newline_after_param: opt.newline_after_param,
            should_override: opt.should_override,
        }
    }
}

impl From<FormatterOptions> for KoiFormatterOptions {
    fn from(opt: FormatterOptions) -> Self {
        Self {
            indent: opt.indent,
            use_tabs: opt.use_tabs,
            newline_before: opt.newline_before,
            newline_after: opt.newline_after,
            compact: opt.compact,
            force_quotes_for_vars: opt.force_quotes_for_vars,
            number_format: std::ptr::null(),
            float_format: std::ptr::null(),
            newline_before_param: opt.newline_before_param,
            newline_after_param: opt.newline_after_param,
            should_override: opt.should_override,
        }
    }
}

/// Initialize KoiFormatterOptions with default values
/// 
/// # Safety
/// 
/// The pointer must be valid and writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiFormatterOptions_Init(options: *mut KoiFormatterOptions) {
    let options = unsafe { options.as_mut() };
    if let Some(options) = options {
        let defaults = FormatterOptions::default();
        *options = defaults.into();
    }
}

/// Command-specific option entry
/// Used in a null-terminated array
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KoiCommandOption {
    /// Command name string
    /// Terminate list with name = NULL
    pub name: *const c_char,
    pub options: KoiFormatterOptions,
}

/// Parameter selector for options
/// If is_position is true, uses position. Otherwise uses name.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KoiParamFormatSelector {
    pub is_position: bool,
    pub position: usize,
    /// Only used if is_position is false
    pub name: *const c_char,
}

/// Parameter-specific option entry
/// Used in a null-terminated array
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KoiParamOption {
    /// Selector for which parameter this applies to
    /// Terminate list with name = NULL AND is_position = false
    pub selector: KoiParamFormatSelector,
    pub options: KoiFormatterOptions,
}

/// Transparent WriterConfig
/// Users can allocate this on stack.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct KoiWriterConfig {
    pub global_options: KoiFormatterOptions,
    pub command_threshold: usize,
    /// Pointer to array of KoiCommandOption, terminated by name=NULL.
    /// Can be NULL if no command options.
    pub command_options: *const KoiCommandOption,
}

/// Helper to convert raw pointer array to HashMap
unsafe fn parse_command_options(ptr: *const KoiCommandOption) -> HashMap<String, FormatterOptions> {
    unsafe {
        let mut map = HashMap::new();
        if ptr.is_null() {
            return map;
        }

        let mut current = ptr;
        while !(*current).name.is_null() {
            let name_str = CStr::from_ptr((*current).name)
                .to_string_lossy()
                .into_owned();
            map.insert(name_str, (*current).options.into());
            current = current.add(1);
        }
        map
    }
}

impl From<&KoiWriterConfig> for WriterConfig {
    fn from(config: &KoiWriterConfig) -> Self {
        Self {
            global_options: config.global_options.into(),
            command_threshold: config.command_threshold,
            command_options: unsafe { parse_command_options(config.command_options) },
        }
    }
}

/// Initialize KoiWriterConfig with default values
/// 
/// # Safety
/// 
/// The pointer must be valid and writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriterConfig_Init(config: *mut KoiWriterConfig) {
    let config = unsafe { config.as_mut() };
    if let Some(config) = config {
        let defaults = WriterConfig::default();
        config.global_options = defaults.global_options.into();
        config.command_threshold = defaults.command_threshold;
        config.command_options = ptr::null();
    }
}
