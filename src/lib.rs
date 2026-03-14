pub mod cli;
pub mod codecs;
pub mod container;
pub mod converter;
pub mod core;
pub mod detector;
pub mod play;
pub mod probe;

pub mod filters;
pub mod io;
pub mod message;
pub mod utils;

pub use message::Result;

pub const EXIT_FAILURE: i32 = 1;
pub const EXIT_SUCCESS: i32 = 0;
