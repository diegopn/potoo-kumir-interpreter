//! Profile-aware interpreter for the Kumir 2.1.0-rc11 language manuals.

pub mod reference;
pub mod subset;

pub use reference::{
    ArrayValue, Complex, Diagnostic, Execution, ExecutionContext, FileHost, FileRequest, Host,
    Interpreter, MAX_INTEGER, MAX_STEPS, MAX_TABLE_ELEMENTS, MIN_INTEGER, Module, ModuleArgument,
    ModuleArgumentMode, ModuleCall, ModuleCallResult, ModuleHost, Parameter, ParameterMode,
    Profile, Program, RuntimeError, Span, Type, Value,
};

/// Parses a source program using the classic Kumir 2.1 profile.
pub fn parse(source: &str) -> Result<Program, Vec<Diagnostic>> {
    reference::parse_classic(source)
}

/// Parses a source program using the explicitly selected Kumir 2.1 profile.
pub fn parse_with_profile(source: &str, profile: Profile) -> Result<Program, Vec<Diagnostic>> {
    reference::parse(source, profile)
}
