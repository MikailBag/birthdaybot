mod bot;
mod db;
mod models;

use anyhow::Context;
use futures_util::StreamExt;
use rusoto_ssm::Ssm;
use teloxide::prelude::Request;

async fn get_token() -> anyhow::Result<String> {
    match tokio::fs::read_to_string("./tg-token").await {
        Ok(s) => Ok(s),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            println!("Retrieving token from SSM");
            let ssm = rusoto_ssm::SsmClient::new(rusoto_core::Region::UsEast1);
            let prefix = std::env::var("SSM_PREFIX").context("SSM_PREFIX missing")?;
            if !prefix.starts_with('/') {
                anyhow::bail!("SSM_PREFIX does not start with slash");
            }
            if prefix.ends_with('/') {
                anyhow::bail!("SSM_PREFIX must not end with slash");
            }
            let req = rusoto_ssm::GetParameterRequest {
                name: format!("{}/tg-token", prefix),
                with_decryption: Some(false),
            };
            let resp = ssm
                .get_parameter(req)
                .await
                .context("failed to fetch a tg-token parameter")?;
            resp.parameter
                .context("parameter info missing")?
                .value
                .context("parameter value missing")
        }
        Err(e) => Err(anyhow::Error::new(e).context("failed to read token")),
    }
}

#[derive(serde::Deserialize)]
struct HttpInfo {
    path: String,
}

#[derive(serde::Deserialize)]
struct RequestContext {
    http: HttpInfo,
}

#[derive(serde::Deserialize)]
struct LambdaHttpRequest {
    #[serde(rename = "requestContext")]
    request_context: RequestContext,
    body: Option<String>,
    #[serde(rename = "isBase64Encoded")]
    is_base64_encoded: bool,
}

#[derive(serde::Deserialize)]
struct LambdaEventRequest {
    #[serde(rename = "greet")]
    _greet: serde_json::Value,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum LambdaRequest {
    Event(LambdaEventRequest),
    Http(LambdaHttpRequest),
}

impl LambdaRequest {
    fn unify(self) -> LambdaHttpRequest {
        match self {
            LambdaRequest::Http(h) => h,
            LambdaRequest::Event(_ev) => LambdaHttpRequest {
                is_base64_encoded: false,
                body: None,
                request_context: RequestContext {
                    http: HttpInfo {
                        path: "$/greet".to_string(),
                    },
                },
            },
        }
    }
}

async fn handler_inner(
    bot: teloxide::Bot,
    db: db::Db,
    path: &str,
    body: &[u8],
) -> anyhow::Result<(i32, String)> {
    let secret = std::env::var("SECRET").context("SECRET missing")?;

    let path_install_webhook = format!("/install-webhook/{}", secret);
    let path_webhook = format!("/hook/{}", secret);
    let path_hi = "/";
    let path_greet = "$/greet";

    if path == path_hi {
        return Ok((200, "hi there".to_string()));
    }

    if path == path_install_webhook {
        if let Err(err) = bot
            .set_webhook(format!(
                "https://r3wfwomtd1.execute-api.us-east-1.amazonaws.com{}",
                path_webhook
            ))
            .send()
            .await
        {
            return Ok((500, format!("Failed to install webhook: {:#}", err)));
        }
        return Ok((200, "webhook installed".to_string()));
    }
    if path == path_greet {
        bot::greet(bot, db).await?;
        return Ok((200, "greet done".to_string()));
    }
    if path != path_webhook {
        return Ok((404, "unknown action".to_string()));
    }
    if body.is_empty() {
        return Ok((400, "Empty body. Did you POST?".to_string()));
    }
    let update = match serde_json::from_slice(body) {
        Ok(u) => u,
        Err(_) => return Ok((400, "invalid json".to_string())),
    };
    match bot::on_message(bot, db, update).await {
        Ok(()) => Ok((200, "OK".to_string())),
        Err(err) => {
            tracing::error!(err=?format_args!("{:#}", err), "failed to process request");
            Ok((500, "Failed to process this request".to_string()))
        }
    }
}

async fn handler(
    ev: LambdaRequest,
    bot: teloxide::Bot,
    db: db::Db,
) -> anyhow::Result<serde_json::Value> {
    let mut ev = ev.unify();
    tracing::info!(path = %ev.request_context.http.path, "processing request");
    let body = match std::mem::take(&mut ev.body) {
        Some(s) => {
            if ev.is_base64_encoded {
                base64::decode(&s)?
            } else {
                s.into_bytes()
            }
        }
        None => Vec::new(),
    };
    let (status, body) = handler_inner(bot, db, &ev.request_context.http.path, &body)
        .await
        .map_err(|e| {
            tracing::error!("Error: {err}", err=format_args!("{:#}", e));
            e
        })?;
    Ok(serde_json::json!({
        "isBase64Encoded": false,
        "statusCode": status,
        "body": body
    }))
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let token = get_token().await?;
    println!("Using token: {}...", &token[..6]);
    let bot = teloxide::Bot::builder().token(token).build();

    /*if std::env::var("DBG").is_ok() {
        println!("debugging");

        let client = reqwest::Client::builder().build()?;
        let res = client.post("https://api.telegram.org/kek").send().await;
        dbg!(res);

        let res = bot.get_me().send().await;
        dbg!(res);

        return Ok(());
    }*/
    let db = db::Db::connect()
        .await
        .context("failed to create db client")?;

    if std::env::var("LOCAL").is_ok() {
        println!("local launch");
        let updates = teloxide::dispatching::update_listeners::polling_default(bot.clone());
        tokio::pin!(updates);
        while let Some(update) = updates.next().await {
            bot::on_message(
                bot.clone(),
                db.clone(),
                update.context("failed to receive an update")?,
            )
            .await?;
        }
        return Ok(());
    }
    lambda::run(lambda::handler_fn(move |upd, _cx| {
        handler(upd, bot.clone(), db.clone())
    }))
    .await
    .map_err(|err| anyhow::anyhow!(err))?;
    Ok(())
}
