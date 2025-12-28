use koicore::ParserConfig;

#[repr(C)]
#[derive(Clone)]
pub struct KoiParserConfig {
    /// The command threshold (number of # required for commands)
    /// 
    /// Lines with fewer # characters than this threshold are treated as text.
    /// Lines with exactly this many # characters are treated as commands.
    /// Lines with more # characters are treated as annotations.
    pub command_threshold: usize,
    /// Whether to skip annotation lines (lines starting with #)
    ///
    /// If set to true, annotation lines will be skipped and not processed as commands.
    /// If set to false, annotation lines will be included in the output as special commands.
    pub skip_annotations: bool,
    /// Whether to convert number commands to special commands
    ///
    /// If set to true, commands with names that are valid integers will be converted
    /// to special number commands. If set to false, they will be treated as regular commands.
    pub convert_number_command: bool,
    /// Whether to preserve indentation in text and annotation lines
    ///
    /// If set to true, leading whitespace (indentation) will be preserved in text and
    /// annotation content. If set to false, leading whitespace will be trimmed.
    pub preserve_indent: bool,
    /// Whether to preserve empty lines as text commands
    ///
    /// If set to true, empty lines will be preserved and returned as empty text commands.
    /// If set to false, empty lines will be skipped.
    pub preserve_empty_lines: bool,
}

impl From<&KoiParserConfig> for ParserConfig {
    fn from(config: &KoiParserConfig) -> Self {
        Self {
            command_threshold: config.command_threshold,
            skip_annotations: config.skip_annotations,
            convert_number_command: config.convert_number_command,
            preserve_indent: config.preserve_indent,
            preserve_empty_lines: config.preserve_empty_lines,
        }
    }
}

/// Initialize a KoiParserConfig with default values
///
/// This function initializes a KoiParserConfig structure with sensible default values:
/// - command_threshold: 1 (lines starting with # are commands)
/// - skip_annotations: false (annotation lines are included in output)
/// - convert_number_command: true (numeric commands are converted to special commands)
///
/// # Arguments
/// * `config` - Pointer to the KoiParserConfig structure to initialize
///
/// # Safety
/// The config pointer must be a valid pointer to a KoiParserConfig structure.
/// If config is NULL, this function does nothing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiParserConfig_Init(config: *mut KoiParserConfig) {
    if config.is_null() {
        return;
    }

    unsafe {
        *config = KoiParserConfig {
            command_threshold: 1,
            skip_annotations: false,
            convert_number_command: true,
            preserve_empty_lines: true,
            preserve_indent: true,
        }
    };
}
