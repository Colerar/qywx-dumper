use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Context, Result};
use log::debug;
use reqwest::{Client, Proxy, Url};

use crate::api::data::{
  AgentListResp, DepartmentMembersResp, DepartmentResp, GetTokenResp, Success, TagMembersResp,
  TagsResp,
};

use self::data::AgentDetail;

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 12_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6 Safari/605.1.15";

mod data;

#[derive(Clone)]
pub struct WxClient {
  client: Client,
  pub token: Arc<RwLock<Option<String>>>,
}

impl WxClient {
  pub async fn new(
    proxy: Option<Url>,
    auth_user: Option<String>,
    auth_pwd: Option<String>,
    user_agent: Option<String>,
  ) -> Result<WxClient> {
    let mut builder = Client::builder().pool_max_idle_per_host(0);
    if let Some(proxy) = proxy {
      let mut proxy = Proxy::all(proxy)?;
      if auth_user.is_some() && auth_pwd.is_some() {
        proxy = proxy.basic_auth(&*auth_user.unwrap(), &*auth_pwd.unwrap())
      }
      builder = builder.proxy(proxy)
    }
    builder = builder.user_agent(user_agent.unwrap_or_else(|| DEFAULT_USER_AGENT.to_string()));
    let reqwest = builder.build().context("Failed to create reqwest client")?;
    Ok(WxClient {
      client: reqwest,
      token: Arc::new(RwLock::new(None)),
    })
  }

  fn token(&self) -> Result<String> {
    let result = self.token.read().unwrap();
    match result.clone() {
      Some(some) => Ok(some),
      None => {
        debug!("Token: {:?}", &self.token);
        Err(anyhow!("Token is None, not login"))
      }
    }
  }

  fn client(&self) -> Client {
    self.client.clone()
  }

  pub async fn login(&self, corp_id: &str, secret: &str) -> Result<GetTokenResp> {
    let resp = self
      .client()
      .get("https://qyapi.weixin.qq.com/cgi-bin/gettoken")
      .query(&[("corpid", corp_id), ("corpsecret", secret)])
      .send()
      .await
      .context("Failed to to get token")?;
    let resp = resp
      .json::<GetTokenResp>()
      .await
      .context("Failed to deserialize GetTokenResp")?;

    if resp.is_success() && resp.access_token.is_some() {
      let mut token = self.token.write().unwrap();
      *token = Some(resp.access_token.clone().unwrap());
    } else {
      return Err(anyhow!("Failed to get token: {:#?}", resp));
    }

    debug!("login: {resp:?}");

    Ok(resp)
  }

  /// get apps basic info
  pub async fn get_agent_list(&self) -> Result<AgentListResp> {
    self
      .client()
      .get("https://qyapi.weixin.qq.com/cgi-bin/agent/list")
      .query(&[("access_token", self.token()?)])
      .send()
      .await
      .context("Failed to get AgentListResp")?
      .json::<AgentListResp>()
      .await
      .context("Failed to deserialize AgentListResp")
  }

  pub async fn get_all_departments(&self) -> Result<DepartmentResp> {
    self.get_departments(None).await
  }

  /// get departments
  /// ## params
  /// - id: [None] for getting all departments with access
  pub async fn get_departments(&self, _id: Option<u32>) -> Result<DepartmentResp> {
    self
      .client()
      .get("https://qyapi.weixin.qq.com/cgi-bin/department/list")
      .query(&[("access_token", self.token()?)])
      .send()
      .await
      .context("Failed to get DepartmentResp")?
      .json::<DepartmentResp>()
      .await
      .context("Failed to deserialize DepartmentResp")
  }

  /// get department members
  pub async fn get_department_members(
    &self,
    id: u32,
    fetch_child: bool,
  ) -> Result<DepartmentMembersResp> {
    self
      .client()
      .get("https://qyapi.weixin.qq.com/cgi-bin/user/list")
      .query(&[
        ("access_token", self.token()?),
        ("department_id", id.to_string()),
        (
          "fetch_child",
          match fetch_child {
            true => "1".to_string(),
            false => "0".to_string(),
          },
        ),
      ])
      .send()
      .await
      .context("Failed to get DepartmentMembersResp")?
      .json::<DepartmentMembersResp>()
      .await
      .context("Failed to deserialize DepartmentMembersResp")
  }

