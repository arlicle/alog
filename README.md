# alog 项目说明

## 项目概述

**alog** 是一个用 Rust 编写的快速静态博客生成器，专注于简洁性和开发体验。该项目的核心功能包括：

- 将 Markdown 文件渲染为静态 HTML 网站
- 支持热更新（文件监控自动重新构建）
- 支持自定义主题和样式
- 灵活的发布时间配置
- 内置开发服务器
- 支持 Frontmatter 元数据
- 支持 RSS 订阅
- 支持标签和分类

## 技术栈

### 核心依赖

- **Rust 2021 Edition** - 主要编程语言
- **clap 4.5** - 命令行参数解析（支持 derive 宏）
- **tokio 1.40** - 异步运行时
- **axum 0.8** - Web 框架，用于开发服务器
- **tower-http 0.5** - HTTP 服务扩展（文件系统支持、追踪）
- **pulldown-cmark 0.13** - Markdown 解析器
- **notify 7.0** - 文件系统监控（热更新）
- **askama 0.12** - 模板引擎
- **chrono 0.4** - 日期时间处理
- **serde** + **toml** + **serde_yaml** - 序列化/反序列化
- **rss 2.0** - RSS feed 生成
- **tracing** - 结构化日志

### 架构

项目采用模块化设计，主要模块包括：

```
src/
├── main.rs          # 应用入口
├── lib.rs           # 库导出
├── cli.rs           # CLI 命令定义
├── config.rs        # 配置管理
├── parser/          # Markdown 解析模块
│   ├── frontmatter.rs   # Frontmatter 解析
│   ├── markdown.rs      # BlogPost 结构定义
│   └── mod.rs
├── renderer/        # 渲染模块
│   ├── html.rs          # HTML 生成
│   ├── template.rs      # 模板渲染
│   ├── rss.rs           # RSS feed 生成
│   └── mod.rs
├── server/          # Web 服务器
│   ├── serve.rs         # 服务器启动
│   └── mod.rs
└── watcher/         # 文件监控
    ├── file_watcher.rs  # 文件变更监控
    └── mod.rs
```

## 构建和运行

### 构建项目

```bash
cargo build --release
```

### 运行命令

#### 构建静态网站

```bash
# 使用默认配置（输入目录: ./md, 输出目录: ./www）
cargo run -- build

# 自定义输入输出目录
cargo run -- build --input-dir ./my-posts --output-dir ./output
```

#### 启动开发服务器（支持热更新）

```bash
# 使用默认端口 7878
cargo run -- serve

# 自定义端口
cargo run -- serve --port 3000

# 自定义输入输出目录
cargo run -- serve --input-dir ./md --output-dir ./www --port 8080
```

### 运行测试

```bash
cargo test
```

### 检查代码质量

```bash
# 格式检查
cargo fmt --check

# Clippy 检查
cargo clippy

# 类型检查
cargo check
```

## 配置说明

项目使用 TOML 格式的配置文件（默认 `config.toml`），配置结构：

```toml
# alog 配置文件
# 请根据需要修改以下配置项

# 输入目录：存放 Markdown 文章的目录
input_dir = "./md"

# 输出目录：生成的静态网站文件存放目录
output_dir = "./www"

# 服务器配置
[server]
    # 端口号：开发服务器监听的端口
    port = 7878

    # 主机地址：开发服务器绑定的地址
    # "0.0.0.0" 表示监听所有网络接口
    # "127.0.0.1" 表示仅本地访问
    host = "0.0.0.0"

# 主题配置
[theme]
    # 主题名称：使用的主题文件夹名称
    # 主题文件应放置在 theme/ 目录下
    name = "default"

    # 自定义 CSS 文件路径（可选）
    # 如果需要自定义样式，可以指定 CSS 文件路径
    # 例如: "custom.css" 或 "./theme/my-theme/style.css"
    # custom_css = "custom.css"

# 分页配置
[pagination]
    # 每页显示的文章数量
    posts_per_page = 15
```

## 开发约定

### 文件命名约定

Markdown 文件支持两种命名方式：

1. **文件名包含日期**（推荐）
   - 格式: `YYYY-MM-DD-标题.md`
   - 示例: `2024-01-15-Hello-World.md`
   - 也支持精确时间: `YYYYMMDDHHmm-标题.md`

2. **目录结构包含日期**
   - 目录结构: `md/YYYY/MM/文件名.md`
   - 示例: `md/2024/01/my-post.md`
   - 日期会从目录结构中提取（设置为当月1号）

3. **Frontmatter 指定日期**
   - 在文件头部使用 YAML frontmatter 指定

### Frontmatter 格式

Markdown 文件支持 YAML frontmatter，格式如下：

```markdown
---
title: 文章标题
date: 2024-01-15
category: 分类
tags: [标签1, 标签2, 标签3]
summary: 文章摘要
---

文章正文内容...
```

### 模板系统

使用 Askama 模板引擎，模板文件位于 `templates/` 目录：
- `index.html` - 首页模板
- `post.html` - 文章详情模板
- `index_pagination.html` - 首页分页模板
- `category.html` - 分类页面模板
- `category_pagination.html` - 分类分页模板
- `tag.html` - 标签页面模板
- `tags.html` - 标签列表模板

### 代码风格

- 使用 Rust 标准格式化: `cargo fmt`
- 遵循 Clippy 建议: `cargo clippy`
- 错误处理使用 `anyhow::Result` 统一错误类型
- 日志使用 `tracing` crate

### 日志配置

使用环境变量控制日志级别：

```bash
# 开发环境：详细信息
RUST_LOG=debug cargo run -- serve

# 生产环境：仅错误
RUST_LOG=error cargo run -- serve
```

## 主题自定义

主题放置在 `theme` 文件夹中，可以通过复制 default 主题并修改来自定义。启用前需要在 `config.toml` 中配置主题名称。

## 输出结构

构建后的网站结构：

```
www/
├── index.html
├── index-page-2.html
├── post/
│   └── [文章slug].html
├── category/
│   └── [分类名].html
├── tag/
│   └── [标签名].html
├── tags.html
└── rss.xml
```

## 常见任务

### 添加新文章

1. 在 `md/` 目录下创建 Markdown 文件
2. 使用推荐的命名格式: `YYYY-MM-DD-标题.md`
3. 添加 frontmatter 元数据（可选）
4. 使用 `cargo run -- serve` 启动开发服务器，会自动重新构建

### 修改主题

1. 复制 `theme/default/` 到新主题文件夹
2. 修改模板文件和样式
3. 在 `config.toml` 中更新主题名称

### 部署到 GitHub Pages

构建完成后，将 `www/` 目录内容推送到 `gh-pages` 分支或配置为 GitHub Pages 源目录。

## 注意事项

- 项目目前处于开发阶段，API 可能会有变化
- 文件监控功能在 serve 模式下会自动重新构建
- 所有日期最终转换为 `chrono::NaiveDate` 格式
- 文章按发布时间降序排列（最新的在前）