#!/usr/bin/env node
import * as cdk from 'aws-cdk-lib';
import { BotStack } from '../lib/bot-stack';
import { config } from 'dotenv';
import { LeaguePointService } from '../lib/lp-stack';
config();

const app = new cdk.App();
const botStack = new BotStack(app, 'MonkeDiscordBot', {
    env: {
        account: "028071413917",
        region: "us-east-1"
    }
});
new LeaguePointService(app, 'MonkeLeaguePointService', {
    env: {
        account: "028071413917",
        region: "us-east-1"
    },
    webhook_handler: botStack.webhook_handler
});
