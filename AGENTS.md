# sqlfmt — 智能体指令

## 项目概述

- Go SQL 格式化器，PostgreSQL 方言（通过 `cockroachdb/cockroachdb-parser`）
- 模块：`github.com/madelynnblue/sqlfmt`
- 基于 Wadler 的 "pretty printer" 论文 — 宽度感知布局与对齐

## 目录边界

| 路径 | 角色 |
|---|---|
| `sqlfmt.go` | 核心库 — `FmtSQL(cfg, stmts)` 和 `FmtJSON(s)` |
| `backend/main.go` | CLI + HTTP 服务器二进制文件（单一入口，两种模式） |
| `wasm/main.go` | 浏览器 WebAssembly 构建（`docs/` 站点） |
| `docs/` | 静态站点 + 预构建的 WASM 产物 |
| `tests/` | 仅 SQL 固定文件 — **不存在 Go 单元测试** |

## 开发者命令

```bash
# CLI — 从 stdin 格式化
cd backend && go run . < input.sql

# CLI — 格式化特定语句
cd backend && go run . --stmt "SELECT 1" --stmt "SELECT 2"

# CLI — 带选项格式化
cd backend && go run . --print-width 80 --casemode lower --align

# HTTP 服务器模式（设置环境变量）
SQLFMT_ADDR=":8080" go run ./backend

# 为 docs 站点构建 WASM
cd wasm && GOOS=js GOARCH=wasm go build -o sqlfmt.wasm && cp sqlfmt.wasm ../docs/

# 构建静态 musl 二进制文件
cd backend && CGO_ENABLED=1 CC=gcc go build \
  -buildvcs=false \
  -ldflags="-linkmode external -extldflags '-static' -X main.version=dev -X main.commit=none -X main.date=unknown"

# 发布（需要 goreleaser + GITHUB_TOKEN）
cd backend && goreleaser release --snapshot   # 预演
cd backend && goreleaser release              # 正式发布
```

## CLI 标志

`--print-width`（默认 80）、`--use-spaces`、`--tab-width`（默认 4）、
`--casemode`（upper|lower|title|spongebob）、`--no-simplify`、`--align`、
`--stmt`、`-h`、`-v`

## 架构特性

- **TEXT 类型恢复**：CockroachDB 解析器将 `TEXT` 转换为 `STRING`。库会在格式化后跟踪并恢复 TEXT（`sqlfmt.go:22-56`）。
- **WITH 和 DISTRIBUTED BY 子句处理**：这些子句在解析前被剥离并保存，格式化后重新插入（`sqlfmt.go:75-215`）。
  - **WITH 子句压缩**：自动尝试将 WITH (...) 压缩为单行。如果压缩后长度超过 `print-width`，保持原始多行格式。
  - **DISTRIBUTED BY 放置**：始终单独占一行。
  - **智能提取存储**：为每个 CREATE TABLE 语句分别存储 WITH（压缩和原始两个版本）和 DISTRIBUTED BY 子句。
- **HTTP 服务器双模式**：如果设置了 `SQLFMT_ADDR` 环境变量 → HTTP 服务器；否则 → CLI。同一二进制文件。
- **HTTP `/fmt` 端点**接受查询参数：`sql`、`n`（行宽）、`indent`、`simplify`、`align`、`case`、`spaces`、`json`。
- **响应缓存**：内存 LRU 缓存（最多 10k 条目），以原始查询字符串为键（`backend/main.go:221-263`）。
- **JSON 回退**：如果 SQL 解析失败，服务器会尝试将输入作为 JSON 格式化（`backend/main.go:309-311`）。

## 测试

- **不存在 `_test.go` 文件。** 验证是手动的：通过二进制文件管道传输 SQL 并检查输出。
- `tests/` 包含 SQL 固定文件（如 `test-distributed-by.sql` 及其对应的 `.expected` 预期文件）用于手动测试。
- 格式化验证：`./sqlfmt < tests/test-distributed-by.sql | diff - tests/test-distributed-by.expected`

## 版本管理

- 打标签：`git tag v0.X.0 && git push origin v0.X.0`
- 版本/提交/日期在构建时通过 ldflags 注入
- GoReleaser 配置位于 `backend/.goreleaser.yaml`，目标平台为 linux/darwin/windows amd64
