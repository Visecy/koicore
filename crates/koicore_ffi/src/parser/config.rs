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
}

impl From<KoiParserConfig> for ParserConfig {
    fn from(config: KoiParserConfig) -> Self {
        Self {
            command_threshold: config.command_threshold,
            skip_annotations: config.skip_annotations,
        }
    }
}
