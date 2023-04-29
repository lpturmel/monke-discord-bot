#!/usr/bin/env node
import * as cdk from 'aws-cdk-lib';
import { AwsStack } from '../lib/aws-stack';
import { config } from 'dotenv';
config();

const app = new cdk.App();
new AwsStack(app, 'MonkeDiscordBot', {
    env: {
        account: "028071413917",
        region: "us-east-1"
    }
});
