#!/bin/sh

aws iam list-roles --query "Roles[?starts_with(RoleName, 'l3-')].RoleName" | jq -r '.[]' | xargs -I{} aws iam detach-role-policy --role-name {} --policy-arn arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole
aws iam list-roles --query "Roles[?starts_with(RoleName, 'l3-')].RoleName" | jq -r '.[]' | xargs -I{} aws iam delete-role --role-name {}

aws lambda list-functions --query "Functions[?starts_with(FunctionName, 'l3-')].FunctionName" | jq -r '.[]' | xargs -I{} aws lambda delete-function --function-name {}

aws apigatewayv2 get-apis --query "Items[?starts_with(Name, 'l3-')].ApiId" | jq -r '.[]' | xargs -I{} aws apigatewayv2 delete-api --api-id {}
