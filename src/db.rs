use anyhow::Context;
use rusoto_dynamodb::DynamoDb;

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

    pub async fn select(&self)

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
