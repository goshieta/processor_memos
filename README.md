# Memos Webhookプロセッサー
OSSであるmemosのwebhookを受け取り、データの処理サーバーに送るための中継サーバー。

処理サーバーは以下のようなスキーマで動作している。
```json
{
  "source": "memos", //MEMOSから流すのでmemosで値は固定
  "content": "生テキスト..." //新しく作成されたメモの内容をそのまま流し込む
}
```

memosのwebhookのスキーマは以下の通り
```json
{
  "url": "https://your-webhook-endpoint.com",
  "activityType": "memos.memo.created",
  "creator": "users/username",
  "memo": {
    "name": "memos/memoid",
    "state": 1,
    "creator": "users/stardust5905",
    "create_time": {"seconds": 1784339156},
    "update_time": {"seconds": 1784339156},
    "content": "ここにメモの本文（Markdown）が入ります",
    "visibility": 1,
    "property": {},
    "snippet": "ここにメモの本文のプレビューが入ります"
  }
}
```

## 技術構成
- 言語: Rust
- Webフレームワーク: actix-web 4
- HTTPクライアント: reqwest 0.12

## 要件
- 処理サーバーのアドレスやこのサーバーのポートなどは環境変数で変更できるようにする。

## セットアップ

```bash
# ビルド
cargo build

# 環境変数の設定
export PROCESSING_SERVER_URL="http://your-processing-server/api"  # 必須
export LISTEN_ADDR="0.0.0.0:8080"  # 任意（デフォルト: 0.0.0.0:8080）

# 起動
cargo run
```

## 環境変数

| 変数名 | 必須 | デフォルト値 | 説明 |
|---|---|---|---|
| `PROCESSING_SERVER_URL` | 必須 | なし | 処理サーバーのURL |
| `LISTEN_ADDR` | 任意 | `0.0.0.0:8080` | このサーバーのバインドアドレス |

## API

### `POST /webhook`

Memosのwebhookを受け取るエンドポイント。

**リクエストボディ:** Memos webhookスキーマ（上記参照）

**レスポンス:**
- `200 OK` - 処理サーバーへの転送に成功
- `502 Bad Gateway` - 処理サーバーへの転送に失敗