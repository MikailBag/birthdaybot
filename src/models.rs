use anyhow::Context;
use rusoto_dynamodb::AttributeValue;
use std::collections::HashMap;

pub struct Date {
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

pub struct User {
    pub id: i32,
    pub birth: Date,
    pub last_greeted_timestamp: u64,
    pub chat_id: i64,
}

impl User {
    pub fn serialize(&self) -> HashMap<String, AttributeValue> {
        let mut m = HashMap::new();
        m.insert(
            "UserId".to_string(),
            AttributeValue {
                s: Some(self.id.to_string()),
                ..Default::default()
            },
        );
        m.insert(
            "BirthDay".to_string(),
            AttributeValue {
                s: Some(self.birth.day.to_string()),
                ..Default::default()
            },
        );
        m.insert(
            "BirthMonth".to_string(),
            AttributeValue {
                s: Some(self.birth.month.to_string()),
                ..Default::default()
            },
        );
        m.insert(
            "BirthYear".to_string(),
            AttributeValue {
                s: Some(self.birth.year.to_string()),
                ..Default::default()
            },
        );
        m.insert(
            "LastTs".to_string(),
            AttributeValue {
                s: Some(self.last_greeted_timestamp.to_string()),
                ..Default::default()
            },
        );
        m.insert(
            "ChatId".to_string(),
            AttributeValue {
                s: Some(self.chat_id.to_string()),
                ..Default::default()
            },
        );

        m
    }

    pub fn deserialize(m: HashMap<String, AttributeValue>) -> anyhow::Result<Self> {
        let user_id = m
            .get("UserId")
            .and_then(|x| x.s.as_ref())
            .context("UserId missing")?
            .parse()?;
        let birth_day = m
            .get("BirthDay")
            .and_then(|x| x.s.as_ref())
            .context("BirthDay missing")?
            .parse()?;
        let birth_month = m
            .get("BirthMonth")
            .and_then(|x| x.s.as_ref())
            .context("BirthMonth missing")?
            .parse()?;
        let birth_year = m
            .get("BirthYear")
            .and_then(|x| x.s.as_ref())
            .context("BirthYear missing")?
            .parse()?;
        let last_ts = m
            .get("LastTs")
            .and_then(|x| x.s.as_ref())
            .context("LastTs missing")?
            .parse()?;
        let chat_id = m
            .get("ChatId")
            .and_then(|x| x.s.as_ref())
            .context("ChatId missing")?
            .parse()?;
        Ok(User {
            id: user_id,
            chat_id,
            last_greeted_timestamp: last_ts,
            birth: Date {
                day: birth_day,
                month: birth_month,
                year: birth_year,
            },
        })
    }
}
