# GitHub Secrets Configuration

This document describes the required GitHub secrets for GT Tooling workflows.

## Required Secrets

### WORKFLOW_TOKEN

A GitHub Personal Access Token (PAT) with the following permissions:

- **Purpose**: Enables workflows to create PRs, manage branches, and interact with the repository
- **Required Permissions**:
  - Actions: Read and Write
  - Contents: Read and Write
  - Pull Requests: Read and Write
  - Workflows: Read and Write

#### Setup Instructions

1. Go to GitHub.com → Settings → Developer settings → Personal access tokens → Fine-grained tokens
2. Click "Generate new token"
3. Configure:
   ```
   Token name: GT-Tooling Workflow Token
   Expiration: 1 year (recommended)
   Repository access: Only select repositories → gt-tooling
   ```
4. Set permissions:
   ```
   Repository permissions:
   - Actions: Read and Write
   - Contents: Read and Write
   - Pull Requests: Read and Write
   - Workflows: Read and Write
   ```
5. Generate token and copy it
6. Add to repository:
   - Go to repository Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `WORKFLOW_TOKEN`
   - Value: [Your generated PAT]

### SNYK_TOKEN

A Snyk.io API token for security scanning:

- **Purpose**: Enables Snyk security scanning for Node.js dependencies
- **Required For**: Security scanning workflow

#### Setup Instructions

1. Go to [Snyk.io](https://snyk.io) and sign up/sign in
2. Go to Account Settings → Auth Token
3. Generate or copy existing token
4. Add to repository:
   - Go to repository Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `SNYK_TOKEN`
   - Value: [Your Snyk token]

## Verification

To verify your secrets are configured correctly:

1. Go to the "Actions" tab in your repository
2. Select "Test Secrets Configuration" workflow
3. Click "Run workflow"
4. Choose test type:
   - `all`: Test all secrets
   - `workflow`: Test only WORKFLOW_TOKEN
   - `snyk`: Test only SNYK_TOKEN

### Expected Results

- **WORKFLOW_TOKEN Test**:
  - Should create and delete a test branch
  - Should create and close a test PR
  - Both operations should complete without permission errors

- **SNYK_TOKEN Test**:
  - Should successfully authenticate with Snyk
  - Should run a security scan (may fail if vulnerabilities found)

## Troubleshooting

If tests fail:

1. **WORKFLOW_TOKEN Issues**:
   - Verify token hasn't expired
   - Check all permissions are correctly set
   - Ensure token has access to this repository

2. **SNYK_TOKEN Issues**:
   - Verify token is valid in Snyk dashboard
   - Check if Snyk account is active
   - Ensure token has correct permissions

## Security Notes

- Never commit tokens to the repository
- Regularly rotate tokens (recommended: every 6-12 months)
- Use the principle of least privilege when setting permissions
- Monitor token usage in GitHub security logs
