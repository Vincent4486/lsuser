use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;

#[derive(Parser, Debug)]
#[command(
    name = "lsuser",
    author = "Vincent Yang",
    version,
    about = "List system users in a clean, block-device-like columnar layout.",
    help_template = "Usage:\n    {usage}\n\nOptions:\n{options}"
)]
struct Args {
    #[arg(
        short,
        long,
        help = "Disable built-in filters and list all system daemon accounts."
    )]
    all: bool,

    #[arg(short, long, help = "Do not print a header line.")]
    noheadings: bool,

    #[arg(
        short,
        long,
        value_name = "LIST",
        help = "Specify which output columns to print (comma-separated). Available: USER,UID,GID,PRIMARY_GROUP,ALL_GROUP,REAL_NAME,HOME,SHELL."
    )]
    output: Option<String>,

    #[arg(short = 'O', long, help = "Output all available columns.")]
    output_all: bool,

    #[arg(short = 'J', long, help = "Use JSON output format.")]
    json: bool,

    #[arg(
        long,
        value_name = "RANGE",
        help = "Filter by UID range (e.g. 0, 0-1000, 1000-)."
    )]
    uid: Option<String>,

    #[arg(
        short = 'g',
        long,
        value_name = "RANGE",
        help = "Filter by GID range (e.g. 0, 0-1000, 1000-)."
    )]
    gid: Option<String>,

    #[arg(long, value_name = "NAME", help = "Filter by group name.")]
    group: Option<String>,

    #[arg(long, help = "Display all group memberships in an ALL_GROUP column.")]
    groups: bool,
}

#[derive(Serialize, Clone, Debug)]
struct UserInfo {
    user: String,
    uid: String,
    gid: String,
    group: String,
    groups: String,
    real_name: String,
    home: String,
    shell: String,
}

fn safe_string(ptr: *mut c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn parse_range(s: &str) -> (Option<u32>, Option<u32>) {
    let err = |msg: &str| -> ! {
        eprintln!("error: {msg}");
        std::process::exit(1);
    };
    if let Some(pos) = s.find('-') {
        let left = s[..pos].trim();
        let right = s[pos + 1..].trim();
        let min = if left.is_empty() {
            None
        } else {
            Some(
                left.parse()
                    .unwrap_or_else(|_| err(&format!("invalid number '{left}'"))),
            )
        };
        let max = if right.is_empty() {
            None
        } else {
            Some(
                right
                    .parse()
                    .unwrap_or_else(|_| err(&format!("invalid number '{right}'"))),
            )
        };
        (min, max)
    } else {
        let val = s
            .trim()
            .parse()
            .unwrap_or_else(|_| err(&format!("invalid value '{s}'")));
        (Some(val), Some(val))
    }
}

fn gid_for_group(name: &str) -> Option<u32> {
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

fn group_name_for_gid(gid: u32) -> Option<String> {
    unsafe {
        let grp = libc::getgrgid(gid as libc::gid_t);
        if grp.is_null() {
            None
        } else {
            Some(safe_string((*grp).gr_name))
        }
    }
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

fn group_names_for_user(user: &str, primary_gid: u32) -> String {
    let gids = group_ids_for_user(user, primary_gid).unwrap_or_else(|| vec![primary_gid]);
    if gids.is_empty() {
        return "N/A".to_string();
    }

    gids.into_iter()
        .map(|gid| group_name_for_gid(gid).unwrap_or_else(|| gid.to_string()))
        .collect::<Vec<String>>()
        .join(",")
}

fn user_in_group(user: &str, primary_gid: u32, target_gid: u32) -> bool {
    if primary_gid == target_gid {
        return true;
    }

    group_ids_for_user(user, primary_gid)
        .is_some_and(|gids| gids.into_iter().any(|gid| gid == target_gid))
}

fn get_users(show_all: bool) -> Vec<UserInfo> {
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
            let group = group_name_for_gid(gid).unwrap_or_else(|| "N/A".to_string());
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
                real_name: if real_name.is_empty() {
                    "N/A".to_string()
                } else {
                    real_name
                },
                home,
                shell,
            });
        }
        libc::endpwent();
    }

    users.sort_by(|a, b| a.user.cmp(&b.user));
    users
}

