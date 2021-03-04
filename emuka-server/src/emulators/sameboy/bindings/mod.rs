#[cfg(target_os = "linux")]
pub mod bindings_unix;
#[cfg(target_os = "linux")]
pub use bindings_unix as bindings;

#[cfg(target_os = "windows")]
pub mod bindings_win;
#[cfg(target_os = "windows")]
pub use bindings_win as bindings;