/**
 * 📋 文檔定期維護檢查機制
 * 監控文檔過期、重複和品質問題
 */

const fs = require('fs');
const path = require('path');

function checkDocumentMaintenance(filePath, content, context) {
    const now = new Date();
    const threeMonthsAgo = new Date(now.getFullYear(), now.getMonth() - 3, now.getDate());
    
    // 檢查文檔是否過期（3個月未更新）
    if (filePath.endsWith('.md') && filePath.includes('docs/')) {
        const fileStats = fs.statSync(filePath);
        const lastModified = fileStats.mtime;
        
        if (lastModified < threeMonthsAgo) {
            console.log(`📅 檢測到過期文檔: ${filePath}`);
            console.log(`⏰ 最後更新: ${lastModified.toISOString().split('T')[0]}`);
            context.suggest('doc-decision-helper', `評估文檔是否需要更新或歸檔: ${filePath}`);
        }
    }
    
    // 檢查重複檔案名稱模式
    if (filePath.includes('docs/') && filePath.endsWith('.md')) {
        const fileName = path.basename(filePath, '.md');
        
        // 常見重複模式
        const duplicatePatterns = [
            /.*_REPORT\.md$/,
            /.*_SUMMARY\.md$/,
            /.*_GUIDE\.md$/,
            /.*_PLAN\.md$/
        ];
        
        duplicatePatterns.forEach(pattern => {
            if (pattern.test(fileName)) {
                console.log(`🔍 檢測到可能重複的文檔類型: ${fileName}`);
                context.suggest('system-doc-maintainer', `檢查是否有重複文檔需要合併`);
            }
        });
    }
    
    // 檢查計劃文檔完整性
    if (filePath.includes('docs/plans/') && content.includes('plan_id:')) {
        // 檢查必要欄位
        const requiredFields = ['status:', 'category:', 'priority:'];
        const missingFields = requiredFields.filter(field => !content.includes(field));
        
        if (missingFields.length > 0) {
            console.log(`📋 計劃文檔缺少必要欄位: ${missingFields.join(', ')}`);
            context.suggest('project-doc-manager', `補完計劃文檔欄位: ${filePath}`);
        }
    }
    
    // 檢查語言一致性
    const simplifiedChinesePatterns = [
        /测试/g, /归档/g, /文件夹/g, /软件/g, /硬件/g,
        /网络/g, /服务器/g, /数据库/g, /代码/g, /用户/g,
        /设置/g, /连接/g, /执行/g, /实时/g, /界面/g
    ];
    
    let hasSimplifiedChinese = false;
    simplifiedChinesePatterns.forEach(pattern => {
        if (pattern.test(content)) {
            hasSimplifiedChinese = true;
        }
    });
    
    if (hasSimplifiedChinese) {
        console.log(`🌏 檢測到簡體中文用語: ${filePath}`);
        context.suggest('smart-doc-router', `修正文檔為正體中文標準`);
    }
}

// 定期檢查機制（每次文檔變更時觸發）
module.exports = function(filePath, change, context) {
    try {
        if (change === 'modified' || change === 'created') {
            const content = fs.readFileSync(filePath, 'utf8');
            checkDocumentMaintenance(filePath, content, context);
        }
    } catch (error) {
        console.log(`❌ 文檔維護檢查失敗: ${error.message}`);
    }
};