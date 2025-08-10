module.exports = {
  name: 'doc-lifecycle',
  events: ['PostToolUse'],
  
  async execute(context) {
    const { filePath, toolName } = context;
    
    // 只處理文檔相關的檔案操作
    if (toolName !== 'Write' || !filePath.endsWith('.md')) {
      return;
    }
    
    try {
      const content = await context.readFile(filePath);
      
      // 檢測新計畫創建 (擴展監控範圍)
      if ((filePath.includes('projects/active/') || filePath.includes('docs/plans/active/')) && 
          (filePath.endsWith('README.md') || filePath.endsWith('.md'))) {
        console.log("📝 檢測到新計畫創建");
        context.suggest('project-doc-manager', '設定新計畫標準結構');
      }
      
      // 檢測系統級決策
      const systemKeywords = /架構|技術選型|資料庫|API設計|安全策略|部署方案|WebSocket|WebCodecs|Whisper|GPU/;
      if (systemKeywords.test(content)) {
        console.log("🚨 檢測到系統級決策");
        context.suggest('system-doc-maintainer', '檢查是否需要更新系統文檔');
      }
      
      // 檢測計畫完成
      if (content.includes('完成') || content.includes('已完成') || content.includes('completed')) {
        console.log("✅ 檢測到計畫完成");
        context.suggest('doc-decision-helper', '評估計畫文檔處理方式');
      }
      
    } catch (error) {
      console.log(`Hook執行錯誤: ${error.message}`);
    }
  }
};
