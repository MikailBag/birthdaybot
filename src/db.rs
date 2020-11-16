use anyhow::Context;
use rusoto_dynamodb::{AttributeValue, DynamoDb};

#[derive(Clone)]
pub struct Db {
    ddb: rusoto_dynamodb::DynamoDbClient,
    table: String,
}

impl Db {
    pub async fn connect() -> anyhow::Result<Db> {
        let table = std::env::var("DDB_TABLE")
            .ok()
            .context("DDB_TABLE missing")?;
        let ddb = rusoto_dynamodb::DynamoDbClient::new(rusoto_core::Region::UsEast1);
        Ok(Db { ddb, table })
    }

    pub async fn select(
        &self,
        day: &str,
        month: &str,
        ts: &str,
    ) -> anyhow::Result<Vec<crate::models::User>> {
        let mut values = std::collections::HashMap::new();
        values.insert(
            ":BirthDay".to_string(),
            AttributeValue {
                s: Some(day.to_string()),
                ..Default::default()
            },
        );
        values.insert(
            ":BirthMonth".to_string(),
            AttributeValue {
                s: Some(month.to_string()),
                ..Default::default()
            },
        );
        values.insert(
            ":LastTs".to_string(),
            AttributeValue {
                s: Some(ts.to_string()),
                ..Default::default()
            },
        );
        let req = rusoto_dynamodb::ScanInput {
            filter_expression: Some(
                "BirthDay = :BirthDay and BirthMonth = :BirthMonth and LastTs <= :LastTs"
                    .to_string(),
            ),
            table_name: self.table.to_string(),
            expression_attribute_values: Some(values),
            ..Default::default()
        };
        let resp = self.ddb.scan(req).await?;
        tracing::info!(
            returned_lines = resp.items.as_ref().map_or(0, Vec::len),
            "Scan done"
        );
        let items = match resp.items {
            Some(its) => its,
            None => return Ok(Vec::new()),
        };
        let mut res = Vec::new();
        for it in items {
            res.push(crate::models::User::deserialize(it).context("failed to parse value")?);
        }
        Ok(res)
    }

    pub async fn put(&self, user: crate::models::User) -> anyhow::Result<()> {
        let req = rusoto_dynamodb::PutItemInput {
            table_name: self.table.clone(),
            condition_expression: None,
            item: user.serialize(),
            expression_attribute_names: None,
            expression_attribute_values: None,
            conditional_operator: None,
            expected: None,
            return_consumed_capacity: None,
            return_item_collection_metrics: None,
            return_values: None,
        };
        self.ddb.put_item(req).await?;
        Ok(())
    }
}
