mod args;
mod fetch;
mod group;
mod print;
mod user;
mod util;

use std::collections::HashMap;

use args::Args;
use clap::Parser;
use fetch::get_users;
use group::{gid_for_group, group_names_for_user, user_in_group};
use print::print_table;
use util::parse_range;

fn main() {
    let args = Args::parse();
    let mut group_cache = HashMap::new();
    let users = get_users(args.all, &mut group_cache);

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

    let mut users: Vec<_> = users.into_iter().filter(|u| {
        if let Some(ref uid_range) = args.uid {
            let (min, max) = parse_range(uid_range);
            let uid: u32 = u.uid.parse().unwrap_or(0);
            if let Some(lo) = min { if uid < lo { return false; } }
            if let Some(hi) = max { if uid > hi { return false; } }
        }
        if let Some(ref gid_range) = args.gid {
            let (min, max) = parse_range(gid_range);
            let gid: u32 = u.gid.parse().unwrap_or(0);
            if let Some(lo) = min { if gid < lo { return false; } }
            if let Some(hi) = max { if gid > hi { return false; } }
        }
        if let Some(target_gid) = target_gid {
            let gid: u32 = u.gid.parse().unwrap_or(0);
            if !user_in_group(&u.user, gid, target_gid) {
                return false;
            }
        }
        true
    }).collect();

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

    if args.groups && !active_columns.contains(&"ALL_GROUP") {
        active_columns.push("ALL_GROUP");
    }

    if active_columns.iter().any(|col| *col == "ALL_GROUP") {
        for user in &mut users {
            let gid: u32 = user.gid.parse().unwrap_or(0);
            user.groups = group_names_for_user(&user.user, gid, &mut group_cache);
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
