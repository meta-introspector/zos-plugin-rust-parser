//! Jocko fuzz test — exercises all C FFI exports with edge-case inputs

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

extern "C" {
    fn zos_plugin_name() -> *mut c_char;
    fn zos_plugin_version() -> *mut c_char;
    fn zos_plugin_commands() -> *mut c_char;
    fn zos_plugin_execute(cmd: *const c_char, arg: *const c_char) -> *mut c_char;
    fn zos_plugin_render() -> *mut c_char;
    fn zos_plugin_init() -> i32;
    fn zos_free_string(s: *mut c_char);
}

fn c_call(cmd: &str, arg: &str) -> String {
    let c = CString::new(cmd).unwrap();
    let a = CString::new(arg).unwrap();
    unsafe {
        let r = zos_plugin_execute(c.as_ptr(), a.as_ptr());
        let s = CStr::from_ptr(r).to_string_lossy().to_string();
        zos_free_string(r);
        s
    }
}

fn c_str(f: unsafe extern "C" fn() -> *mut c_char) -> String {
    unsafe { let p = f(); let s = CStr::from_ptr(p).to_string_lossy().to_string(); zos_free_string(p); s }
}

#[test]
fn jocko_init() { unsafe { assert_eq!(zos_plugin_init(), 0); } }

#[test]
fn jocko_identity() {
    let name = c_str(zos_plugin_name);
    let ver = c_str(zos_plugin_version);
    assert!(!name.is_empty());
    assert!(ver.contains("0."));
}

#[test]
fn jocko_render_gui() {
    let gui = c_str(zos_plugin_render);
    assert!(gui.starts_with("["), "GUI should be JSON array: {}", gui);
    let v: serde_json::Value = serde_json::from_str(&gui).unwrap();
    assert!(v.is_array());
}

#[test]
fn jocko_fuzz_all_commands() {
    let cmds = c_str(zos_plugin_commands);
    let inputs = ["", "42", "hello world", "0", "999999999", "{}", "null", "🧮🦀"];
    for cmd in cmds.split(',') {
        for input in &inputs {
            let result = c_call(cmd, input);
            let v: serde_json::Value = serde_json::from_str(&result).expect(&format!("bad JSON: cmd={} input={}", cmd, input));
            assert!(v.get("shard").is_some(), "missing shard: cmd={} input={}", cmd, input);
        }
    }
}

#[test]
fn jocko_da51_shard_valid() {
    let cmds = c_str(zos_plugin_commands);
    let first_cmd = cmds.split(',').next().unwrap();
    let result = c_call(first_cmd, "42");
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    let shard = &v["shard"];
    assert!(shard["cid"].as_str().unwrap().starts_with("bafk"));
    assert!(shard["dasl"].as_str().unwrap().starts_with("0xda51"));
    let orb = shard["orbifold"].as_array().unwrap();
    assert_eq!(orb.len(), 3);
    assert!(orb[0].as_u64().unwrap() < 71);
    assert!(orb[1].as_u64().unwrap() < 59);
    assert!(orb[2].as_u64().unwrap() < 47);
}

#[test]
fn jocko_unknown_command() {
    let result = c_call("nonexistent_command_xyz", "");
    let v: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(v["result"].get("error").is_some());
}
