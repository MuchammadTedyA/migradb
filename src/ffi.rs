use crate::migrator::{Migrator, MigratorBuilder};
use crate::tracker::InMemoryTracker;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::Arc;

pub struct MigratorHandle {
    migrator: Migrator,
}

#[no_mangle]
pub extern "C" fn migrator_new(migrations_dir: *const c_char) -> *mut MigratorHandle {
    if migrations_dir.is_null() {
        return std::ptr::null_mut();
    }

    let dir_str = unsafe { CStr::from_ptr(migrations_dir) };
    let dir_path = match dir_str.to_str() {
        Ok(s) => PathBuf::from(s),
        Err(_) => return std::ptr::null_mut(),
    };

    let tracker = Arc::new(InMemoryTracker::new());
    let migrator = MigratorBuilder::new()
        .migrations_dir(dir_path)
        .build(tracker);

    Box::into_raw(Box::new(MigratorHandle { migrator }))
}

#[no_mangle]
pub extern "C" fn migrator_free(handle: *mut MigratorHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

#[no_mangle]
pub extern "C" fn migrator_run(handle: *mut MigratorHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }

    let handle = unsafe { &mut *handle };

    match handle.migrator.run() {
        Ok(migrations) => {
            let result = serde_json::json!({
                "success": true,
                "applied": migrations.len(),
                "migrations": migrations.iter().map(|m| {
                    serde_json::json!({
                        "version": m.version,
                        "name": m.name,
                    })
                }).collect::<Vec<_>>()
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": e.to_string(),
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn migrator_status(handle: *mut MigratorHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }

    let handle = unsafe { &mut *handle };

    match handle.migrator.status() {
        Ok(status_list) => {
            let result = serde_json::json!({
                "success": true,
                "migrations": status_list.iter().map(|s| {
                    serde_json::json!({
                        "version": s.version,
                        "name": s.name,
                        "applied": s.applied,
                        "applied_at": s.applied_at.map(|dt| dt.to_rfc3339()),
                    })
                }).collect::<Vec<_>>()
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": e.to_string(),
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn migrator_create(
    handle: *mut MigratorHandle,
    name: *const c_char,
    content: *const c_char,
) -> *mut c_char {
    if handle.is_null() || name.is_null() || content.is_null() {
        return std::ptr::null_mut();
    }

    let handle = unsafe { &mut *handle };
    let name_str = unsafe { CStr::from_ptr(name) };
    let content_str = unsafe { CStr::from_ptr(content) };

    let name = match name_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let content = match content_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match handle.migrator.create_migration(name, content) {
        Ok(path) => {
            let result = serde_json::json!({
                "success": true,
                "path": path.to_string_lossy().to_string(),
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": e.to_string(),
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn migrator_remove_pending(handle: *mut MigratorHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }

    let handle = unsafe { &mut *handle };

    match handle.migrator.remove_pending() {
        Ok(Some(path)) => {
            let result = serde_json::json!({
                "success": true,
                "removed": path.to_string_lossy().to_string(),
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
        Ok(None) => {
            let result = serde_json::json!({
                "success": true,
                "removed": null,
                "message": "No pending migrations to remove",
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": e.to_string(),
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn migrator_generate_models(
    handle: *mut MigratorHandle,
    schema_path: *const c_char,
    target_lang: *const c_char,
    output_dir: *const c_char,
    package_name: *const c_char,
) -> *mut c_char {
    if handle.is_null() || schema_path.is_null() || target_lang.is_null() || output_dir.is_null() {
        return std::ptr::null_mut();
    }

    let handle = unsafe { &mut *handle };
    let schema_str = match unsafe { CStr::from_ptr(schema_path) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let lang_str = match unsafe { CStr::from_ptr(target_lang) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let out_dir_str = match unsafe { CStr::from_ptr(output_dir) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let pkg_opt = if package_name.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(package_name) }.to_str() {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    };

    match handle.migrator.generate_models(schema_str, lang_str, out_dir_str, pkg_opt) {
        Ok(paths) => {
            let result = serde_json::json!({
                "success": true,
                "count": paths.len(),
                "files": paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>()
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": e.to_string(),
            });
            CString::new(result.to_string()).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn migrator_diff_drawdb(
    handle: *mut MigratorHandle,
    schema_path: *const c_char,
    migration_name: *const c_char,
    dialect: *const c_char,
    force_full: bool,
) -> *mut c_char {
    if handle.is_null() || schema_path.is_null() {
        return std::ptr::null_mut();
    }

    let handle = unsafe { &mut *handle };
    let schema_str = match unsafe { CStr::from_ptr(schema_path) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let name_str = if migration_name.is_null() {
        "sync_drawdb"
    } else {
        match unsafe { CStr::from_ptr(migration_name) }.to_str() {
            Ok(s) if !s.is_empty() => s,
            _ => "sync_drawdb",
        }
    };
    let dialect_opt = if dialect.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(dialect) }.to_str() {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    };

    match handle.migrator.diff_drawdb(schema_str, name_str, dialect_opt, force_full) {
        Ok(res) => {
            let json = serde_json::json!({
                "success": res.success,
                "migration_path": res.migration_path,
                "diff_summary": res.diff_summary,
                "is_empty": res.is_empty,
                "statements": res.statements,
                "error": res.error,
            });
            CString::new(json.to_string()).unwrap().into_raw()
        }
        Err(e) => {
            let json = serde_json::json!({
                "success": false,
                "diff_summary": "",
                "is_empty": false,
                "statements": Vec::<String>::new(),
                "error": e.to_string(),
            });
            CString::new(json.to_string()).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn migrator_plan_sync_drawdb(
    handle: *mut MigratorHandle,
    schema_path: *const c_char,
    dialect: *const c_char,
    force_full: bool,
) -> *mut c_char {
    if handle.is_null() || schema_path.is_null() {
        return std::ptr::null_mut();
    }

    let handle = unsafe { &mut *handle };
    let schema_str = match unsafe { CStr::from_ptr(schema_path) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let dialect_opt = if dialect.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(dialect) }.to_str() {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    };

    match handle.migrator.plan_sync_drawdb(schema_str, dialect_opt, force_full) {
        Ok(res) => {
            let json = serde_json::json!({
                "success": res.success,
                "is_empty": res.is_empty,
                "diff_summary": res.diff_summary,
                "statements": res.statements,
                "sql": res.sql,
                "dialect": res.dialect,
                "error": res.error,
            });
            CString::new(json.to_string()).unwrap().into_raw()
        }
        Err(e) => {
            let json = serde_json::json!({
                "success": false,
                "is_empty": false,
                "diff_summary": "",
                "statements": Vec::<String>::new(),
                "sql": "",
                "dialect": "",
                "error": e.to_string(),
            });
            CString::new(json.to_string()).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn migrator_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
