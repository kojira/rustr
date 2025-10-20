# デバッグテストモード - 使い方ガイド

## 概要

RustrはeGui/canvasベースのアプリケーションであるため、通常のE2Eテストツール（Selenium、Playwright等）では
DOM要素にアクセスできません。そのため、**アプリケーション内部にテストランナーを埋め込む**方式を採用しています。

デバッグテストモードを有効にすると、アプリ起動時に自動でテストシナリオが実行され、
画面上部のバーとブラウザコンソールで進捗を確認できます。

## クイックスタート

### 1. デバッグモードでビルド

```bash
cd /path/to/rustr
./scripts/build-wasm-debug.sh
```

このスクリプトは以下を実行します：
- `--features debug-test` フラグ付きでWASMビルド
- 静的ファイル（index.html, app.js）をpkgディレクトリにコピー

### 2. 開発サーバーを起動

```bash
cd ui/pkg
python3 -m http.server 8080
```

または：

```bash
npx serve ui/pkg -p 8080
```

### 3. ブラウザで開く

**重要**: URLパラメータ `?debug_test=1` を付けて開きます。

```
http://localhost:8080/?debug_test=1
```

または：

```
http://localhost:8080/?debug_test=true
```

### 4. 自動テストの実行を確認

- 画面上部に黄色いバーが表示され、現在のステップが表示されます
- ブラウザの開発者ツール（F12）を開いてコンソールを確認
- `🧪` マークのログでテストの進行状況を確認できます

## 自動実行されるテストシナリオ

デフォルトでは以下のシナリオが自動実行されます：

### ステップ1: Idle（待機）
- テスト開始前の初期状態
- すぐに次のステップへ進みます

### ステップ2: OnboardingCreateKey（オンボーディングスキップ）
- 新規キー生成でオンボーディングをスキップ
- メイン画面に自動遷移

### ステップ3: TransitionToMain（メイン画面確認）
- アプリの状態が `Main` になっているか確認
- 確認できるまで待機

### ステップ4: OpenChannel（チャンネルオープン）
- テストチャンネル `test_channel_001` を開く
- リレーへのREQメッセージ送信

### ステップ5: SendMessage（メッセージ送信）
- 自動テストメッセージを送信
- 内容: `"🤖 自動テスト: Hello from debug mode!"`

### ステップ6: VerifyTimeline（タイムライン確認）
- タイムラインのイベント数を確認
- コンソールに `📊 Timeline has X events` と表示
- 3秒待機してから次へ

### ステップ7: Completed（完了）
- 全テスト完了
- 画面上部に `✅ Debug Test: COMPLETED` と表示

## 待機時間の調整

各ステップ間のデフォルト待機時間は **60フレーム（約1秒）** です。

待機時間を変更するには、`ui/src/debug_test.rs` の `wait_frames` を編集：

```rust
Self {
    enabled,
    current_step: TestStep::Idle,
    step_index: 0,
    frame_counter: 0,
    wait_frames: 120, // 2秒に変更（60fps想定）
    scenario,
}
```

特定のステップだけ長く待機する場合：

```rust
TestStep::VerifyTimeline => {
    log::info!("🧪 Verifying timeline...");
    let event_count = app.debug_get_timeline_count();
    log::info!("📊 Timeline has {} events", event_count);
    self.wait_frames = 300; // このステップだけ5秒待機
    self.advance_step();
}
```

## カスタムシナリオの作成

### 基本的な手順

1. `ui/src/debug_test.rs` を開く
2. `TestStep` enum に新しいステップを追加（必要に応じて）
3. `DebugTestRunner::new()` の `scenario` 配列を編集
4. `tick()` メソッドに新しいステップの処理を追加

### 例1: DM送信テストを追加

