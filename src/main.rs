use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{env, fs};

use anyhow::{Context, Result};
use clap::{Parser, ValueHint};
use clap_verbosity_flag::Verbosity;
use itertools::Itertools;
use log::{debug, error, info, warn};
use reqwest::Url;
use tokio::spawn;
use tokio::time::sleep;

use crate::api::WxClient;
use crate::util::ReplaceSpecial;

mod api;
mod util;

#[derive(Parser, Debug, Clone)]
#[clap(name = "qywx-dumper", bin_name = "qywx-dumper", version, about, long_about = None)]
struct Cli {
  /// Output directory
  #[arg(short = 'O', long, value_parser, value_name = "DIR")]
  #[arg(value_hint = ValueHint::DirPath, default_value = "output")]
  output: PathBuf,
  /// Corporation ID, every enterprise has one
  #[arg(short = 'i', long)]
  #[arg(env = "WX_CORP_ID", value_parser, value_name = "ID")]
  corp_id: Option<String>,
  /// Corporation Secret, every app has one
  #[arg(short = 's', long)]
  #[arg(env = "WX_CORP_SECRET", value_parser, value_name = "SECRET")]
  corp_secret: Option<String>,
  /// Token, requires: (ID and Secret) or TOKEN
  #[arg(short = 't', long)]
  #[arg(env = "WX_CORP_TOKEN", value_parser, value_name = "SECRET")]
  corp_token: Option<String>,
  /// Custom user agent, optional
  #[arg(short = 'u', long)]
  user_agent: Option<String>,
  /// Sending request through a proxy, http, https, socks5 are supported
  #[arg(short = 'p', long, value_parser, value_name = "URL")]
  proxy: Option<Url>,
  /// Proxy username, optional
  #[arg(long, value_parser, alias = "user", value_name = "USER")]
  proxy_user: Option<String>,
  /// Proxy password, optional
  #[arg(long, value_parser, alias = "password", value_name = "PWD")]
  proxy_password: Option<String>,
  /// always overwrite files
  #[arg(short = 'y', long, value_parser, alias = "yes")]
  overwrite: bool,
  /// Fetch departments members recursively
  #[arg(short = 'r', long, value_parser, default_value_t = false)]
  recursive: bool,
  /// Delay for batch requests, in ms
  #[arg(short = 'd', long, value_parser, default_value_t = 200)]
  delay: u64,
  #[clap(flatten)]
  verbose: Verbosity<DefaultLevel>,
}

