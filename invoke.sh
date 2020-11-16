#!/usr/bin/env bash

aws lambda invoke --function-name birthday_bot --payload '{"greet": {}}' --cli-binary-format \
  raw-in-base64-out --log-type Tail  ./response.json --query LogResult --output text |  base64 -d 