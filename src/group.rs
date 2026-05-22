use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_char;

use crate::util::safe_string;

pub fn gid_for_group(name: &str) -> Option<u32> {
    let c_name = CString::new(name).ok()?;
    unsafe {
        let grp = libc::getgrnam(c_name.as_ptr());
        if grp.is_null() {
            None
        } else {
            Some((*grp).gr_gid)
        }
    }
}

pub fn group_name_for_gid(gid: u32) -> Option<String> {
    unsafe {
        let grp = libc::getgrgid(gid as libc::gid_t);
        if grp.is_null() {
            None
        } else {
            Some(safe_string((*grp).gr_name))
        }
    }
}

pub fn cached_group_name(gid: u32, cache: &mut HashMap<u32, Option<String>>) -> Option<String> {
    cache.entry(gid).or_insert_with(|| group_name_for_gid(gid)).clone()
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
)))]
fn group_ids_for_user(user: &str, primary_gid: u32) -> Option<Vec<u32>> {
    let c_user = CString::new(user).ok()?;
    let mut size: libc::c_int = 16;
    loop {
        let mut groups = vec![0 as libc::gid_t; size as usize];
        let mut ngroups = size;
        let ret = unsafe {
            libc::getgrouplist(
                c_user.as_ptr(),
                primary_gid as libc::gid_t,
                groups.as_mut_ptr(),
                &mut ngroups,
            )
        };

        if ret >= 0 {
            return Some(
                groups
                    .into_iter()
                    .take(ngroups as usize)
                    .map(|gid| gid as u32)
                    .collect(),
            );
        }

        if ngroups > size {
            size = ngroups;
        } else {
            size = size.saturating_mul(2);
        }

        if size <= 0 || size > 4096 {
            return None;
        }
    }
}

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
))]
fn group_ids_for_user(user: &str, primary_gid: u32) -> Option<Vec<u32>> {
    let c_user = CString::new(user).ok()?;
    let mut size: libc::c_int = 16;
    loop {
        let mut groups = vec![0 as libc::c_int; size as usize];
        let mut ngroups = size;
        let ret = unsafe {
            libc::getgrouplist(
                c_user.as_ptr(),
                primary_gid as libc::c_int,
                groups.as_mut_ptr(),
                &mut ngroups,
            )
        };

        if ret >= 0 {
            return Some(
                groups
                    .into_iter()
                    .take(ngroups as usize)
                    .map(|gid| gid as u32)
                    .collect(),
            );
        }

        if ngroups > size {
            size = ngroups;
        } else {
            size = size.saturating_mul(2);
        }

        if size <= 0 || size > 4096 {
            return None;
        }
    }
}

pub fn group_names_for_user(user: &str, primary_gid: u32, cache: &mut HashMap<u32, Option<String>>) -> String {
    let gids = group_ids_for_user(user, primary_gid).unwrap_or_else(|| vec![primary_gid]);
    if gids.is_empty() {
        return "N/A".to_string();
    }

    gids.into_iter()
        .map(|gid| cached_group_name(gid, cache).unwrap_or_else(|| gid.to_string()))
        .collect::<Vec<String>>()
        .join(",")
}

pub fn user_in_group(user: &str, primary_gid: u32, target_gid: u32) -> bool {
    if primary_gid == target_gid {
        return true;
    }

    group_ids_for_user(user, primary_gid)
        .map_or(false, |gids| gids.into_iter().any(|gid| gid == target_gid))
}
