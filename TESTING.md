# テスト戦略

## 概要

RustrはeGui/canvasベースのアプリケーションであるため、通常のDOM操作ベースのE2Eテストツールは使用できません。
代わりに、以下の3層のテスト戦略を採用しています。

## 1. ユニットテスト（ネイティブ）

### 実行方法

```bash
./scripts/test.sh
```

### 対象

- `core/src/relay.rs`: WebSocketメッセージのパース、指数バックオフ
- `core/src/subscription.rs`: 購読管理、EOSE処理（WASM専用）
- `core/src/outbox.rs`: 送信キュー管理（WASM専用）
- `core/tests/integration_test.rs`: 統合テスト

### 制約

- **WASM環境が必要な機能**（`js_sys::Date::now()`など）を使うテストは、`#[cfg(all(test, target_arch = "wasm32"))]`でWASM専用にしています
- ネイティブターゲットでは、`js_sys`を使わない純粋なロジックのみテストします

## 2. WASM統合テスト（ブラウザ）

### 実行方法

```bash
cd ui
wasm-pack test --headless --chrome
```

### 対象

- `core/src/subscription.rs::tests`: チャンネル/DM購読、EOSE拡張
- `core/src/outbox.rs::tests`: イベントエンキュー、OK応答処理
- `ui/tests/e2e_helper.rs`: アプリケーション起動確認、Canvas操作

### 制約

- **現在は`secp256k1`のビルド問題により実行不可**
- 将来的にはCI/CD環境（GitHub Actions）で実行予定

## 3. 手動E2Eテスト

### 実行方法

```bash
./scripts/dev-server.sh
# ブラウザで http://localhost:8080 を開く
```

### テストシナリオ

#### オンボーディング
1. [ ] NIP-07拡張機能が検出される
2. [ ] 新規キー生成が機能する
3. [ ] 既存キーのインポートが機能する

#### チャンネル
1. [ ] デフォルトチャンネル一覧が表示される
2. [ ] チャンネルを開くとタイムラインが表示される
3. [ ] メッセージ送信が機能する
4. [ ] 他のユーザーのメッセージが表示される
5. [ ] スクロールで過去のメッセージを読み込む

#### DM
1. [ ] DM一覧が表示される
2. [ ] DMを開くとタイムラインが表示される
3. [ ] DM送信が機能する（暗号化）
4. [ ] 受信DMが復号化されて表示される

#### リレー接続
1. [ ] 初回接続時にリレーに接続される
2. [ ] 接続状態がUIに表示される
3. [ ] 切断時に再接続が試みられる

#### UI/UX
1. [ ] スマホでタッチ操作が機能する
2. [ ] IME入力が機能する（日本語など）
3. [ ] フォントサイズが適切
4. [ ] スペーシングが適切

## 4. CI/CD（GitHub Actions）

### 設定ファイル

`.github/workflows/test.yml`

### ジョブ

1. **test-core**: ネイティブターゲットでのユニットテスト
2. **test-wasm**: WASM環境でのブラウザテスト（将来）
3. **lint**: フォーマット、Clippy

### 実行タイミング

- `main`ブランチへのpush
- Pull Request作成時

## テストヘルパー

### `ui/tests/e2e_helper.rs`

ブラウザ環境でのテストヘルパー関数を提供：

- `TestHelper::is_app_running()`: アプリケーションが起動しているか確認
- `TestHelper::get_canvas_size()`: Canvasのサイズを取得
- `TestHelper::click_canvas(x, y)`: Canvas上の座標をクリック

### 使用例

```rust
#[wasm_bindgen_test]
fn test_app_starts() {
    assert!(TestHelper::is_app_running(), "App should be running");
}
```

## 今後の改善

1. **WASM環境でのテスト実行**
   - `secp256k1`のビルド問題を解決
   - CI/CDでのブラウザテスト自動化

2. **カバレッジ測定**
   - `cargo-tarpaulin`または`grcov`の導入

3. **パフォーマンステスト**
   - 大量メッセージ処理のベンチマーク
   - メモリ使用量の監視

4. **スナップショットテスト**
   - UIの視覚的回帰テスト（将来的に）

