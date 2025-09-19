use crate::{EtherlinkError, Result};
use libc::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};
use std::ptr;
use tracing::{debug, error, warn};

/// FFI bridge for Rust ↔ Zig interoperability
#[derive(Debug)]
pub struct ZigBridge {
    initialized: bool,
}

impl ZigBridge {
    /// Create a new Zig bridge instance
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    /// Initialize the Zig bridge
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            warn!("Zig bridge already initialized");
            return Ok(());
        }

        debug!("Initializing Zig bridge");

        // TODO: Initialize actual Zig FFI once ghostplane is integrated
        self.initialized = true;

        debug!("Zig bridge initialized successfully");
        Ok(())
    }

    /// Check if the bridge is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Call a Zig function with parameters
    pub async fn call_zig_function(&self, function_name: &str, params: &[u8]) -> Result<Vec<u8>> {
        if !self.initialized {
            return Err(EtherlinkError::Ffi("Bridge not initialized".to_string()));
        }

        debug!("Calling Zig function: {}", function_name);

        // TODO: Implement actual Zig FFI calls once ghostplane is integrated
        // For now, return empty response
        Ok(Vec::new())
    }

    /// Submit a transaction to GhostPlane via FFI
    pub async fn submit_ghostplane_transaction(&self, tx_data: &[u8]) -> Result<String> {
        if !self.initialized {
            return Err(EtherlinkError::Ffi("Bridge not initialized".to_string()));
        }

        debug!("Submitting transaction to GhostPlane");

        // TODO: Implement actual GhostPlane transaction submission
        Ok("0x1234567890abcdef".to_string())
    }

    /// Query GhostPlane state via FFI
    pub async fn query_ghostplane_state(&self, query: &str) -> Result<String> {
        if !self.initialized {
            return Err(EtherlinkError::Ffi("Bridge not initialized".to_string()));
        }

        debug!("Querying GhostPlane state: {}", query);

        // TODO: Implement actual GhostPlane state query
        Ok("{}".to_string())
    }

    /// Shutdown the Zig bridge
    pub fn shutdown(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        debug!("Shutting down Zig bridge");

        // TODO: Cleanup Zig FFI resources
        self.initialized = false;

        debug!("Zig bridge shutdown complete");
        Ok(())
    }
}

impl Default for ZigBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ZigBridge {
    fn drop(&mut self) {
        if self.initialized {
            if let Err(e) = self.shutdown() {
                error!("Error during Zig bridge shutdown: {}", e);
            }
        }
    }
}

/// Safe FFI helpers for C string conversion
pub mod ffi_helpers {
    use super::*;

    /// Convert Rust string to C string safely
    pub fn rust_to_c_string(input: &str) -> Result<CString> {
        CString::new(input).map_err(|e| EtherlinkError::Ffi(format!("Invalid C string: {}", e)))
    }

    /// Convert C string to Rust string safely
    pub unsafe fn c_to_rust_string(c_str: *const c_char) -> Result<String> {
        if c_str.is_null() {
            return Err(EtherlinkError::Ffi("Null C string pointer".to_string()));
        }

        unsafe { CStr::from_ptr(c_str) }
            .to_str()
            .map(|s| s.to_string())
            .map_err(|e| EtherlinkError::Ffi(format!("Invalid UTF-8 in C string: {}", e)))
    }

    /// Convert byte slice to C-compatible buffer
    pub fn bytes_to_c_buffer(data: &[u8]) -> (*const u8, usize) {
        (data.as_ptr(), data.len())
    }

    /// Convert C buffer to Rust byte vector safely
    pub unsafe fn c_buffer_to_bytes(ptr: *const u8, len: usize) -> Result<Vec<u8>> {
        if ptr.is_null() {
            return Err(EtherlinkError::Ffi("Null buffer pointer".to_string()));
        }

        if len == 0 {
            return Ok(Vec::new());
        }

        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        Ok(slice.to_vec())
    }
}

// External C/Zig function declarations (to be implemented)
unsafe extern "C" {
    // Placeholder for future Zig FFI functions
    fn ghostplane_init() -> c_int;
    fn ghostplane_submit_tx(data: *const c_void, len: usize) -> *const c_char;
    fn ghostplane_query_state(query: *const c_char) -> *const c_char;
    fn ghostplane_cleanup() -> c_int;
}

/// Low-level FFI interface (unsafe, for internal use only)
pub mod low_level {
    use super::*;

    /// Initialize GhostPlane via FFI (unsafe)
    pub unsafe fn init_ghostplane() -> Result<()> {
        let result = unsafe { ghostplane_init() };
        if result == 0 {
            Ok(())
        } else {
            Err(EtherlinkError::Ffi(format!("GhostPlane init failed with code: {}", result)))
        }
    }

    /// Submit transaction to GhostPlane via FFI (unsafe)
    pub unsafe fn submit_transaction_raw(data: &[u8]) -> Result<String> {
        let result_ptr = unsafe { ghostplane_submit_tx(data.as_ptr() as *const c_void, data.len()) };
        unsafe { ffi_helpers::c_to_rust_string(result_ptr) }
    }

    /// Query GhostPlane state via FFI (unsafe)
    pub unsafe fn query_state_raw(query: &str) -> Result<String> {
        let c_query = ffi_helpers::rust_to_c_string(query)?;
        let result_ptr = unsafe { ghostplane_query_state(c_query.as_ptr()) };
        unsafe { ffi_helpers::c_to_rust_string(result_ptr) }
    }

    /// Cleanup GhostPlane via FFI (unsafe)
    pub unsafe fn cleanup_ghostplane() -> Result<()> {
        let result = unsafe { ghostplane_cleanup() };
        if result == 0 {
            Ok(())
        } else {
            Err(EtherlinkError::Ffi(format!("GhostPlane cleanup failed with code: {}", result)))
        }
    }
}