use teloxide::utils::command::BotCommand;
use teloxide::{prelude::Request, types::UpdateKind};

#[derive(Debug, BotCommand)]
#[command(rename = "lowercase", parse_with = "split")]
enum Command {
    #[command(description = "get some help")]
    Help,
    #[command(description = "/register DD.MM.YYYY (do not use trailing zeroes)")]
    Register(String),
}

fn parse_date(s: &str) -> Option<crate::models::Date> {
    let items = s.split('.').collect::<Vec<_>>();
    if items.len() != 3 {
        return None;
    }
    let day = items[0].parse().ok()?;
    let month = items[1].parse().ok()?;
    let year = items[2].parse().ok()?;
    Some(crate::models::Date { day, month, year })
}

pub(crate) async fn on_message(
    bot: teloxide::Bot,
    db: crate::db::Db,
    update: teloxide::prelude::Update,
) -> anyhow::Result<()> {
    let msg = match update.kind {
        UpdateKind::Message(m) => m,
        _ => return Ok(()),
    };

    let me = bot.get_me().send().await?.user.username.unwrap_or_default();

    if let Some(text) = msg.text() {
        let cmd = match Command::parse(text, me) {
            Ok(c) => c,
            Err(e) => {
                bot.send_message(msg.chat.id, format!("I did not understand you: {}", e))
                    .send()
                    .await?;
                return Ok(());
            }
        };
        dbg!(&cmd);
        match cmd {
            Command::Help => {
                bot.send_message(msg.chat.id, Command::descriptions())
                    .send()
                    .await?;
            }
            Command::Register(birth) => {
                let user_id = match msg.from() {
                    Some(u) => u.id,
                    _ => return Ok(()),
                };
                let date = parse_date(&birth);
                let date = match date {
                    Some(d) => d,
                    None => {
                        bot.send_message(msg.chat.id, "I couldn't parse your birth date")
                            .send()
                            .await?;
                        return Ok(());
                    }
                };

                let user = crate::models::User {
                    id: user_id,
                    birth: date,
                    last_greeted_timestamp: 0,
                    chat_id: msg.chat.id,
                };

                db.put(user).await?;

                bot.send_message(msg.chat.id, "done").send().await?;
            }
        }
    }

    Ok(())
}

pub(crate) async fn greet(bot: teloxide::Bot, db: crate::db::Db) -> anyhow::Result<()> {
    
}
