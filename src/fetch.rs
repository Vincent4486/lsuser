use std::collections::HashMap;

use crate::group::cached_group_name;
use crate::user::UserInfo;
use crate::util::safe_string;

pub fn get_users(show_all: bool, cache: &mut HashMap<u32, Option<String>>) -> Vec<UserInfo> {
    let mut users = Vec::new();
    let target_os = std::env::consts::OS;

    unsafe {
        libc::setpwent();
        loop {
            let pwd = libc::getpwent();
            if pwd.is_null() {
                break;
            }

            let name = safe_string((*pwd).pw_name);
            let uid = (*pwd).pw_uid;
            let gid = (*pwd).pw_gid;
            let group = cached_group_name(gid, cache).unwrap_or_else(|| "N/A".to_string());
            let gecos = safe_string((*pwd).pw_gecos);
            let real_name = gecos.split(',').next().unwrap_or("").to_string();
            let home = safe_string((*pwd).pw_dir);
            let shell = safe_string((*pwd).pw_shell);

            let is_system = if target_os == "macos" {
                name.starts_with('_') || uid < 501
            } else if target_os.ends_with("bsd") || target_os == "dragonfly" {
                uid < 1001 || uid == 65534
            } else {
                uid < 1000 || uid == 65534
            };

            if !show_all && is_system {
                continue;
            }

            users.push(UserInfo {
                user: name,
                uid: uid.to_string(),
                gid: gid.to_string(),
                group,
                groups: String::new(),
                real_name: if real_name.is_empty() { "N/A".to_string() } else { real_name },
                home,
                shell,
            });
        }
        libc::endpwent();
    }

    users.sort_by(|a, b| a.user.cmp(&b.user));
    users
}
