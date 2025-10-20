# Core

Nostrクライアントのコアロジックを提供するクレート。

## 責務

- **Storage抽象化**: IndexedDB実装を含むStorage trait
- **Relay接続管理**: WebSocket接続、再接続、指数バックオフ
- **購読管理**: NIP-01購読、EOSE処理、時間窓の段階的拡大
- **送信キュー**: イベント送信、NIP-20 OK確認、再送ロジック
- **署名**: NIP-07対応、内蔵鍵（WebCrypto暗号化）
- **暗号化**: NIP-04 DM暗号化/復号化

## 主要API

```rust
pub struct CoreHandle { ... }

impl CoreHandle {
    pub async fn init(relay_urls: Vec<String>, storage: Arc<dyn Storage>) -> Result<Self>;
    pub async fn open_channel(&mut self, channel_id: &str);
    pub async fn open_dm(&mut self, peer: &str);
    pub async fn send_public(&mut self, channel_id: &str, content: &str) -> String;
    pub async fn send_dm(&mut self, peer: &str, plaintext: &str) -> String;
    pub fn poll_events(&mut self, max: u32) -> Vec<UiRow>;
    pub async fn tick(&mut self);
}
```

## 将来の拡張

このクレートは将来的に独立したライブラリとして公開可能な設計になっています。

