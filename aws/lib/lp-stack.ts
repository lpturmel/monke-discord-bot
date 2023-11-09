import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as events from "aws-cdk-lib/aws-events";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as targets from "aws-cdk-lib/aws-events-targets";
import { Schedule, ScheduleExpression } from "@aws-cdk/aws-scheduler-alpha";
import { LambdaInvoke } from "@aws-cdk/aws-scheduler-targets-alpha";
import { join } from 'path';
import { AttributeType, BillingMode, Table } from 'aws-cdk-lib/aws-dynamodb';
import { TimeZone } from 'aws-cdk-lib';

interface LeaguePointServiceProps extends cdk.StackProps {
    webhook_handler: lambda.Function;
}
export class LeaguePointService extends cdk.Stack {
    constructor(scope: Construct, id: string, props?: LeaguePointServiceProps) {
        super(scope, id, props);

        const prefix = "monke-league-point-service";
        const table = new Table(this, `${prefix}-table`, {
            tableName: `${prefix}-table`,
            partitionKey: {
                name: "id",
                type: AttributeType.STRING,
            },
            sortKey: {
                name: "sk",
                type: AttributeType.STRING,
            },
            billingMode: BillingMode.PAY_PER_REQUEST,
        });
        const lpHandler = new lambda.Function(
            this,
            `${prefix}-api-handler`,
            {
                functionName: `${prefix}-backend-handler`,
                runtime: lambda.Runtime.PROVIDED_AL2,
                handler: "not.required",
                memorySize: 1024,
                code: lambda.Code.fromAsset(join(__dirname, "../../target/lambda/lp-serv/bootstrap.zip")),
                architecture: lambda.Architecture.ARM_64,
                environment: {
                    DISCORD_APP_ID: "1101587526097047563",
                    DISCORD_GUILD_ID: process.env.DISCORD_GUILD_ID!,
                    DISCORD_BOT_TOKEN: process.env.DISCORD_BOT_TOKEN!,
                    RIOT_API_KEY: process.env.RIOT_API_KEY!,
                    TFT_RIOT_API_KEY: process.env.TFT_RIOT_API_KEY!,
                    RUST_BACKTRACE: "1",
                    LP_DB_TABLE_NAME: table.tableName,
                },
                description:
                    "Monke League Point tracker Service",
            },
        );

        table.grantReadWriteData(lpHandler);
        table.grantReadWriteData(props?.webhook_handler!);

        new Schedule(this, `${prefix}-new--schedule`, {
            schedule: ScheduleExpression.cron({
                minute: '0',
                hour: '0',
                timeZone: TimeZone.AMERICA_NEW_YORK,
            }),
            target: new LambdaInvoke(lpHandler, {}),
            description: 'This is a schedule to pull the League Points of users every day at midnight',
        });
        new Schedule(this, `${prefix}-new-schedule`, {
            schedule: ScheduleExpression.cron({
                minute: '59',
                hour: '23',
                timeZone: TimeZone.AMERICA_NEW_YORK,
            }),
            target: new LambdaInvoke(lpHandler, {}),
            description: 'This is a schedule to pull the League Points of users every day before midnight',
        });
    }
}

