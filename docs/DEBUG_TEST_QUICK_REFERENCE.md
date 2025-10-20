# デバッグテストモード - クイックリファレンス

## 🚀 クイックスタート

```bash
# 1. ビルド
./scripts/build-wasm-debug.sh

# 2. サーバー起動
cd ui/pkg && python3 -m http.server 8080

# 3. ブラウザで開く
# http://localhost:8080/?debug_test=1
```

## 📋 デフォルトシナリオ

1. ⏸️  Idle → 待機
2. 🔑 OnboardingCreateKey → 新規キー生成
3. 🏠 TransitionToMain → メイン画面確認
4. 📢 OpenChannel → チャンネルオープン
5. ✉️  SendMessage → メッセージ送信
6. 📊 VerifyTimeline → タイムライン確認
7. ✅ Completed → 完了

## 🔧 カスタマイズ場所

### シナリオ編集
`ui/src/debug_test.rs` の `scenario` 配列

```rust
let scenario = vec![
    TestStep::Idle,
    TestStep::OnboardingCreateKey,
    // ここに追加
    TestStep::Completed,
];
```

### 待機時間調整
`ui/src/debug_test.rs` の `wait_frames`

```rust
wait_frames: 60,  // 1秒（60fps想定）
```

### デバッグAPI追加
`ui/src/app.rs` に追加

```rust
#[cfg(feature = "debug-test")]
pub fn debug_your_method(&self) -> ReturnType {
    // 実装
}
```

## 📝 よく使うコード

### 新しいステップ追加

```rust
// 1. TestStep enum に追加
#[derive(Debug, Clone, PartialEq)]
pub enum TestStep {
    YourNewStep { param: String },
}

// 2. scenario に追加
TestStep::YourNewStep { param: "value".to_string() },

// 3. tick() に処理追加
TestStep::YourNewStep { param } => {
    log::info!("🧪 Executing: {}", param);
    app.debug_your_method(param.clone());
    self.advance_step();
}
```

### 条件付き待機

```rust
TestStep::WaitForConnection => {
    if app.debug_is_connected() {
        log::info!("✅ Connected!");
        self.advance_step();
    } else {
        log::info!("⏳ Waiting...");
        // 次のフレームで再チェック
    }
}
```

### エラーハンドリング

```rust
TestStep::YourStep => {
    match app.debug_your_method() {
        Ok(_) => {
            log::info!("✅ Success");
            self.advance_step();
        }
        Err(e) => {
            log::error!("❌ Error: {:?}", e);
            self.current_step = TestStep::Completed;
        }
    }
}
```

## 🎨 ログ絵文字

- `🧪` テスト実行
- `✅` 成功
- `⏳` 待機
- `⚠️` 警告
- `❌` エラー
- `📊` データ/統計
- `🔑` 認証/キー
- `📢` チャンネル
- `💬` DM
- `🏠` メイン画面

## 🐛 トラブルシューティング

| 症状 | 原因 | 対処 |
|------|------|------|
| ステータスバーが出ない | URLパラメータなし | `?debug_test=1` を付ける |
| テストが開始しない | 通常ビルドを使用 | `build-wasm-debug.sh` でビルド |
| 途中で止まる | 待機時間不足 | `wait_frames` を増やす |
| エラーが出る | API未実装 | デバッグAPIを追加 |

## 📚 関連ドキュメント

- 詳細ガイド: `docs/DEBUG_TEST_MODE.md`
- テスト戦略: `TESTING.md`
- 実装ファイル: `ui/src/debug_test.rs`
- デバッグAPI: `ui/src/app.rs`

## ⚡ Tips

- **複数シナリオ**: 環境変数で切り替え可能
- **スクリーンショット**: ブラウザの開発者ツールで撮影
- **ログ保存**: コンソール右クリック → Save as...
- **速度調整**: `wait_frames` を変更
- **本番影響なし**: `#[cfg(feature = "debug-test")]` で保護

## 🔒 セキュリティ

本番ビルドでは完全に除外されます：

```bash
# 本番用（デバッグコードなし）
./scripts/build-wasm.sh

# 開発用（デバッグコードあり）
./scripts/build-wasm-debug.sh
```

