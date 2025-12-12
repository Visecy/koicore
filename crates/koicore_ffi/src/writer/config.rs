use koicore::writer::{FormatterOptions, NumberFormat};
use std::ffi::c_char;
use std::ptr;

/// Number format for numeric values
#[repr(C)]
#[derive(Clone, Copy)]
pub enum KoiNumberFormat {
    Decimal = 0,
    Hex = 1,
    Octal = 2,
    Binary = 3,
}

impl From<KoiNumberFormat> for NumberFormat {
    fn from(format: KoiNumberFormat) -> Self {
        match format {
            KoiNumberFormat::Decimal => NumberFormat::Decimal,
            KoiNumberFormat::Hex => NumberFormat::Hex,
            KoiNumberFormat::Octal => NumberFormat::Octal,
            KoiNumberFormat::Binary => NumberFormat::Binary,
        }
    }
}

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
    pub number_format: KoiNumberFormat,
    pub newline_before_param: bool,
    pub newline_after_param: bool,
}

impl From<KoiFormatterOptions> for FormatterOptions {
    fn from(opt: KoiFormatterOptions) -> Self {
        Self {
            indent: opt.indent,
            use_tabs: opt.use_tabs,
            newline_before: opt.newline_before,
            newline_after: opt.newline_after,
            compact: opt.compact,
            force_quotes_for_vars: opt.force_quotes_for_vars,
            number_format: opt.number_format.into(),
            newline_before_param: opt.newline_before_param,
            newline_after_param: opt.newline_after_param,
        }
    }
}

/// Initialize KoiFormatterOptions with default values
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiFormatterOptions_Init(options: *mut KoiFormatterOptions) {
    if !options.is_null() {
        let defaults = FormatterOptions::default();
        let number_format = match defaults.number_format {
            NumberFormat::Decimal => KoiNumberFormat::Decimal,
            NumberFormat::Hex => KoiNumberFormat::Hex,
            NumberFormat::Octal => KoiNumberFormat::Octal,
            NumberFormat::Binary => KoiNumberFormat::Binary,
        };

        // Write directly to the pointer
        unsafe {
            (*options).indent = defaults.indent;
            (*options).use_tabs = defaults.use_tabs;
            (*options).newline_before = defaults.newline_before;
            (*options).newline_after = defaults.newline_after;
            (*options).compact = defaults.compact;
            (*options).force_quotes_for_vars = defaults.force_quotes_for_vars;
            (*options).number_format = number_format;
            (*options).newline_before_param = defaults.newline_before_param;
            (*options).newline_after_param = defaults.newline_after_param;
        }
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

/// Initialize KoiWriterConfig with default values
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriterConfig_Init(config: *mut KoiWriterConfig) {
    if !config.is_null() {
        unsafe {
            // First init global options
            KoiFormatterOptions_Init(&mut (*config).global_options);
            // koicore::writer::WriterConfig::default().command_threshold is 1
            (*config).command_threshold = 1;
            (*config).command_options = ptr::null();
        }
    }
}
