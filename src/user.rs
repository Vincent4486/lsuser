use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct UserInfo {
    pub user: String,
    pub uid: String,
    pub gid: String,
    pub group: String,
    pub groups: String,
    pub real_name: String,
    pub home: String,
    pub shell: String,
}
