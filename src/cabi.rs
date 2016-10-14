use std::ptr;
use std::mem;
use std::slice;
use std::os::raw::{c_int, c_uint};

use errors::Error;
use unified::View;


unsafe fn notify_err<T>(err: Error, err_out: *mut *mut u8) -> *mut T {
    if !err_out.is_null() {
        let s = format!("{}\x00", err);
        *err_out = Box::into_raw(s.into_boxed_str()) as *mut u8;
    }
    0 as *mut T
}

#[no_mangle]
pub unsafe fn lsm_view_from_json(bytes: *const u8, len: c_uint, err_out: *mut *mut u8) -> *mut View {
    match View::json_from_slice(slice::from_raw_parts(
        mem::transmute(bytes),
        len as usize
    )) {
        Ok(v) => Box::into_raw(Box::new(v)),
        Err(err) => notify_err(err, err_out)
    }
}

#[no_mangle]
pub unsafe fn lsm_view_from_memdb(bytes: *const u8, len: c_uint, err_out: *mut *mut u8) -> *mut View {
    // XXX: this currently copies because that's safer.  Consider improving this?
    match View::memdb_from_vec(slice::from_raw_parts(
        mem::transmute(bytes),
        len as usize
    ).to_vec()) {
        Ok(v) => Box::into_raw(Box::new(v)),
        Err(err) => notify_err(err, err_out)
    }
}

#[no_mangle]
pub unsafe fn lsm_view_free(view: *mut View) {
    if !view.is_null() {
        Box::from_raw(view);
    }
}

#[no_mangle]
pub unsafe fn lsm_view_lookup_token(view: *const View, line: u32, col: u32,
                                    src_line_out: *mut u32,
                                    src_col_out: *mut u32,
                                    name_out: *mut *const u8,
                                    src_out: *mut *const u8,
                                    src_id_out: *mut u32) -> c_int {
    match (*view).lookup_token(line, col) {
        None => 0,
        Some(tm) => {
            *src_line_out = tm.line;
            *src_col_out = tm.col;
            *name_out = match tm.name {
                Some(name) => name.as_ptr(),
                None => ptr::null()
            };
            *src_out = tm.src.as_ptr();
            *src_id_out = tm.src_id;
            1
        }
    }
}

#[no_mangle]
pub unsafe fn lsm_view_get_source_contents(view: *const View, src_id: u32, len_out: *mut u32) -> *const u8 {
    match (*view).get_source_contents(src_id) {
        None => ptr::null(),
        Some(contents) => {
            *len_out = contents.len() as u32;
            contents.as_ptr()
        }
    }
}

#[no_mangle]
pub unsafe fn lsm_view_dump_memdb(view: *mut View, len_out: *mut c_uint) -> *mut u8 {
    let memdb = (*view).dump_memdb();
    *len_out = memdb.len() as c_uint;
    Box::into_raw(memdb.into_boxed_slice()) as *mut u8
}

#[no_mangle]
pub unsafe fn lsm_buffer_free(buf: *mut u8) {
    if !buf.is_null() {
        Box::from_raw(buf);
    }
}