  pub async fn get_tags(&self) -> Result<TagsResp> {
    self
      .client()
      .get("https://qyapi.weixin.qq.com/cgi-bin/tag/list")
      .query(&[("access_token", self.token()?)])
      .send()
      .await
      .context("Failed to get TagsResp")?
      .json::<TagsResp>()
      .await
      .context("Failed to deserialize TagsResp")
  }

  pub async fn get_tag_members(&self, tag_id: u32) -> Result<TagMembersResp> {
    self
      .client()
      .get("https://qyapi.weixin.qq.com/cgi-bin/tag/get")
      .query(&[
        ("access_token", self.token()?),
        ("tagid", tag_id.to_string()),
      ])
      .send()
      .await
      .context("Failed to get TagMembersResp")?
      .json::<TagMembersResp>()
      .await
      .context("Failed to deserialize TagMembersResp")
  }

  pub async fn get_agent_detail(&self, agent_id: u32) -> Result<AgentDetail> {
    self
      .client()
      .get("https://qyapi.weixin.qq.com/cgi-bin/agent/get")
      .query(&[
        ("access_token", self.token()?),
        ("agentid", agent_id.to_string()),
      ])
      .send()
      .await
      .context("Failed to get AgentDetail")?
      .json::<AgentDetail>()
      .await
      .context("Failed to deserialize AgentDetail")
  }
}

#[cfg(test)]
mod tests {

  use std::sync::{Arc, RwLock};

  use anyhow::{Context, Result};

  use lazy_static::lazy_static;
  use log::debug;

  use crate::api::WxClient;
  use crate::init_logger;

  lazy_static! {
    static ref TOKEN: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
  }

  async fn client() -> Result<WxClient> {
    init_logger("debug");
    let cli = WxClient::new(None, None, None, None).await?;
    let option = { TOKEN.read().unwrap().clone() };
    match option {
      None => {
        let resp = cli
          .login(
            std::env::var("WX_CORP_ID")
              .context("No env WX_CORP_ID")?
              .as_str(),
            std::env::var("WX_CORP_SECRET")
              .context("No env WX_CORP_SECRET")?
              .as_str(),
          )
          .await?;
        if let Some(ac) = resp.access_token {
          let mut token = TOKEN.write().unwrap();
          *token = Some(ac);
        }
      }
      some @ Some(_) => *cli.token.write().unwrap() = some,
    };
    Ok(cli)
  }

  #[tokio::test]
  async fn get_agent_list_test() -> Result<()> {
    let agent = client().await?.get_agent_list().await?;
    debug!("{agent:?}");
    Ok(())
  }

  #[tokio::test]
  async fn get_departments_test() -> Result<()> {
    let resp = client().await?.get_departments(None).await?;
    debug!("{resp:?}");
    Ok(())
  }

  #[tokio::test]
  async fn get_department_members_test() -> Result<()> {
    let cli = client().await?;
    let departments = cli.get_departments(None).await?;
    let department = &departments.departments[0];
    let members = cli.get_department_members(department.id, true).await?;
    dbg!(members);
    Ok(())
  }

  #[tokio::test]
  async fn get_tags_test() -> Result<()> {
    let resp = client().await?.get_tags().await?;
    debug!("{resp:?}");
    Ok(())
  }

  #[tokio::test]
  async fn get_tag_members_test() -> Result<()> {
    let cli = client().await?;
    let tags = cli.get_tags().await?;
    debug!("{tags:?}");
    for x in tags.tags.into_iter().take(20) {
      let members = cli.get_tag_members(x.id).await;
      debug!("{members:?}");
    }
    Ok(())
  }

  #[tokio::test]
  async fn get_agent_list() -> Result<()> {
    let cli = client().await?;
    let agents = cli.get_agent_list().await?;
    debug!("{agents:?}");
    for x in agents.agent_list.into_iter().take(1) {
      let members = cli.get_agent_detail(x.id).await;
      debug!("{members:?}");
    }
    Ok(())
  }
}
