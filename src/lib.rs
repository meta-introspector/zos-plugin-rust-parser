mod plugin_trait;
use plugin_trait::*;
use serde_json::json;
use std::os::raw::c_char;

#[no_mangle] pub extern "C" fn zos_plugin_name() -> *mut c_char { to_c("rust-parser") }
#[no_mangle] pub extern "C" fn zos_plugin_version() -> *mut c_char { to_c("0.2.0") }
#[no_mangle] pub extern "C" fn zos_plugin_commands() -> *mut c_char { to_c("parse,functions,structs") }

#[no_mangle] pub extern "C" fn zos_plugin_execute(cmd: *const c_char, arg: *const c_char) -> *mut c_char {
    let cmd = unsafe { std::ffi::CStr::from_ptr(cmd) }.to_str().unwrap_or("");
    let arg = unsafe { std::ffi::CStr::from_ptr(arg) }.to_str().unwrap_or("");
    let result = json!({"plugin": "rust-parser", "command": cmd, "arg": arg, "status": "ok"});
    let shard = DA51Shard::from_result("rust-parser", cmd, &result);
    to_c(&serde_json::to_string(&json!({"result": result, "shard": shard})).unwrap())
}

#[no_mangle] pub extern "C" fn zos_plugin_render() -> *mut c_char {
    let gui = vec![
        GuiComponent::Heading { level: 2, text: "🦀 Rust Parser".into() },
        GuiComponent::Button { label: "Parse".into(), command: "parse".into() },
    ];
    to_c(&serde_json::to_string(&gui).unwrap())
}

#[no_mangle] pub extern "C" fn zos_plugin_init() -> i32 { 0 }

#[cfg(test)]
mod jocko_fuzz {
    use super::*;
    use std::ffi::{CStr, CString};

    fn s(p: *mut c_char) -> String { unsafe { let s = CStr::from_ptr(p).to_string_lossy().into(); zos_free_string(p); s } }
    fn ex(cmd: &str, arg: &str) -> String {
        let c = CString::new(cmd).unwrap(); let a = CString::new(arg).unwrap();
        s(unsafe { zos_plugin_execute(c.as_ptr(), a.as_ptr()) })
    }

    #[test] fn init() { unsafe { assert_eq!(zos_plugin_init(), 0); } }
    #[test] fn identity() { assert!(!s(unsafe{zos_plugin_name()}).is_empty()); }
    #[test] fn render_gui() { assert!(s(unsafe{zos_plugin_render()}).starts_with("[")); }

    #[test] fn fuzz_all_commands() {
        let cmds = s(unsafe{zos_plugin_commands()});
        for cmd in cmds.split(',') {
            for input in &["", "42", "hello", "0", "999999", "{}", "🧮"] {
                let r = ex(cmd, input);
                let v: serde_json::Value = serde_json::from_str(&r).unwrap();
                assert!(v.get("shard").is_some(), "no shard: {}({})", cmd, input);
            }
        }
    }

    #[test] fn da51_valid() {
        let r = ex(&s(unsafe{zos_plugin_commands()}).split(',').next().unwrap().to_string(), "42");
        let v: serde_json::Value = serde_json::from_str(&r).unwrap();
        assert!(v["shard"]["cid"].as_str().unwrap().starts_with("bafk"));
        assert!(v["shard"]["dasl"].as_str().unwrap().starts_with("0xda51"));
        assert_eq!(v["shard"]["orbifold"].as_array().unwrap().len(), 3);
    }
}