#[tokio::main]
async fn main() -> Result<()> {
  let args: Cli = Cli::parse();
  pretty_env_logger::env_logger::Builder::new()
    .filter_level(args.verbose.log_level_filter())
    .init();
  debug!("Args: {args:?}");

  if (args.corp_id.is_none() && args.corp_secret.is_none()) && args.corp_token.is_none() {
    error!("For login, you must provide: (ID and Secret) or Token.");
    exit(1);
  }

  if args.output.exists() {
    if args.overwrite {
      warn!("Overwriting files according to --overwrite option...");
      if args.output.is_file() {
        fs::remove_file(&args.output).context("Failed to delete file")?;
      } else if args.output.is_dir() {
        fs::remove_dir_all(&args.output).context("Failed to delete directory")?;
      }
    } else {
      error!(
        "Output path '{}', is already exists, append -y, --yes or --overwrite to overwrite it.",
        args.output.to_string_lossy()
      );
      exit(1);
    }
  }

  fs::create_dir_all(&args.output).context("Failed to create folder 'output'")?;

  env::set_current_dir(&args.output).context("Failed to set current dir")?;

  let wx = WxClient::new(
    args.proxy,
    args.proxy_user,
    args.proxy_password,
    args.user_agent,
  )
  .await;

  let wx = match wx {
    Ok(wx) => wx,
    Err(err) => {
      error!("Failed to create WeChat client: {:?}", err);
      exit(1);
    }
  };

  if args.corp_id.is_some() && args.corp_secret.is_some() {
    if let Err(err) = wx
      .login(&*args.corp_id.unwrap(), &*args.corp_secret.unwrap())
      .await
    {
      error!("Failed to login with provided id and secret: {:?}", err);
      exit(1);
    };
  } else if args.corp_token.is_some() {
    let mut token = wx.token.write().unwrap();
    *token = args.corp_token;
  } else {
    error!("For login, you must provide: (ID and Secret) or Token.");
    exit(1);
  }

  info!("Get token successfully");

  let agent_job = || {
    let wx = wx.clone();
    async move {
      let agents = wx
        .get_agent_list()
        .await
        .context("Failed to get agent list")?;
      let agent_to_print = agents
        .agent_list
        .iter()
        .map(|i| format!("{} - {}", i.id, i.name))
        .join(", ");
      info!("Agents: {agent_to_print}");
      let file = File::create("agents.json").context("Failed to create agents.json")?;
      let mut buf_writer = BufWriter::new(file);
      buf_writer
        .write(&*serde_json::to_vec_pretty(&agents).context("Failed to serialize")?)
        .context("Failed to write json")?;
      let result: Result<()> = Ok(());
      result
    }
  };

  let department_job = || {
    let wx = wx.clone();
    async move {
      let resp = wx
        .get_all_departments()
        .await
        .context("Failed to get departments list")?;
      info!("Total {} departments to query", resp.departments.len());
      let file = File::create("departments.json").context("Failed to create departments.json")?;
      let mut buf_writer = BufWriter::new(file);
      buf_writer
        .write(&*serde_json::to_vec_pretty(&resp).context("Failed to serialize")?)
        .context("Failed to write json")?;

      fs::create_dir_all("departments")?;

      let mut vec = Vec::new();
      for x in resp.departments {
        let recursive = args.recursive;
        let wx = wx.clone();
        let handle = spawn(async move {
          let resp = match wx.get_department_members(x.id, recursive).await {
            Ok(resp) => resp,
            Err(err) => {
              error!(
                "Failed to get the members of department: {} - {}: {:?}",
                x.id, x.name, err
              );
              return;
            }
          };

          let path = PathBuf::from(format!(
            "departments/{}",
            format!("members-{}-{}.json", x.id, x.name).replace_special_char()
          ));
          let file = match File::create(&path) {
            Ok(file) => file,
            Err(err) => {
              error!("Failed to create {}: {err:?}", path.to_string_lossy());
              return;
            }
          };
          let json = match serde_json::to_vec_pretty(&resp).context("Failed to serialize") {
            Ok(json) => json,
            Err(err) => {
              error!("Failed to serialize json: {err:?}");
              return;
            }
          };
          let mut buf_writer = BufWriter::new(file);
          match buf_writer.write(&*json) {
            Ok(_) => info!(
              "Successfully save department members to {}, total {}",
              path.to_string_lossy(),
              resp.members.len()
            ),
            Err(err) => error!(
              "Failed to save department members to {}: {err:?}",
              path.to_string_lossy()
            ),
          };
        });
        vec.push(handle);
        sleep(Duration::from_millis(args.delay)).await;
      }
      for x in vec {
        x.await?;
      }
      let result: Result<()> = Ok(());
      result
    }
  };

  let tag_job = || {
    let wx = wx.clone();
    async move {
      let resp = wx.get_tags().await.context("Failed to get tags list")?;
      info!("Total {} tags to query", resp.tags.len());
      let file = File::create("tags.json").context("Failed to create tags.json")?;
      let mut buf_writer = BufWriter::new(file);
      buf_writer
        .write(&*serde_json::to_vec_pretty(&resp).context("Failed to serialize")?)
        .context("Failed to write json")?;

      fs::create_dir_all("tags")?;

      let txt = Arc::new(RwLock::new(String::from("These tags has no member:\n")));

      let mut vec = Vec::new();
      for x in resp.tags {
        let wx = wx.clone();
        let txt = txt.clone();
        let handle = spawn(async move {
          let resp = match wx.get_tag_members(x.id).await {
            Ok(resp) => resp,
            Err(err) => {
              error!(
                "Failed to get the members of tag: {} - {}: {:?}",
                x.id, x.name, err
              );
              return;
            }
          };

          if resp.members.is_empty() && resp.code == Some(0) {
            let mut txt = txt.write().unwrap();
            txt.push_str(&*format!("{} - {}\n", x.id, x.name));
            return;
          }

          let path = PathBuf::from(format!(
            "tags/{}",
            format!("members-{}-{}.json", x.id, x.name).replace_special_char()
          ));
          let file = match File::create(&path) {
            Ok(file) => file,
            Err(err) => {
              error!("Failed to create {}: {err:?}", path.to_string_lossy());
              return;
            }
          };
          let json = match serde_json::to_vec_pretty(&resp).context("Failed to serialize") {
            Ok(json) => json,
            Err(err) => {
              error!("Failed to serialize json: {err:?}");
              return;
            }
          };
          let mut buf_writer = BufWriter::new(file);
          match buf_writer.write(&*json) {
            Ok(_) => info!(
              "Successfully save tag members to {}, total {}",
              path.to_string_lossy(),
              resp.members.len()
            ),
            Err(err) => error!(
              "Failed to save tag members to {}: {err:?}",
              path.to_string_lossy()
            ),
          };
        });
        vec.push(handle);
        sleep(Duration::from_millis(args.delay)).await;
      }
      for x in vec {
        x.await?;
      }

      let txt_file = File::create("tags/_empty.txt").context("Failed to create tags/_empty.txt")?;
      let mut buf_writer = BufWriter::new(txt_file);
      buf_writer.write_all(txt.read().unwrap().as_bytes())?;

      let result: Result<()> = Ok(());
      result
    }
  };

  let agent_job = spawn(agent_job());
  let department_job = spawn(department_job());
  let tag_job = spawn(tag_job());

  if let Err(err) = agent_job.await? {
    error!("Fetch agent list job failed: {err:?}");
  }

  if let Err(err) = department_job.await? {
    error!("Fetch department members job failed: {err:?}");
  }

  if let Err(err) = tag_job.await? {
    error!("Fetch tag members job failed: {err:?}");
  }
  Ok(())
}

#[cfg(test)]
fn init_logger(level: &str) {
  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", level);
  }
  pretty_env_logger::init();
}

type DefaultLevel = clap_verbosity_flag::InfoLevel;
