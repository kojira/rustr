# Rustr - egui Nostr Client

スマホ最適化されたegui製NostrクライアントWebアプリケーション（WASM）

## 特徴

- **NIP-28**: パブリックチャット対応
- **NIP-04**: DM（ダイレクトメッセージ）対応
- **NIP-07**: ブラウザ拡張機能による署名
- **Storage抽象化**: IndexedDB（将来的にネイティブ移行可能）
- **スマホ最適化**: タッチ操作、IME対応、レスポンシブUI

## 技術スタック

- **UI**: egui + eframe + wgpu-web
- **Core**: Rust（Nostr I/O、購読管理、送信キュー）
- **Storage**: IndexedDB（rexie）
- **Nostr**: rust-nostr v0.43

## セットアップ

### 必要な環境

- Rust (stable)
- wasm-pack
- Node.js（開発サーバー用）

### インストール

```bash
# リポジトリをクローン
git clone <repository-url>
cd rustr

# wasm32ターゲットを追加
rustup target add wasm32-unknown-unknown

# wasm-packをインストール
cargo install wasm-pack
```

### WASMビルド

**注意**: macOSではsecp256k1のビルドに問題がある場合があります。
その場合は、GitHub Actionsでビルドするか、Linuxマシン/Dockerを使用してください。

```bash
# WASMをビルド
./scripts/build-wasm.sh

# または手動で
cd core
wasm-pack build --target web --out-dir ../ui/pkg
```

### 開発

```bash
# 開発サーバーを起動（自動ビルド含む）
./scripts/dev-server.sh

# または手動で
cd ui/pkg
python3 -m http.server 8080
# または
npx serve
```

http://localhost:8080 でアプリケーションが起動します。

### テスト

#### ユニットテスト

```bash
# ユニットテストを実行
./scripts/test.sh

# または手動で
cd core
cargo test --lib --target aarch64-apple-darwin
cargo test --test integration_test --target aarch64-apple-darwin
```

**注意**: WASM環境でのテストは`wasm-pack test --headless --chrome`で実行できますが、
`secp256k1`のビルド問題により、現在はネイティブターゲットでのテストのみサポートしています。

#### デバッグテストモード（自動E2Eテスト）

アプリ起動時に自動でテストシナリオを実行するデバッグモードを提供しています：

```bash
# デバッグテスト機能を有効にしてビルド
./scripts/build-wasm-debug.sh

# 開発サーバーを起動
cd ui/pkg
python3 -m http.server 8080

# ブラウザで開く（URLパラメータでデバッグモード有効化）
# http://localhost:8080/?debug_test=1
```

デバッグモードでは以下のシナリオが自動実行されます：
1. オンボーディングをスキップ（新規キー生成）
2. メイン画面に遷移
3. テストチャンネルを開く
4. テストメッセージを送信
5. タイムラインを確認

実行状態は画面上部に黄色いバーで表示されます。

## プロジェクト構造

```
rustr/
├── core/           # コアロジック（WASM）
│   ├── src/
│   │   ├── lib.rs
│   │   ├── types.rs
│   │   ├── storage/
│   │   ├── relay.rs
│   │   ├── subscription.rs
│   │   ├── outbox.rs
│   │   └── signer/
│   └── Cargo.toml
├── ui/             # egui UI
│   ├── src/
│   │   ├── main.rs
│   │   ├── app.rs
│   │   ├── timeline.rs
│   │   ├── composer.rs
│   │   └── onboarding.rs
│   ├── index.html
│   └── Cargo.toml
├── scripts/
│   └── build-wasm.sh
└── Cargo.toml      # workspace
```

## トラブルシューティング

### macOSでのビルドエラー

macOSのApple clangはWASMターゲットをサポートしていない場合があります。

解決策：
1. GitHub Actionsでビルド（推奨）
2. Dockerを使用してLinux環境でビルド
3. Linuxマシンでビルド

### ビルドエラー: `secp256k1-sys`

```
error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-unknown"'
```

これはシステムのclangがWASMターゲットをサポートしていないためです。
GitHub ActionsまたはLinux環境でビルドしてください。

## ライセンス

MIT License

## 参考プロジェクト

- [NostrShrine](https://github.com/kojira/NostrShrine) - rust-nostr WASMの実装例
- [rust-nostr](https://github.com/rust-nostr/nostr) - Nostrライブラリ