```rust
// TestStep enum に追加
#[derive(Debug, Clone, PartialEq)]
pub enum TestStep {
    // ... 既存のステップ
    OpenDm { peer: String },
    SendDm { content: String },
    VerifyDmTimeline,
}

// scenario に追加
let scenario = vec![
    TestStep::Idle,
    TestStep::OnboardingCreateKey,
    TestStep::TransitionToMain,
    TestStep::OpenChannel { 
        channel_id: "test_channel_001".to_string() 
    },
    TestStep::SendMessage { 
        content: "Public message test".to_string() 
    },
    TestStep::VerifyTimeline,
    
    // DM テスト追加
    TestStep::OpenDm { 
        peer: "npub1test...".to_string() 
    },
    TestStep::SendDm { 
        content: "🤖 DM test message".to_string() 
    },
    TestStep::VerifyDmTimeline,
    
    TestStep::Completed,
];

// tick() メソッドに処理を追加
TestStep::OpenDm { peer } => {
    log::info!("🧪 Opening DM with: {}", peer);
    app.debug_open_dm(peer.clone());
    self.advance_step();
}

TestStep::SendDm { content } => {
    log::info!("🧪 Sending DM: {}", content);
    app.debug_send_message(content.clone());
    self.advance_step();
}

TestStep::VerifyDmTimeline => {
    log::info!("🧪 Verifying DM timeline...");
    let event_count = app.debug_get_timeline_count();
    log::info!("📊 DM Timeline has {} events", event_count);
    self.wait_frames = 180;
    self.advance_step();
}
```

### 例2: リレー接続確認テスト

```rust
// TestStep に追加
VerifyRelayConnection,

// scenario に追加
TestStep::VerifyRelayConnection,

// app.rs にデバッグAPIを追加
#[cfg(feature = "debug-test")]
pub fn debug_get_connected_relay_count(&self) -> usize {
    if let Some(core) = self.core.borrow().as_ref() {
        core.relays.iter().filter(|r| r.is_connected()).count()
    } else {
        0
    }
}

// tick() に処理を追加
TestStep::VerifyRelayConnection => {
    let connected = app.debug_get_connected_relay_count();
    log::info!("🧪 Connected relays: {}", connected);
    if connected > 0 {
        log::info!("✅ At least one relay is connected");
        self.advance_step();
    } else {
        log::warn!("⏳ Waiting for relay connection...");
        // 接続されるまで待機
    }
}
```

### 例3: エラーハンドリングテスト

```rust
// TestStep に追加
TestInvalidMessage,
VerifyErrorHandling,

// scenario に追加
TestStep::TestInvalidMessage,
TestStep::VerifyErrorHandling,

// tick() に処理を追加
TestStep::TestInvalidMessage => {
    log::info!("🧪 Testing invalid message handling...");
    // 空メッセージを送信してエラーハンドリングをテスト
    app.debug_send_message("".to_string());
    self.advance_step();
}

TestStep::VerifyErrorHandling => {
    // エラーメッセージが表示されているか確認
    if let Some(error) = app.debug_get_error_message() {
        log::info!("✅ Error handled correctly: {}", error);
    } else {
        log::warn!("⚠️ No error message displayed");
    }
    self.advance_step();
}
```

## デバッグAPIの追加

新しいテストステップを追加する際、アプリの状態にアクセスする必要がある場合は、
`ui/src/app.rs` にデバッグAPIを追加します。

### デバッグAPIの追加方法

```rust
// ui/src/app.rs の impl NostrApp ブロック内に追加

#[cfg(feature = "debug-test")]
pub fn debug_your_method_name(&self) -> YourReturnType {
    // アプリの内部状態にアクセス
    // 例: self.timeline.event_count()
}
```

**重要**: 必ず `#[cfg(feature = "debug-test")]` 属性を付けてください。
これにより、通常のリリースビルドにはデバッグコードが含まれません。

### 既存のデバッグAPI

```rust
// オンボーディングをスキップ
pub fn debug_skip_onboarding(&mut self)

// メイン画面かどうか確認
pub fn is_main_screen(&self) -> bool

// チャンネルを開く
pub fn debug_open_channel(&mut self, channel_id: String)

// DMを開く
pub fn debug_open_dm(&mut self, peer: String)

// メッセージ送信
pub fn debug_send_message(&mut self, content: String)

// タイムラインのイベント数取得
pub fn debug_get_timeline_count(&self) -> usize
```

## ログ出力のカスタマイズ

### コンソールログ

