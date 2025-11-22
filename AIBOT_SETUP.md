# AI Bot Setup Guide

This document provides instructions for setting up the "Brown Ninja Bot", an AI-powered assistant that helps manage issues and pull requests in this repository.

## Overview

The Brown Ninja Bot uses a combination of a GitHub App and the Opencode.ai service to:
- Automatically analyze new issues.
- Propose code fixes and create pull requests.
- Respond to workflow failures.

To enable this functionality, you need to configure several secrets in your repository.

## Required Secrets

The following secrets must be configured in your repository's **Settings > Secrets and variables > Actions** page for the bot to function correctly:

- `APP_ID`: The unique ID of your custom GitHub App.
- `APP_PRIVATE_KEY`: The private key generated for your GitHub App.
- `OPENCODE_API_KEY`: Your API key for the Opencode.ai service.

### Automated Setup with `make update-secrets`

For convenience, you can use the `make update-secrets` command to set all required GitHub repository secrets from a local `.env` file.

**Steps:**

1.  Create a file named `.env` in the root of your project (it should *not* be committed to Git).
2.  Populate it with your secrets using the following format:

    ```bash
    # From your GitHub App
    APP_ID="YOUR_APP_ID_HERE"
    APP_PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----\n"

    # From Opencode.ai
    OPENCODE_API_KEY="YOUR_OPENCODE_API_KEY_HERE"

    # Example: Other secrets from .env.example
    AWS_IAM_ROLE_ARN="arn:aws:iam::123456789012:role/github-actions-role"
    AZURE_CLIENT_ID="your_azure_client_id"
    AZURE_TENANT_ID="your_azure_tenant_id"
    TF_BACKEND_BUCKET="your-terraform-state-bucket"
    ```
    **Important:** For the multi-line `APP_PRIVATE_KEY`, you must replace actual newlines with `\n` characters and enclose the entire key in double quotes.

3.  Run the command:

    ```bash
    make update-secrets
    ```

    This will upload the secrets from your `.env` file to your GitHub repository for both GitHub Actions and Dependabot.

---

## Setup Instructions

### Part A: GitHub App Setup

The bot authenticates with GitHub using its own GitHub App.

**1. Create a New GitHub App**
   - Go to your GitHub account **Settings > Developer settings > GitHub Apps** and click **New GitHub App**.
   - Fill in the required details:
     - **App name:** Give it a unique name (e.g., "Brown Ninja Bot for [your-username]").
     - **Homepage URL:** You can use your repository URL.
   - You do not need to set a webhook URL for this bot's functionality.
   - **Repository permissions:** This is the most critical part. The bot needs the following permissions:
     - `Contents`: Read & write
     - `Issues`: Read & write
     - `Metadata`: Read-only
     - `Pull requests`: Read & write
   - Under "Where can this GitHub App be installed?", select **Only on this account**.
   - Click **Create GitHub App**.

**2. Get the App ID**
   - After creating the app, you will be redirected to its settings page.
   - The **App ID** is a number displayed on this page.
   - Go to your repository's **Settings > Secrets and variables > Actions**, and create a new repository secret named `APP_ID`. Paste the App ID here.

**3. Generate a Private Key**
   - On your GitHub App's settings page, scroll down to the "Private keys" section.
   - Click **Generate a private key**.
   - A `.pem` file will be downloaded to your computer. **Treat this file like a password.**
   - Open the `.pem` file with a text editor. Copy the entire content, including `-----BEGIN RSA PRIVATE KEY-----` and `-----END RSA PRIVATE KEY-----`.
   - Go to your repository's **Settings > Secrets and variables > Actions**, and create a new repository secret named `APP_PRIVATE_KEY`.
   - Paste the full, multi-line content of the `.pem` file into the secret's value field. It is crucial that the newlines are preserved.

**4. Install the GitHub App**
   - This is the final and most important step for the GitHub App.
   - On your GitHub App's settings page, go to the "Install App" tab in the sidebar.
   - Click **Install** next to your username or organization.
   - On the next screen, you can choose to install the app on "All repositories" or "Only select repositories".
   - **You must select the repository where this workflow is located (e.g., `debanjanbasu/aws-lambda-mcp`)** and click **Install**.
   - If you don't do this, you will get an "Integration not found" error in your workflow runs.

---

### Part B: Opencode.ai Setup

The bot uses the Opencode.ai service for its code generation capabilities.

**1. Get your API Key**
   - Sign up or log in to [opencode.ai](https://opencode.ai/).
   - Navigate to your account settings or API section to find your API key. For detailed instructions, refer to the official documentation: [https://opencode.ai/docs#configure](https://opencode.ai/docs#configure)

**2. Configure the Repository Secret**
   - Go to your repository's **Settings > Secrets and variables > Actions**.
   - Create a new repository secret named `OPENCODE_API_KEY`.
   - Paste your Opencode.ai API key into the value field.
