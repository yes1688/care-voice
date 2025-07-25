# Git 工作流程指南

## 🌟 分支策略 (Git Flow)

### 分支結構
```
main (生產分支)
├── develop (開發分支)
├── feature/* (功能分支)
├── release/* (發布分支)
└── hotfix/* (緊急修復分支)
```

### 分支說明

#### 🎯 main (主分支)
- **用途**: 生產環境代碼
- **特點**: 永遠保持穩定可部署狀態
- **保護**: 只能通過 PR 合併
- **標籤**: 每個版本發布都應該打標籤

#### 🚀 develop (開發分支)  
- **用途**: 開發整合分支
- **特點**: 包含最新開發功能
- **來源**: 所有 feature 分支合併到此
- **目標**: 定期合併到 main 進行發布

#### ⚡ feature/* (功能分支)
- **命名**: `feature/功能描述` 或 `feature/issue-編號`
- **例子**: `feature/whisper-upgrade`, `feature/audio-processing`
- **來源**: 從 develop 分支創建
- **合併**: 完成後合併回 develop

#### 📦 release/* (發布分支)
- **命名**: `release/版本號`
- **例子**: `release/v1.0.0`
- **用途**: 準備新版本發布
- **合併**: 同時合併到 main 和 develop

#### 🔥 hotfix/* (緊急修復分支)
- **命名**: `hotfix/問題描述`
- **例子**: `hotfix/security-patch`
- **來源**: 從 main 分支創建
- **合併**: 同時合併到 main 和 develop

## 📋 工作流程

### 1. 開始新功能
```bash
# 切換到最新的 develop 分支
git checkout develop
git pull origin develop

# 創建功能分支
git checkout -b feature/新功能名稱

# 開發和提交
git add .
git commit -m "feat: 添加新功能"

# 推送到遠端
git push -u origin feature/新功能名稱
```

### 2. 完成功能開發
```bash
# 確保與 develop 同步
git checkout develop
git pull origin develop
git checkout feature/新功能名稱
git rebase develop

# 推送更新
git push -f origin feature/新功能名稱

# 創建 Pull Request 到 develop
```

### 3. 準備發布
```bash
# 從 develop 創建發布分支
git checkout develop
git pull origin develop
git checkout -b release/v1.0.0

# 更新版本號、修復最後問題
git commit -m "chore: 準備 v1.0.0 發布"

# 合併到 main
git checkout main
git merge release/v1.0.0
git tag v1.0.0

# 合併回 develop
git checkout develop
git merge release/v1.0.0

# 刪除發布分支
git branch -d release/v1.0.0
```

### 4. 緊急修復
```bash
# 從 main 創建修復分支
git checkout main
git checkout -b hotfix/緊急問題

# 修復問題
git commit -m "fix: 修復緊急問題"

# 合併到 main
git checkout main
git merge hotfix/緊急問題
git tag v1.0.1

# 合併到 develop
git checkout develop
git merge hotfix/緊急問題

# 刪除修復分支
git branch -d hotfix/緊急問題
```

## 🏷️ 提交信息規範

### 格式
```
<類型>(範圍): <描述>

[可選的正文]

[可選的腳註]
```

### 類型
- `feat`: 新功能
- `fix`: 錯誤修復
- `docs`: 文檔變更
- `style`: 代碼格式變更
- `refactor`: 重構
- `test`: 測試相關
- `chore`: 建構過程或輔助工具變動

### 例子
```
feat(audio): 添加 Opus 音頻編碼支持

- 實現 Opus 編碼器集成
- 添加音頻質量配置選項
- 更新音頻處理管道

Closes #123
```

## 🔧 Git 配置建議

### 全局配置
```bash
# 設置用戶信息
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# 設置預設分支名稱
git config --global init.defaultBranch main

# 設置合併策略
git config --global pull.rebase false
git config --global merge.ff false

# 設置編輯器
git config --global core.editor "code --wait"
```

### 分支保護 (GitHub/GitLab)
- **main 分支**:
  - 要求 PR 審查
  - 要求狀態檢查通過
  - 禁止強制推送
  - 禁止刪除

- **develop 分支**:
  - 要求 PR 審查
  - 允許管理員繞過

## 📊 分支命名規範

### 功能分支
- `feature/whisper-integration`
- `feature/user-authentication`
- `feature/issue-42-audio-upload`

### 修復分支
- `fix/memory-leak`
- `fix/audio-playback-bug`
- `hotfix/security-vulnerability`

### 文檔分支
- `docs/api-documentation`
- `docs/setup-guide`

### 重構分支
- `refactor/audio-pipeline`
- `refactor/database-layer`

## 🎯 最佳實踐

### ✅ 推薦做法
1. **小而頻繁的提交**: 每個提交都應該是一個邏輯單元
2. **描述性的提交信息**: 說明為什麼而不只是做了什麼
3. **定期同步**: 經常從上游分支拉取更新
4. **使用 rebase**: 保持線性歷史記錄
5. **刪除已合併分支**: 保持倉庫整潔

### ❌ 避免做法
1. **直接推送到 main**: 總是通過 PR 合併
2. **大型提交**: 避免一次性提交大量變更
3. **模糊的提交信息**: 避免使用 "fix", "update" 等不明確的信息
4. **長期存在的分支**: 及時合併或刪除功能分支
5. **強制推送共享分支**: 只在個人分支使用 force push

## 🔄 當前專案狀態

### 現有分支
- ✅ `main`: 當前最新代碼 (21a6773)
- ✅ `develop`: 開發分支 (21a6773)  
- ⚠️ `master`: 舊分支，建議刪除 (8455945)

### 建議下一步
1. 刪除舊的 `master` 分支
2. 設置遠端儲存庫並推送分支
3. 在遠端設置分支保護規則
4. 開始使用標準工作流程進行開發

---

**建立時間**: 2025-07-26 03:25  
**基於**: Git Flow 和 GitHub Flow 最佳實踐