テストランナーは以下の絵文字プレフィックスでログを出力します：

- `🧪` テストステップの実行
- `✅` 成功
- `⏳` 待機中
- `⚠️` 警告
- `📊` データ/統計情報

独自のログを追加する場合：

```rust
log::info!("🧪 Your test message");
log::warn!("⚠️ Warning message");
log::error!("❌ Error message");
```

### UI表示のカスタマイズ

画面上部のステータスバーの表示をカスタマイズするには、
`get_status_text()` メソッドを編集：

```rust
pub fn get_status_text(&self) -> String {
    if !self.enabled {
        return String::new();
    }

    match &self.current_step {
        TestStep::Completed => {
            "✅ Debug Test: ALL TESTS PASSED!".to_string()
        }
        _ => {
            format!(
                "🧪 Test [{}/{}] {} (frame: {})",
                self.step_index,
                self.scenario.len(),
                self.current_step_name(), // カスタムメソッド
                self.frame_counter
            )
        }
    }
}

// ステップ名を取得するヘルパーメソッド
fn current_step_name(&self) -> &str {
    match &self.current_step {
        TestStep::Idle => "Initializing",
        TestStep::OnboardingCreateKey => "Creating Key",
        TestStep::TransitionToMain => "Loading Main",
        TestStep::OpenChannel { .. } => "Opening Channel",
        TestStep::SendMessage { .. } => "Sending Message",
        TestStep::VerifyTimeline => "Verifying Timeline",
        TestStep::Completed => "Completed",
    }
}
```

## トラブルシューティング

### テストが開始されない

**症状**: 画面上部にステータスバーが表示されない

**原因と対処**:
1. URLパラメータを確認: `?debug_test=1` が付いているか
2. ビルド確認: `./scripts/build-wasm-debug.sh` でビルドしたか
3. ブラウザコンソールを確認: `🧪 Debug test mode enabled!` が表示されているか

### テストが途中で止まる

**症状**: 特定のステップで進まなくなる

**対処**:
1. ブラウザコンソールでエラーを確認
2. 待機時間を増やす: `self.wait_frames = 300;`
3. ステップの条件を確認（例: リレー接続待ちなど）

### エラーが発生する

**症状**: コンソールにエラーメッセージが表示される

**対処**:
1. エラーメッセージを確認
2. デバッグAPIの実装を確認
3. アプリの状態を確認（例: `core` が初期化されているか）

### テストが速すぎる/遅すぎる

**対処**:
- `wait_frames` を調整
- 60fps想定で、60フレーム = 1秒
- 特定のステップだけ調整可能

## 本番ビルドへの影響

デバッグテスト機能は `#[cfg(feature = "debug-test")]` で保護されているため、
通常のビルド（`./scripts/build-wasm.sh`）では**完全に除外**されます。

```bash
# 通常ビルド（デバッグコードなし）
./scripts/build-wasm.sh

# デバッグビルド（デバッグコードあり）
./scripts/build-wasm-debug.sh
```

本番環境では通常ビルドを使用してください。

## CI/CDでの利用

GitHub Actionsなどでヘッドレスブラウザテストを実行する場合：

```yaml
- name: Build WASM (debug)
  run: ./scripts/build-wasm-debug.sh

- name: Install Chrome
  uses: browser-actions/setup-chrome@latest

- name: Run debug test
  run: |
    cd ui/pkg
    python3 -m http.server 8080 &
    SERVER_PID=$!
    
    # ヘッドレスChromeで開く
    google-chrome \
      --headless \
      --disable-gpu \
      --dump-dom \
      "http://localhost:8080/?debug_test=1"
    
    # サーバー停止
    kill $SERVER_PID
```

## まとめ

- ✅ URLパラメータ `?debug_test=1` で有効化
- ✅ `./scripts/build-wasm-debug.sh` でビルド
- ✅ `ui/src/debug_test.rs` でシナリオをカスタマイズ
- ✅ `ui/src/app.rs` にデバッグAPIを追加
- ✅ ブラウザコンソールで進捗確認
- ✅ 本番ビルドには影響なし

詳細は `TESTING.md` も参照してください。

