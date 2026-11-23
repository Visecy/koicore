#![doc = include_str!("../README.md")]

pub mod command;
pub mod parser;
pub mod writer;

pub use command::{Command, Parameter, Value};
pub use parser::{ParseError, Parser, ParserConfig};
pub use writer::{
    Anchor, ApplyMode, Edit, Expected, FormatterOptions, IndentPolicy, NewlineStyle, Patch,
    PatchEdit, PatchError, PatchOptions, PatchReport, PatchWarning, Writer, WriterConfig,
    apply_patch, apply_patch_with_config,
};
