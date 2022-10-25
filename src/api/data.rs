use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub trait Success {
  fn is_success(&self) -> bool;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTokenResp {
  #[serde(rename = "errcode")]
  pub code: Option<i32>,
  #[serde(rename = "errmsg")]
  pub msg: Option<String>,
  pub access_token: Option<String>,
  pub expires_in: Option<u32>,
}

impl Success for GetTokenResp {
  fn is_success(&self) -> bool {
    self.access_token.is_some()
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentListResp {
  #[serde(rename = "errcode")]
  pub code: Option<i32>,
  #[serde(rename = "errmsg")]
  pub msg: Option<String>,
  #[serde(rename = "agentlist")]
  pub agent_list: Vec<AgentBasic>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentBasic {
  #[serde(rename = "agentid")]
  pub id: u32,
  pub name: String,
  pub square_logo_url: Option<String>,
  pub round_logo_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentDetail {
  #[serde(rename = "errcode")]
  pub code: Option<i32>,
  #[serde(rename = "errmsg")]
  pub msg: Option<String>,
  #[serde(rename = "agentid")]
  pub agent_id: Option<u32>,
  pub square_logo_url: Option<String>,
  pub description: Option<String>,
  pub allow_userinfos: Option<AllowUserInfos>,
  #[serde(rename = "allow_partys")]
  pub allow_parties: Option<AllowParties>,
  pub allow_tags: Option<AllowTags>,
  pub close: Option<u32>,
  pub redirect_domain: Option<String>,
  pub report_location_flag: Option<u32>,
  #[serde(rename = "isreportenter")]
  pub is_report_enter: Option<u32>,
  pub home_url: Option<String>,
  #[serde(rename = "customized_publish_status")]
  pub publish_status: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AllowUserInfos {
  pub user: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
  #[serde(rename = "userid")]
  pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AllowParties {
  #[serde(rename = "partyid")]
  pub party_id: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AllowTags {
  #[serde(rename = "tagid")]
  pub tag_id: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DepartmentResp {
  #[serde(rename = "errcode")]
  pub code: Option<i32>,
  #[serde(rename = "errmsg")]
  pub msg: Option<String>,
  #[serde(rename = "department")]
  pub departments: Vec<Department>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Department {
  pub id: u32,
  pub name: String,
  #[serde(rename = "parentid")]
  pub parent_id: Option<u32>,
  pub order: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DepartmentMembersResp {
  #[serde(rename = "errcode")]
  pub code: Option<i32>,
  #[serde(rename = "errmsg")]
  pub msg: Option<String>,
  #[serde(rename = "userlist")]
  pub members: Vec<DepartmentMember>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DepartmentMember {
  pub name: String,
  pub department: Vec<u32>,
  pub position: String,
  pub mobile: String,
  pub gender: String,
  pub email: String,
  pub avatar: String,
  #[serde(rename = "isleader")]
  pub is_leader: u32,
  pub status: u32,
  pub enable: u32,
  pub hide_mobile: u32,
  pub english_name: String,
  pub telephone: String,
  pub order: Vec<u32>,
  pub main_department: Option<u32>,
  pub qr_code: String,
  pub alias: String,
  pub is_leader_in_dept: Vec<u32>,
  pub thumb_avatar: String,
  pub biz_mail: Option<String>,
  #[serde(rename = "userid")]
  pub user_id: String,
  pub extattr: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TagsResp {
  #[serde(rename = "errcode")]
  pub code: Option<i32>,
  #[serde(rename = "errmsg")]
  pub msg: Option<String>,
  #[serde(rename = "taglist")]
  pub tags: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
  #[serde(rename = "tagid")]
  pub id: u32,
  #[serde(rename = "tagname")]
  pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TagMembersResp {
  #[serde(rename = "errcode")]
  pub code: Option<i32>,
  #[serde(rename = "errmsg")]
  pub msg: Option<String>,
  #[serde(rename = "userlist")]
  pub members: Vec<TagMember>,
  #[serde(rename = "partylist")]
  department_list: Vec<u32>,
  #[serde(rename = "tagname")]
  tag_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TagMember {
  #[serde(rename = "userid")]
  id: String,
  name: String,
}
