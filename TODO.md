# TODO リスト

## ✅ 完了した機能（v0.1.0）

### Core Module

#### 1. CoreHandle API実装
- [x] `init()` - 初期化処理
- [x] `connect_all()` - 全Relay接続
- [x] `open_channel()` - チャンネル購読のRelay送信
- [x] `open_dm()` - DM購読のRelay送信
- [x] `send_public()` - イベント作成・署名・送信
- [x] `send_dm()` - NIP-04暗号化・イベント作成・署名・送信
- [x] `poll_events()` - 受信イベントの取得
- [x] `tick()` - 定期処理（Relay再接続、メッセージ処理、Outbox処理）

#### 2. RelayConnection
- [x] WebSocketイベントハンドラー実装
  - [x] `onopen` - 接続確立時の処理
  - [x] `onmessage` - メッセージ受信処理（キューイング）
  - [x] `onerror` - エラーハンドリング
  - [x] `onclose` - 切断時の処理
- [x] 指数バックオフ再接続
- [x] メッセージキュー管理

#### 3. Signer実装
- [x] **InternalSigner**: nostrクレートを使った鍵ペア生成
- [x] **InternalSigner**: nostrクレートを使った署名
- [x] **InternalSigner**: NIP-04暗号化/復号化
- [x] **Nip07Signer**: 基本実装
- [x] **Nip07Signer**: NIP-04暗号化/復号化（window.nostr.nip04経由）

#### 4. Storage
- [x] IndexedDB実装（rexie）
- [x] フィルター適用（kinds, authors, since, until）
- [x] イベント保存・取得
- [x] Outboxキュー管理
- [x] 鍵ペア保存

#### 5. SubscriptionManager
- [x] チャンネル購読（NIP-28）
- [x] DM購読（NIP-04）
- [x] EOSE処理
- [x] ウィンドウ拡張（段階的: 10分→1時間→1日→7日→30日）

#### 6. OutboxQueue
- [x] 送信キュー管理
- [x] 再送処理
- [x] NIP-20 OK応答処理

### UI Module

#### 7. Onboarding
- [x] Welcome画面
- [x] Signer選択（NIP-07 / Import / Create）
- [x] 鍵のインポート処理
- [x] 鍵の生成処理
- [x] CoreHandle初期化

#### 8. Timeline
- [x] 実データ表示（poll_events経由）
- [x] リアルタイム更新（tick()ループ）
- [x] イベント重複チェック
- [x] 時系列ソート
- [x] 時刻フォーマット（相対時間）

#### 9. App
- [x] UIとCoreの統合（Rc<RefCell<>>）
- [x] Relay接続管理
- [x] チャンネル購読
- [x] メッセージ送信
- [x] tick()定期実行
- [x] エラーハンドリング

## 🟡 今後の改善（v0.2.0+）

### 機能追加

#### 1. UI/UX改善
- [ ] DM一覧表示
  - [ ] StorageからDMスレッド取得
  - [ ] スレッド選択UI
  - [ ] 未読カウント表示
- [ ] 設定画面
  - [ ] Relay管理（追加・削除）
  - [ ] 鍵のエクスポート
  - [ ] テーマ切り替え
- [ ] プロフィール表示（NIP-01）
- [ ] リアクション（NIP-25）
- [ ] メンション通知
- [ ] 画像表示・アップロード（NIP-94）

#### 2. パフォーマンス最適化
- [ ] 仮想スクロール（大量イベント対応）
- [ ] IndexedDBインデックス最適化
- [ ] イベントキャッシュ戦略
- [ ] バッチ処理

#### 3. プロトコル対応拡張
- [ ] NIP-05（DNS認証）
- [ ] NIP-10（スレッド）
- [ ] NIP-25（リアクション）
- [ ] NIP-42（認証）
- [ ] NIP-94（ファイルメタデータ）

#### 4. その他
- [ ] オフライン対応
- [ ] PWA対応（将来的に除外予定だが検討）
- [ ] マークダウン対応
- [ ] 絵文字ピッカー
- [ ] プッシュ通知

## 📊 現在の実装状況

### ✅ 完全実装（動作確認済み）
- ✅ プロジェクト構造（2クレート構成）
- ✅ Storage抽象化（IndexedDB実装）
- ✅ Signer抽象化（NIP-07 + Internal）
- ✅ Relay接続管理（WebSocket + 自動再接続）
- ✅ Subscription管理（EOSE + ウィンドウ拡張）
- ✅ Outbox送信キュー
- ✅ UI完全実装（オンボーディング、タイムライン、コンポーザー）
- ✅ WASMビルド環境
- ✅ 実際のNostr通信
- ✅ 実際のイベント送受信
- ✅ NIP-04暗号化/復号化
- ✅ 鍵の生成・インポート
- ✅ リアルタイム更新

### 🎯 対応プロトコル
- ✅ NIP-01（基本プロトコル）
- ✅ NIP-04（暗号化DM）
- ✅ NIP-07（ブラウザ拡張）
- ✅ NIP-20（OK応答）
- ✅ NIP-28（パブリックチャット）

## 🚀 デプロイ状況

- ✅ WASMビルド成功
- ✅ 開発サーバー動作確認
- ✅ GitHubリポジトリ: https://github.com/kojira/rustr
- 📝 本番デプロイ: 未実施（GitHub Pages / Vercel / Cloudflare Pages等で可能）

## 📝 備考

**v0.1.0時点のアプリケーション状態：**
- ✅ UIは完全に動作
- ✅ WASMビルドは成功
- ✅ 実際のNostr通信が動作
- ✅ 暗号化機能が動作
- ✅ 実用レベルのNostrクライアントとして機能

**次のマイルストーン（v0.2.0）:**
- DM一覧表示
- 設定画面
- プロフィール表示
- パフォーマンス最適化

---

**最終更新: 2025-01-20**
**バージョン: 0.1.0**
**ステータス: 実用レベル達成 🎉**
