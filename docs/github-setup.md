# GitHub Repository Setup Guide

This guide helps you configure GitHub repository settings for the Contexture project.

## Table of Contents

1. [Workflow Permissions](#workflow-permissions)
2. [Secrets Configuration](#secrets-configuration)
3. [Branch Protection Rules](#branch-protection-rules)

---

## Workflow Permissions

### Step 1: Configure Workflow Permissions

1. Go to your GitHub repository
2. Click **Settings** → **Actions** → **General**
3. Scroll down to **Workflow permissions**
4. Select **Read and write permissions**
5. Check ✅ **Allow GitHub Actions to create and approve pull requests**
6. Click **Save**

**Why?** This allows workflows to:
- Create releases (for the release workflow)
- Upload artifacts
- Comment on pull requests
- Update benchmark results

---

## Secrets Configuration

### Required Secrets

Currently, the project doesn't require any secrets for basic functionality. However, you may want to add these optional secrets:

### Optional Secrets

#### 1. CODECOV_TOKEN (Optional for private repos)

**Purpose:** Upload coverage reports to Codecov

**How to get it:**
1. Go to [codecov.io](https://codecov.io)
2. Sign in with GitHub
3. Add your repository
4. Copy the upload token

**How to add:**
1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Click **New repository secret**
3. Name: `CODECOV_TOKEN`
4. Value: Paste your token
5. Click **Add secret**

#### 2. PERSONAL_ACCESS_TOKEN (Optional for advanced features)

**Purpose:** For advanced GitHub API operations

**How to create:**
1. Go to **GitHub Settings** → **Developer settings** → **Personal access tokens** → **Tokens (classic)**
2. Click **Generate new token (classic)**
3. Name: `Contexture CI/CD`
4. Expiration: Choose appropriate duration
5. Select scopes:
   - ✅ `repo` (Full control of private repositories)
   - ✅ `workflow` (Update GitHub Action workflows)
6. Click **Generate token**
7. Copy the token immediately (you won't see it again!)

**How to add:**
1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Click **New repository secret**
3. Name: `PERSONAL_ACCESS_TOKEN`
4. Value: Paste your token
5. Click **Add secret**

### Verify Secrets

After adding secrets, you should see them listed under:
**Settings** → **Secrets and variables** → **Actions** → **Repository secrets**

---

## Branch Protection Rules

### Step 1: Protect the `main` Branch

1. Go to **Settings** → **Branches**
2. Click **Add branch protection rule**
3. Branch name pattern: `main`

### Step 2: Configure Protection Rules

#### Required Settings

✅ **Require a pull request before merging**
- ✅ Require approvals: `1`
- ✅ Dismiss stale pull request approvals when new commits are pushed
- ✅ Require review from Code Owners (optional)

✅ **Require status checks to pass before merging**
- ✅ Require branches to be up to date before merging
- **Required status checks** (add these after first workflow run):
  
  **重要提示：** 状态检查项只有在工作流运行至少一次后才会出现在列表中。
  
  **首次配置步骤：**
  1. 先不添加任何状态检查，保存分支保护规则
  2. 推送代码触发工作流运行
  3. 等待工作流完成
  4. 返回分支保护设置，点击 "Edit" 编辑规则
  5. 在 "Status checks that are required" 搜索框中搜索并添加：
     - `rust-tests (ubuntu-latest, stable)` 或 `Rust Core Tests (ubuntu-latest, stable)`
     - `rust-tests (windows-latest, stable)` 或 `Rust Core Tests (windows-latest, stable)`
     - `rust-tests (macos-latest, stable)` 或 `Rust Core Tests (macos-latest, stable)`
     - `code-coverage` 或 `Code Coverage`
     - `security-audit` 或 `Security Audit`
  
  **注意：** 状态检查的名称可能是：
  - Job 名称：`Rust Core Tests`
  - 或完整名称：`rust-tests (ubuntu-latest, stable)`
  
  **如何查看可用的状态检查：**
  1. 进入 Actions 标签页
  2. 点击最近的工作流运行
  3. 查看左侧的 Job 名称
  4. 这些就是可以添加为必需检查的名称

✅ **Require conversation resolution before merging**

✅ **Require signed commits** (optional but recommended)

✅ **Require linear history** (optional)

✅ **Include administrators** (optional - enforces rules on admins too)

#### Optional Settings

⬜ **Allow force pushes** (NOT recommended for main)
⬜ **Allow deletions** (NOT recommended for main)

### Step 3: Save the Rule

Click **Create** or **Save changes**

---

## Additional Recommendations

### 1. Enable Dependabot

1. Go to **Settings** → **Code security and analysis**
2. Enable:
   - ✅ **Dependency graph**
   - ✅ **Dependabot alerts**
   - ✅ **Dependabot security updates**

### 2. Configure Code Scanning

1. Go to **Security** → **Code scanning**
2. Click **Set up code scanning**
3. Choose **CodeQL Analysis**
4. Commit the workflow file

### 3. Set Up Environments (Optional)

For production deployments:

1. Go to **Settings** → **Environments**
2. Click **New environment**
3. Name: `production`
4. Configure:
   - ✅ Required reviewers
   - ✅ Wait timer (optional)
   - ✅ Deployment branches: `main` only

---

## Verification Checklist

After completing the setup, verify:

- [ ] Workflow permissions are set to "Read and write"
- [ ] Required secrets are added (if any)
- [ ] Branch protection rules are active on `main`
- [ ] Status checks are required before merging
- [ ] Dependabot is enabled
- [ ] First workflow run succeeds

---

## Troubleshooting

### Workflow Permission Errors

**Error:** `Resource not accessible by integration`

**Solution:** 
1. Check workflow permissions in Settings → Actions → General
2. Ensure "Read and write permissions" is selected

### Status Checks Not Appearing

**Problem:** Required status checks don't show up in the list

**Solution:**
1. Push a commit to trigger the workflows
2. Wait for workflows to complete
3. Status checks will appear in the branch protection settings
4. Add them to required checks

### Secrets Not Working

**Problem:** Workflow can't access secrets

**Solution:**
1. Verify secret names match exactly (case-sensitive)
2. Check that secrets are added at repository level, not environment level
3. Ensure workflow has permission to access secrets

---

## Next Steps

After completing this setup:

1. ✅ Mark tasks 0.1.4 and 0.1.5 as complete in `tasks.md`
2. Push your first commit to test the CI/CD pipeline
3. Create a pull request to test branch protection
4. Monitor the Actions tab for workflow results

---

## Support

If you encounter issues:

1. Check the [GitHub Actions documentation](https://docs.github.com/en/actions)
2. Review workflow logs in the Actions tab
3. Check the [Contexture CI/CD documentation](ci-cd.md)

---

**Last Updated:** 2024
**Maintainer:** Contexture Team
