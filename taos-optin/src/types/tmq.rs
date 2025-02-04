use std::{borrow::Cow, ffi::c_void};

use taos_query::prelude::RawError;

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct tmq_resp_err_t(i32);

impl PartialEq<i32> for tmq_conf_res_t {
    fn eq(&self, other: &i32) -> bool {
        self == other
    }
}

impl tmq_resp_err_t {
    pub fn ok_or(self, s: impl Into<Cow<'static, str>>) -> Result<(), RawError> {
        match self {
            Self(0) => Ok(()),
            _ => Err(RawError::from_string(s.into())),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tmq_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tmq_conf_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tmq_list_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tmq_message_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]
pub enum tmq_conf_res_t {
    Unknown = -2,
    Invalid = -1,
    Ok = 0,
}

impl tmq_conf_res_t {
    pub fn ok(self, k: &str, v: &str) -> Result<(), RawError> {
        match self {
            Self::Ok => Ok(()),
            Self::Invalid => Err(RawError::from_string(format!(
                "Invalid key value pair ({k}, {v})"
            ))),
            Self::Unknown => Err(RawError::from_string(format!("Unknown key {k}"))),
        }
    }
}

pub(crate) type tmq_commit_cb =
    unsafe extern "C" fn(tmq: *mut tmq_t, resp: tmq_resp_err_t, param: *mut c_void);

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum tmq_res_t {
    TMQ_RES_INVALID = -1,
    TMQ_RES_DATA = 1,
    TMQ_RES_TABLE_META = 2,
    TMQ_RES_METADATA = 3,
}
