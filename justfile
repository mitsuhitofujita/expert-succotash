# justfile for expert-succotash project
# タスクランナーの設定ファイル

# デフォルトタスク: 利用可能なコマンドを表示
default:
    @just --list

# コンパイルチェック（ビルドせずに型チェックのみ）
check:
    cd apps/api && cargo check

# APIサーバーのビルド
build:
    cd apps/api && cargo build

# APIサーバーのリリースビルド
build-release:
    cd apps/api && cargo build --release

# APIサーバーの起動
run:
    cd apps/api && cargo run

# 開発サーバーの起動（ホットリロード付き）
dev:
    cd apps/api && cargo watch -x run

# データベース接続確認（開発用DB）
check-db:
    cargo run -p check-db

# データベース接続確認（テスト用DB）
check-db-test:
    DATABASE_URL="${TEST_DATABASE_URL}" cargo run -p check-db

# データベースのテーブル一覧を表示
list-tables:
    cargo run -p list-tables

# データベースの作成
db-create:
    cd apps/api && sqlx database create

# データベースの削除
db-drop:
    cd apps/api && sqlx database drop -y

# マイグレーションの実行
db-migrate:
    cd apps/api && sqlx migrate run --source db/migrations

# マイグレーションのロールバック（最後の1つ）
db-migrate-revert:
    cd apps/api && sqlx migrate revert --source db/migrations

# 新しいマイグレーションファイルの作成
db-migrate-add name:
    cd apps/api && sqlx migrate add {{name}} --source db/migrations

# データベースのリセット（削除 → 作成 → マイグレーション）
db-reset:
    #!/usr/bin/env bash
    set -euo pipefail
    PROJECT_ROOT="{{justfile_directory()}}"
    cd "$PROJECT_ROOT/apps/api"
    echo "Dropping database..."
    sqlx database drop -y || true
    echo "Creating database..."
    sqlx database create
    echo "Running migrations..."
    sqlx migrate run --source db/migrations
    echo "Database reset completed successfully"

# sqlxのオフラインモードデータを生成（マイグレーション変更時やクエリ追加時に実行）
sqlx-prepare:
    cd apps/api && cargo sqlx prepare

# sqlxのオフラインモードデータが最新か確認（CI用）
sqlx-check:
    cd apps/api && cargo sqlx prepare --check

# APIサーバーの起動確認（ビルド → 起動 → ヘルスチェック）
check-server:
    #!/usr/bin/env bash
    set -euo pipefail
    PROJECT_ROOT="{{justfile_directory()}}"
    cd "$PROJECT_ROOT"
    echo "Building API server..."
    cd apps/api && cargo build
    echo "Starting API server in background..."
    cargo run &
    SERVER_PID=$!
    echo "Server PID: $SERVER_PID"
    echo "Waiting for server to start..."
    sleep 3
    echo "Testing health endpoint..."
    if curl -f http://localhost:3000/health; then
        echo -e "\n✓ Health check passed"
        RESULT=0
    else
        echo -e "\n✗ Health check failed"
        RESULT=1
    fi
    echo "Stopping server..."
    kill $SERVER_PID || true
    wait $SERVER_PID 2>/dev/null || true
    exit $RESULT

# コードフォーマット（ワークスペース全体）
fmt:
    cargo fmt --all

# コードフォーマットのチェック（ワークスペース全体）
fmt-check:
    cargo fmt --all --check

# リンターの実行（ワークスペース全体）
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery

# リンターの監視モード（ファイル変更時に自動実行）
watch-lint:
    cd apps/api && cargo watch -x 'clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery'

# テストの実行（ワークスペース全体）
test:
    cargo test --workspace

# テストの監視モード（ファイル変更時に自動実行）
watch-test:
    cd apps/api && cargo watch -x test

# すべてのチェックを実行（フォーマット、リント、テスト、ビルド）
ci: fmt-check lint test build
    @echo "All checks passed!"

# 開発環境のクリーンアップ
clean:
    cd apps/api && cargo clean

# 依存関係の更新確認
outdated:
    cd apps/api && cargo outdated
