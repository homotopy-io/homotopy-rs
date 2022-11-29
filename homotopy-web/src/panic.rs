use std::{io::Write, sync::Mutex};

use homotopy_model::proof;
use wasm_bindgen::prelude::*;
use zip::write::{FileOptions, ZipWriter};

use crate::model::{generate_download, ModelError};

// This file contains all the disgusting
// panic handling logic which is responsible
// for producing crash dumps.
// Out of sight, out of mind, don't break it.
pub fn panic_handler(info: &std::panic::PanicInfo<'_>) {
    display_panic_message();

    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::hook(info);
}

#[derive(Default)]
struct CrashDump {
    actions: Vec<String>,
    import: Option<Vec<u8>>,
}

static CRASH_INFO: Mutex<CrashDump> = Mutex::new(CrashDump::new());

impl CrashDump {
    const fn new() -> Self {
        Self {
            actions: Vec::new(),
            import: None,
        }
    }

    fn push_action(&mut self, action: &proof::Action) {
        if let proof::Action::ImportProof(buf) = action {
            self.actions.clear();
            self.import = Some(buf.0.clone());
        } else {
            let data = serde_json::to_string(&action).expect("Failed to serialize action.");
            self.actions.push(data);
        }
    }

    fn pop_action(&mut self) -> bool {
        match (self.actions.pop().is_some(), self.import.is_some()) {
            (true, _) => true,
            (false, true) => {
                self.import = None;
                false
            }
            (false, false) => false,
        }
    }

    fn needs_zip(&self) -> bool {
        self.import.is_some()
    }

    fn get_dump(&self, safe: bool) -> Option<Vec<u8>> {
        let mut actions: Vec<u8> = Vec::new();
        if safe {
            actions.extend(b"[true,[");
        } else {
            actions.extend(b"[false,[");
        }
        actions.extend(self.actions.join(",").as_bytes());
        actions.extend(b"]]");

        if let Some(ibuf) = &self.import {
            let mut buf: Vec<u8> = vec![0; ibuf.len() + actions.len()];
            let size = {
                let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf[..]));
                let options =
                    FileOptions::default().compression_method(zip::CompressionMethod::DEFLATE);

                zip.start_file("crash_last_import.hom", options).ok()?;
                let mut j = 0;
                while j < ibuf.len() {
                    j += zip.write(&ibuf[j..]).ok()?;
                }

                zip.start_file("crash_action_dump.txt", options).ok()?;

                let mut j = 0;
                while j < actions.len() {
                    j += zip.write(&actions[j..]).ok()?;
                }
                zip.flush().ok()?;
                zip.finish().ok()?.position()
            };
            buf.resize(size as usize, 0);
            Some(buf)
        } else {
            Some(actions)
        }
    }
}

pub fn push_action(action: &proof::Action) {
    CRASH_INFO.lock().unwrap().push_action(action);
}

pub fn pop_action() -> bool {
    CRASH_INFO.lock().unwrap().pop_action()
}

pub fn needs_zip() -> bool {
    CRASH_INFO.lock().unwrap().needs_zip()
}

pub fn get_dump(safe: bool) -> Option<Vec<u8>> {
    CRASH_INFO.lock().unwrap().get_dump(safe)
}

pub fn export_dump(safe: bool) -> Result<(), ModelError> {
    let data = get_dump(safe).ok_or(ModelError::Internal)?;
    if needs_zip() {
        generate_download("homotopy_io_state", "zip", &data).map_err(ModelError::Export)
    } else {
        generate_download("homotopy_io_actions", "txt", &data).map_err(ModelError::Export)
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub fn display_panic_message();
}
