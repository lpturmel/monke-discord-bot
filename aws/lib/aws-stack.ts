import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { HttpApi, HttpMethod } from "@aws-cdk/aws-apigatewayv2-alpha";
import { HttpLambdaIntegration } from "@aws-cdk/aws-apigatewayv2-integrations-alpha";
import * as lambda from "aws-cdk-lib/aws-lambda";
import { join } from 'path';


// import * as sqs from 'aws-cdk-lib/aws-sqs';

export class AwsStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const prefix = "monke-discord-bot";
            const webhookHandler = new lambda.Function(
            this,
            `${prefix}-api-handler`,
            {
                functionName: `${prefix}-backend-handler`,
                runtime: lambda.Runtime.PROVIDED_AL2,
                handler: "not.required",
                memorySize: 1024,
                code: lambda.Code.fromAsset(join(__dirname, "../../target/lambda/monke-bot/bootstrap.zip")),
                architecture: lambda.Architecture.ARM_64,
                environment: {
                    RUST_BACKTRACE: "1",
                    DISCORD_APP_ID: "1101587526097047563",
                    DISCORD_GUILD_ID: process.env.DISCORD_GUILD_ID!,
                    DISCORD_BOT_TOKEN: process.env.DISCORD_BOT_TOKEN!,
                    RIOT_API_KEY: process.env.RIOT_API_KEY!,
                },
                description:
                    "Monke Discord server Slash command integration",
            },
        );


    const api = new HttpApi(this, `${prefix}-http-api`, {
            apiName: `${prefix}-backend-api`,
            description:
                "Monke Discord server Slash command integration",
            defaultIntegration: new HttpLambdaIntegration(
                "DwarfInvasionBotApiIntegration",
                webhookHandler,
            ),
        });

    api.addRoutes({
        path: '/integration',
        methods: [ HttpMethod.POST ],
        integration: new HttpLambdaIntegration("DiscordIntegration", webhookHandler),
    });
  }
}
