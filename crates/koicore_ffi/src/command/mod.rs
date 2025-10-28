//! # Command Module
//!
//! This module provides FFI functions for creating and manipulating KoiLang commands.
//! It allows C and other foreign languages to work with parsed commands, access their
//! parameters, and handle composite data structures.
//!
//! ## Main Components
//!
//! - [`KoiCommand`]: Represents a parsed KoiLang command with name and parameters
//! - [`KoiCompositeList`]: Represents a list data structure in KoiLang
//! - [`KoiCompositeDict`]: Represents a dictionary data structure in KoiLang
//!
//! ## Command Structure
//!
//! A KoiLang command consists of:
//! - A name (prefixed with `#` in the source text)
//! - Zero or more parameters of various types (integers, floats, strings, composites)
//!
//! ## Parameter Types
//!
//! - Integer parameters: 32-bit signed integers
//! - Float parameters: 64-bit floating-point numbers
//! - String parameters: UTF-8 encoded text
//! - Composite parameters: Lists or dictionaries containing other parameters
//!
//! ## Memory Management
//!
//! All objects created by this module must be freed using the corresponding `_Del` function
//! to avoid memory leaks. Composite objects (lists, dictionaries) own their elements,
//! which are automatically freed when the parent is deleted.

#[allow(clippy::module_inception)]
mod command;
mod param;
mod list;
mod dict;

pub use command::KoiCommand;
pub use list::KoiCompositeList;
pub use dict::KoiCompositeDict;
