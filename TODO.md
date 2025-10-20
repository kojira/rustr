# TODO リスト

## 🔴 重要度：高（動作に必須）

### Core Module

#### 1. CoreHandle API実装
- [ ] `init()` - 初期化処理
- [ ] `open_channel()` - チャンネル購読のRelay送信
- [ ] `open_dm()` - DM購読のRelay送信
- [ ] `send_public()` - イベント作成・署名・送信
- [ ] `send_dm()` - NIP-04暗号化・イベント作成・署名・送信
- [ ] `poll_events()` - 受信イベントの取得
- [ ] `tick()` - 定期処理（Relay再接続、Outbox処理）

#### 2. RelayConnection
- [ ] WebSocketイベントハンドラー実装
  - [ ] `onopen` - 接続確立時の処理
  - [ ] `onmessage` - メッセージ受信処理
  - [ ] `onerror` - エラーハンドリング
  - [ ] `onclose` - 切断時の処理
- [ ] 指数バックオフ再接続
- [ ] 購読の再送

#### 3. Signer実装
- [ ] **InternalSigner**: nostrクレートを使った鍵ペア生成
- [ ] **InternalSigner**: nostrクレートを使った署名
- [ ] **InternalSigner**: NIP-04暗号化/復号化
- [ ] **Nip07Signer**: NIP-04暗号化/復号化

#### 4. Storage
- [ ] フィルター適用の完全実装（現状は全件取得）

## 🟡 重要度：中（機能拡張）

### UI Module

#### 5. Onboarding
- [ ] NIP-07の可用性チェック
- [ ] 鍵のインポート処理
- [ ] 鍵の生成処理
- [ ] Storageへの保存

#### 6. Timeline
- [ ] CoreHandle経由での実際のイベント取得
- [ ] リアルタイム更新
- [ ] 画像表示
- [ ] リンクプレビュー
- [ ] 遅延ロード

#### 7. App
- [ ] Storageから鍵の有無を確認
- [ ] CoreHandleの初期化と統合
- [ ] DM一覧表示
- [ ] 設定画面

## 🟢 重要度：低（最適化・改善）

#### 8. その他
- [ ] 時刻フォーマットの改善
- [ ] エラーメッセージの改善
- [ ] ローディング状態の表示
- [ ] オフライン対応

## 📊 現在の実装状況

### ✅ 完了
- プロジェクト構造（2クレート構成）
- Storage抽象化（IndexedDB実装）
- Signer抽象化（基本構造）
- Relay基本構造
- Subscription基本構造
- Outbox基本構造
- UI基本構造（オンボーディング、タイムライン、コンポーザー）
- WASMビルド環境

### 🚧 部分実装（モック含む）
- CoreHandle API（構造のみ、実装なし）
- RelayConnection（接続のみ、イベントハンドラーなし）
- Signer（基本構造のみ、実際の暗号化なし）
- Timeline（デモデータ表示のみ）

### ❌ 未実装
- 実際のRelay通信
- 実際のイベント送受信
- NIP-04暗号化/復号化
- 鍵の生成・インポート
- リアルタイム更新

## 🎯 次のステップ（優先順位順）

1. **Signerの完全実装**（鍵生成、署名、NIP-04）
2. **RelayConnectionのイベントハンドラー**
3. **CoreHandle APIの実装**
4. **UIとCoreの統合**
5. **実際のRelay接続テスト**

## 📝 備考

現在のアプリケーションは：
- ✅ UIは完全に動作（デモデータ）
- ✅ WASMビルドは成功
- ⚠️ 実際のNostr通信は未実装
- ⚠️ 暗号化機能は未実装

**実用的なNostrクライアントとして動作させるには、上記のTODOを実装する必要があります。**

