use std::collections::HashMap;

use crate::user::UserInfo;

pub fn print_table(users: &[UserInfo], columns: &[&str], no_headings: bool) {
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
