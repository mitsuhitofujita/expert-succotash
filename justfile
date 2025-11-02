# justfile for expert-succotash project
# タスクランナーの設定ファイル

# デフォルトタスク: 利用可能なコマンドを表示
default:
    @just --list

# APIサーバーのビルド
build:
    cd apps/api && cargo build

# APIサーバーのリリースビルド
build-release:
    cd apps/api && cargo build --release

# APIサーバーの起動
run:
    cd apps/api && cargo run

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

# コードフォーマット
fmt:
    cd apps/api && cargo fmt

# コードフォーマットのチェック
fmt-check:
    cd apps/api && cargo fmt --check

# リンターの実行
lint:
    cd apps/api && cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery

# テストの実行
test:
    cd apps/api && cargo test

# すべてのチェックを実行（フォーマット、リント、テスト、ビルド）
ci: fmt-check lint test build
    @echo "All checks passed!"

# 開発環境のクリーンアップ
clean:
    cd apps/api && cargo clean

# 依存関係の更新確認
outdated:
    cd apps/api && cargo outdated
