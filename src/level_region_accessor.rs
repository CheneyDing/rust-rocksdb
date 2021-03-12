// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.

use super::LevelRegionAccessorResult;
use crocksdb_ffi::{
    self, DBLevelRegionAccessor, DBLevelRegionAccessorRequest,
};
use libc::{c_char, c_void};
use std::{ffi::CString, slice};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LevelRegionAccessorRequest<'a> {
    pub smallest_user_key: &'a [u8],
    pub largest_user_key: &'a [u8],
}

pub trait LevelRegionAccessor {
    fn name(&self) -> &CString;
    fn level_regions(&self, req: &LevelRegionAccessorRequest) -> *const LevelRegionAccessorResult;
}

extern "C" fn level_region_destructor<P: LevelRegionAccessor>(ctx: *mut c_void) {
    unsafe {
        // Recover from raw pointer and implicitly drop.
        Box::from_raw(ctx as *mut P);
    }
}

extern "C" fn level_region_accessor_name<A: LevelRegionAccessor>(
    ctx: *mut c_void,
) -> *const c_char {
    let accessor = unsafe { &*(ctx as *mut A) };
    accessor.name().as_ptr()
}

extern "C" fn level_region_accessor_level_regions<'a, A: 'a + LevelRegionAccessor>(
    ctx: *mut c_void,
    request: *mut DBLevelRegionAccessorRequest,
) -> *const LevelRegionAccessorResult<'a> {
    let accessor = unsafe { &*(ctx as *mut A) };
    let req = unsafe {
        let mut smallest_key_len: usize = 0;
        let smallest_key = crocksdb_ffi::crocksdb_level_region_accessor_request_smallest_user_key(
            request,
            &mut smallest_key_len,
        ) as *const u8;
        let mut largest_key_len: usize = 0;
        let largest_key = crocksdb_ffi::crocksdb_level_region_accessor_request_largest_user_key(
            request,
            &mut largest_key_len,
        ) as *const u8;
        LevelRegionAccessorRequest {
            smallest_user_key: slice::from_raw_parts(smallest_key, smallest_key_len),
            largest_user_key: slice::from_raw_parts(largest_key, largest_key_len),
        }
    };
    accessor.level_regions(&req) as _
}

pub fn new_level_region_accessor<A: 'static + LevelRegionAccessor>(
    accessor: A,
) -> *mut DBLevelRegionAccessor {
    unsafe {
        crocksdb_ffi::crocksdb_level_region_accessor_create(
            Box::into_raw(Box::new(accessor)) as *mut c_void,
            level_region_destructor::<A>,
            level_region_accessor_name::<A>,
            level_region_accessor_level_regions::<A>,
        )
    }
}