fn print_table(users: &[UserInfo], columns: &[&str], no_headings: bool) {
    if users.is_empty() {
        return;
    }

    let mut widths = HashMap::new();
    for col in columns {
        widths.insert(*col, col.len());
    }

    for user in users {
        for col in columns {
            let val_len = match *col {
                "USER" => user.user.len(),
                "UID" => user.uid.len(),
                "GID" => user.gid.len(),
                "PRIMARY_GROUP" => user.group.len(),
                "ALL_GROUP" => user.groups.len(),
                "REAL_NAME" => user.real_name.len(),
                "HOME" => user.home.len(),
                "SHELL" => user.shell.len(),
                _ => 0,
            };
            if let Some(current_max) = widths.get_mut(col) {
                if val_len > *current_max {
                    *current_max = val_len;
                }
            }
        }
    }

    if !no_headings {
        let header = columns
            .iter()
            .map(|col| format!("{:<width$}", col, width = widths[col]))
            .collect::<Vec<_>>()
            .join("  ");
        println!("{}", header);
    }

    for user in users {
        let row = columns
            .iter()
            .map(|col| {
                let val = match *col {
                    "USER" => &user.user,
                    "UID" => &user.uid,
                    "GID" => &user.gid,
                    "PRIMARY_GROUP" => &user.group,
                    "ALL_GROUP" => &user.groups,
                    "REAL_NAME" => &user.real_name,
                    "HOME" => &user.home,
                    "SHELL" => &user.shell,
                    _ => "",
                };
                format!("{:<width$}", val, width = widths[col])
            })
            .collect::<Vec<_>>()
            .join("  ");
        println!("{}", row);
    }
}

fn main() {
    let args = Args::parse();
    let users = get_users(args.all);

    let target_gid = if let Some(ref group_name) = args.group {
        match gid_for_group(group_name) {
            Some(gid) => Some(gid),
            None => {
                eprintln!("error: group '{group_name}' does not exist");
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let mut users: Vec<UserInfo> = users
        .into_iter()
        .filter(|u| {
            if let Some(ref uid_range) = args.uid {
                let (min, max) = parse_range(uid_range);
                let uid: u32 = u.uid.parse().unwrap_or(0);
                if let Some(lo) = min {
                    if uid < lo {
                        return false;
                    }
                }
                if let Some(hi) = max {
                    if uid > hi {
                        return false;
                    }
                }
            }
            if let Some(ref gid_range) = args.gid {
                let (min, max) = parse_range(gid_range);
                let gid: u32 = u.gid.parse().unwrap_or(0);
                if let Some(lo) = min {
                    if gid < lo {
                        return false;
                    }
                }
                if let Some(hi) = max {
                    if gid > hi {
                        return false;
                    }
                }
            }
            if let Some(target_gid) = target_gid {
                let gid: u32 = u.gid.parse().unwrap_or(0);
                if !user_in_group(&u.user, gid, target_gid) {
                    return false;
                }
            }
            true
        })
        .collect();

    let all_available = vec![
        "USER",
        "UID",
        "GID",
        "PRIMARY_GROUP",
        "ALL_GROUP",
        "REAL_NAME",
        "HOME",
        "SHELL",
    ];
    let default_columns = vec!["USER", "UID", "HOME", "SHELL"];

    let custom_cols;
    let mut active_columns: Vec<&str> = if args.output_all {
        all_available.clone()
    } else if let Some(ref o) = args.output {
        custom_cols = o
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .collect::<Vec<String>>();

        let filtered: Vec<&str> = custom_cols
            .iter()
            .map(|s| s.as_str())
            .filter(|s| all_available.contains(s))
            .collect();

        if filtered.is_empty() {
            default_columns
        } else {
            filtered
        }
    } else {
        default_columns
    };

    if args.groups && !active_columns.contains(&"ALL_GROUP") {
        active_columns.push("ALL_GROUP");
    }

    if active_columns.contains(&"ALL_GROUP") {
        for user in &mut users {
            let gid: u32 = user.gid.parse().unwrap_or(0);
            user.groups = group_names_for_user(&user.user, gid);
        }
    }

    if args.json {
        let mut json_users = Vec::new();
        for user in users {
            let mut map = serde_json::Map::new();
            for col in &active_columns {
                let val = match *col {
                    "USER" => user.user.clone(),
                    "UID" => user.uid.clone(),
                    "GID" => user.gid.clone(),
                    "PRIMARY_GROUP" => user.group.clone(),
                    "ALL_GROUP" => user.groups.clone(),
                    "REAL_NAME" => user.real_name.clone(),
                    "HOME" => user.home.clone(),
                    "SHELL" => user.shell.clone(),
                    _ => String::new(),
                };
                map.insert(col.to_string(), serde_json::Value::String(val));
            }
            json_users.push(serde_json::Value::Object(map));
        }

        let wrapper = serde_json::json!({ "users": json_users });
        println!("{}", serde_json::to_string_pretty(&wrapper).unwrap());
        return;
    }

    print_table(&users, &active_columns, args.noheadings);
}
