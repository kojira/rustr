# UI

egui製のNostrクライアントUI。

## 責務

- **アプリケーションUI**: egui + eframeを使用したWebアプリケーション
- **タイムライン**: メッセージ表示、遅延描画、行キャッシュ
- **Composer**: メッセージ入力、IME対応
- **オンボーディング**: 初回起動時の鍵生成/インポート

## ビルド

```bash
wasm-pack build --target web ui
```

## 開発サーバー

```bash
basic-http-server ui/pkg/
```

