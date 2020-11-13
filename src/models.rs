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
    pub fn serialize(&self) -> HashMap<String, rusoto_dynamodb::AttributeValue> {
        let mut m = HashMap::new();
        m.insert(
            "user_id".to_string(),
            AttributeValue {
                s: Some(self.id.to_string()),
                ..Default::default()
            },
        );
        m.insert(
            "birth_day".to_string(),
            AttributeValue {
                n: Some(self.birth.day.to_string()),
                ..Default::default()
            },
        );
        m.insert(
            "birth_month".to_string(),
            AttributeValue {
                n: Some(self.birth.month.to_string()),
                ..Default::default()
            },
        );
        m.insert(
            "birth_year".to_string(),
            AttributeValue {
                n: Some(self.birth.year.to_string()),
                ..Default::default()
            },
        );

        m
    }
}
