#!/usr/bin/env bash
set -e

cargo build --release --target x86_64-unknown-linux-musl
cp ./target/x86_64-unknown-linux-musl/release/birthdaybot ./target/bootstrap

echo "creating package"
( cd target && zip lambda.zip bootstrap )

echo "uploading package"
aws lambda update-function-code --function-name birthday_bot --zip-file fileb://./target/lambda.zip
aws lambda publish-version --function birthday_bot
