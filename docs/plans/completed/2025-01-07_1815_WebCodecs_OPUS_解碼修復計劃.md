# WebCodecs OPUS 解碼修復計劃

---
plan_id: webcodecs-opus-decoding-fix
status: active
priority: critical
category: bug-fix
created: 2025-01-07T18:15:00Z
updated: 2025-01-07T18:15:00Z
assignee: claude-code
estimated_hours: 2
tags: [webcodecs, opus, audio-decoding, rust, transcription]
---

## 🚨 問題概述

Care Voice 語音轉錄系統在處理 WebCodecs OPUS 音頻時出現嚴重品質問題，轉錄結果為無意義重複文字，而非正確的中文語音內容。

### 症狀
- **錯誤輸出**：`"請不各點資 訂閱 轉發 打賞支持明鏡與點擊目請不各點資 訂閱 轉發 打賞支持明鏡與點擊目"`
- **預期輸出**：`"神說要有了光就有...."`
- **技術錯誤**：28% OPUS 包解碼失敗，需要 FEC 恢復
- **系統錯誤**：轉錄超時 (30秒)

## 🔍 根本原因分析

### 初步診斷
1. **大量包損壞**：401-436 個包中約 30% 出現 `corrupted stream` 錯誤
2. **智能拆分問題**：WebCodecs 原始流拆分算法不準確
3. **音頻品質受損**：FEC 恢復雖成功但音頻品質下降
4. **轉錄模型困難**：受損音頻導致 Whisper 處理困難

### 業界標準研究發現

通過對最新業界標準的深入研究，發現**我們的修復方向完全錯誤**：

#### ❌ 錯誤假設
1. **連續流假設**：以為 WebCodecs 輸出連續的 OPUS 流
2. **整體解碼嘗試**：試圖將 139KB 數據作為單個包解碼
3. **忽略狀態管理**：未考慮 OPUS 有狀態特性

#### ✅ 業界標準事實
1. **獨立包序列**：WebCodecs 輸出 `EncodedAudioChunk` 對象序列，每個是獨立 OPUS 包
2. **有狀態解碼**：OPUS 是有狀態編解碼器，包間有重疊區塊，需要順序處理
3. **包級處理**：標準做法是維持單一解碼器實例，逐包解碼

## 🎯 技術解決方案

### 核心修復策略

基於 [Opus WebCodecs Registration](https://w3c.github.io/webcodecs/opus_codec_registration.html) 和 [Rust opus crate 文檔](https://docs.rs/opus/)：

#### 1. **正確的 OPUS 解碼模式**
```rust
// 業界標準做法
let mut decoder = Decoder::new(sample_rate, channels)?;
for packet in opus_packets {
    let decoded_audio = decoder.decode_float(packet, &mut output, false)?;
    // 維持解碼器狀態，處理有狀態重疊
}
```

#### 2. **改善包邊界檢測**
- 基於 OPUS TOC (Table of Contents) 字節精確識別
- 使用實際包長度而非固定大小拆分
- 實作智能包驗證機制

#### 3. **狀態管理優化**
- 單一解碼器實例貫穿整個會話
- 正確處理包間狀態依賴
- 實作錯誤恢復機制

## 📋 實施計劃

### Phase 1: 移除錯誤邏輯
- [ ] 移除 `decode_webcodecs_continuous_stream` 函數
- [ ] 恢復到改進的包級處理邏輯
- [ ] 清理相關錯誤處理代碼

### Phase 2: 重寫包拆分算法
- [ ] 實作基於 TOC 字節的包邊界檢測
- [ ] 改善包大小預測算法
- [ ] 增加包完整性驗證

### Phase 3: 優化解碼器狀態管理
- [ ] 修改解碼器池實現
- [ ] 確保單一解碼器實例持續性
- [ ] 實作狀態錯誤恢復

### Phase 4: 系統級優化
- [ ] 調整轉錄超時從 30s 到 60-90s
- [ ] 改善錯誤訊息和用戶反饋
- [ ] 增加處理進度指示器

### Phase 5: 測試驗證
- [ ] 測試相同語音樣本 `"神說要有了光就有...."`
- [ ] 驗證包錯誤率 < 5%
- [ ] 確認轉錄結果正確性
- [ ] 性能基準測試

## 🧪 測試計劃

### 測試案例
1. **基礎功能測試**
   - 輸入：中文語音 "神說要有了光就有...."
   - 預期：正確的中文轉錄結果
   - 成功標準：轉錄準確度 > 90%

2. **包解碼品質測試**
   - 監控：包解碼錯誤率
   - 目標：< 5% 包解碼失敗
   - 無需：大量 FEC 恢復

3. **性能測試**
   - 轉錄延遲：< 60 秒
   - 系統響應：無超時錯誤
   - 資源使用：合理的 CPU/內存消耗

### 驗收標準
- ✅ 正確的中文語音轉錄
- ✅ 包錯誤率 < 5%
- ✅ 無轉錄超時錯誤
- ✅ 符合業界 OPUS 處理標準

## 📊 風險評估

### 高風險
- **解碼器 API 更改**：可能影響其他音頻格式支援
- **狀態管理複雜性**：多線程環境下的狀態同步

### 中風險
- **性能影響**：優化後的拆分算法可能影響處理速度
- **相容性問題**：不同瀏覽器 WebCodecs 實作差異

### 緩解策略
- 保留原有邏輯作為後備方案
- 段階性部署和測試
- 詳細的錯誤日誌和監控

## 🔄 回滾計劃

如果修復失敗：
1. 恢復 git 提交：`git reset --hard HEAD~1`
2. 重新啟動服務：`./start.sh`
3. 驗證系統基本功能
4. 重新評估解決方案

## 📈 成功指標

### 定量指標
- 包解碼成功率：> 95%
- 轉錄準確度：> 90%
- 處理延遲：< 60 秒
- 系統可用性：> 99%

### 定性指標
- 用戶反饋正面
- 轉錄結果有意義
- 系統響應穩定
- 符合業界標準

## 📚 參考資料

### 技術標準
- [WebCodecs Opus Codec Registration](https://w3c.github.io/webcodecs/opus_codec_registration.html)
- [RFC 6716: Definition of the Opus Audio Codec](https://datatracker.ietf.org/doc/html/rfc6716)
- [Opus Codec Official Documentation](https://opus-codec.org/docs/)

### 實作參考
- [Rust opus crate](https://docs.rs/opus/)
- [WebCodecs AudioEncoder MDN](https://developer.mozilla.org/en-US/docs/Web/API/AudioEncoder)
- [OPUS Decoder API](https://opus-codec.org/docs/opus_api-1.5/group__opus__decoder.html)

### 社群討論
- [Stack Overflow: WebCodecs OPUS Frames](https://stackoverflow.com/questions/73665100/using-web-api-audioencoder-to-output-opus-frames)
- [GitHub Issue: WebCodecs Opus Implementation](https://github.com/xiph/opus/issues/221)

---

**計劃建立時間**: 2025-01-07T18:15:00Z  
**預估完成時間**: 2025-01-07T20:00:00Z  
**負責人**: Claude Code AI Assistant  
**狀態**: ✅ Active - 已核准實施  