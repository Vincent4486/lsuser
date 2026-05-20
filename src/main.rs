use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Parser, Debug)]
#[command(
    name = "lsuser",
    author,
    version,
    about = "List system users in a clean, block-device-like columnar layout.",
    help_template = "Usage:\n    {usage}\n\nOptions:\n{options}"
)]
struct Args {
    #[arg(short, long, help = "Disable built-in filters and list all system daemon accounts.")]
    all: bool,

    #[arg(short, long, help = "Do not print a header line.")]
    noheadings: bool,

    #[arg(short, long, value_name = "LIST", help = "Specify which output columns to print (comma-separated).")]
    output: Option<String>,

    #[arg(short = 'O', long, help = "Output all available columns.")]
    output_all: bool,

    #[arg(short = 'J', long, help = "Use JSON output format.")]
    json: bool,
}

#[derive(Serialize, Clone, Debug)]
struct UserInfo {
    user: String,
    uid: String,
    gid: String,
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
            let gecos = safe_string((*pwd).pw_gecos);
            let real_name = gecos.split(',').next().unwrap_or("").to_string();
            let home = safe_string((*pwd).pw_dir);
            let shell = safe_string((*pwd).pw_shell);

            let is_system = if target_os == "macos" {
                name.starts_with('_') || uid < 500
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

    let all_available = vec!["USER", "UID", "GID", "REAL_NAME", "HOME", "SHELL"];
    let default_columns = vec!["USER", "UID", "HOME", "SHELL"];

    let custom_cols;
    let active_columns: Vec<&str> = if args.output_all {
        all_available.clone()
    } else if let Some(ref o) = args.output {
        custom_cols = o.split(',')
            .map(|s| s.trim().to_uppercase())
            .collect::<Vec<String>>();
        
        let filtered: Vec<&str> = custom_cols.iter()
            .map(|s| s.as_str())
            .filter(|s| all_available.contains(s))
            .collect();
            
        if filtered.is_empty() { default_columns } else { filtered }
    } else {
        default_columns
    };

    if args.json {
        let mut json_users = Vec::new();
        for user in users {
            let mut map = serde_json::Map::new();
            for col in &active_columns {
                let val = match *col {
                    "USER" => user.user.clone(),
                    "UID" => user.uid.clone(),
                    "GID" => user.gid.clone(),
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
