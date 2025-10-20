# Rustr - 開発進捗

## ✅ 完了した項目

### 1. プロジェクト構造 (2クレート構成)
- ✅ `core/` - コアロジック（WASM）
- ✅ `ui/` - egui UI
- ✅ Cargo workspace設定

### 2. Core Module
- ✅ エラー処理 (`CoreError`)
- ✅ Storage抽象化 (`Storage` trait)
  - ✅ IndexedDB実装 (`IndexedDbStorage`)
  - ✅ Mock実装 (`MockStorage`)
- ✅ Signer抽象化 (`Signer` trait)
  - ✅ NIP-07実装 (`Nip07Signer`)
  - ✅ Internal実装 (`InternalSigner` with WebCrypto)
- ✅ Relay接続 (`RelayConnection`)
- ✅ 購読管理 (`SubscriptionManager`)
- ✅ 送信キュー (`OutboxQueue`)
- ✅ 型定義 (`types.rs`)

### 3. UI Module
- ✅ メインアプリ構造 (`NostrApp`)
- ✅ タイムライン (`Timeline`)
- ✅ コンポーザー (`Composer`)
- ✅ オンボーディング (`Onboarding`)
- ✅ index.html (CSP設定済み)

### 4. ビルド環境
- ✅ WASM build成功
- ✅ Homebrew LLVM使用 (`secp256k1`対応)
- ✅ wasm-pack設定
- ✅ 開発サーバースクリプト

## 🚀 動作確認

### ビルド
```bash
./scripts/build-wasm.sh
```

### 開発サーバー起動
```bash
./scripts/dev-server.sh
```

アクセス: http://localhost:8080

## 📊 ビルド結果
- **エラー**: 0個 ✅
- **警告**: 9個（未使用変数、dead_code）
- **WASMサイズ**: 約14KB (core) + UI
- **ビルド時間**: 約1.5秒

## 🔧 技術スタック

### Frontend
- **UI Framework**: egui + eframe
- **Rendering**: wgpu-web
- **Language**: Rust (WASM)

### Backend (Core)
- **Nostr**: rust-nostr v0.43
- **Storage**: IndexedDB (rexie)
- **Crypto**: WebCrypto API (PBKDF2, AES-GCM)
- **WebSocket**: web-sys

### Dependencies
- `nostr = "0.43"` (default-features = false, features = ["std"])
- `rexie` - IndexedDB
- `eframe = "0.29"` - egui framework
- `wasm-bindgen` - JS interop
- `getrandom` (features = ["js"]) - WASM random

## 📝 実装済み機能

### Core
1. **Storage**
   - IndexedDB (events, dm_threads, last_seen, outbox, keypair)
   - Base64エンコード/デコード
   - フィルタリング（基本実装）

2. **Signer**
   - NIP-07 (window.nostr)
   - Internal (WebCrypto暗号化)
   - 鍵ペア生成・保存・読み込み

3. **Relay**
   - WebSocket接続
   - メッセージパース (EVENT, EOSE, OK, NOTICE)
   - 購読管理

4. **Subscription**
   - 窓管理 (時間窓)
   - EOSE処理
   - 段階的拡大

5. **Outbox**
   - 送信キュー
   - ステータス管理

### UI
1. **オンボーディング**
   - Welcome画面
   - Signer選択 (NIP-07 / Import / Create)
   - 鍵インポート
   - 鍵生成

2. **メインビュー**
   - トップバー (ナビゲーション)
   - タイムライン (スクロール可能)
   - コンポーザー (マルチライン入力)

3. **タイムライン**
   - イベント表示
   - 時刻表示
   - アクションボタン (Reply, Like)

4. **コンポーザー**
   - テキスト入力
   - 送信ボタン
   - Ctrl+Enter送信

## 🔜 未実装機能 (TODO)

### Core
1. **CoreHandle API**
   - init()
   - open_channel()
   - open_dm()
   - send()
   - poll_events()
   - tick()

2. **RelayConnection**
   - イベントハンドラー (onopen, onmessage, onerror, onclose)
   - 指数バックオフ再接続
   - 購読の再送

3. **SubscriptionManager**
   - フィルター生成の完全実装
   - 複数購読の管理

4. **OutboxQueue**
   - 再送ロジック
   - NIP-20 OK処理

5. **NIP-04暗号化**
   - 実際の暗号化/復号化実装

### UI
1. **CoreHandleとの統合**
   - 実際のイベント取得
   - 実際のメッセージ送信
   - リアルタイム更新

2. **タイムライン拡張**
   - 画像表示
   - リンクプレビュー
   - 遅延ロード
   - 差分更新

3. **設定画面**
   - Relay設定
   - プロフィール編集

4. **DM一覧**
   - スレッド一覧
   - 未読管理

## 🐛 既知の問題

1. **警告**
   - 未使用変数 (CoreHandle内)
   - dead_code (未実装フィールド)

2. **macOS環境**
   - Apple clangではWASMビルド不可
   - Homebrew LLVMが必要

## 📚 ドキュメント

- [README.md](README.md) - セットアップとトラブルシューティング
- [PROGRESS.md](PROGRESS.md) - このファイル

## 🎯 次のステップ

1. CoreHandle APIの完全実装
2. UIとCoreの統合
3. 実際のRelay接続テスト
4. NIP-04暗号化の実装
5. テストの追加

