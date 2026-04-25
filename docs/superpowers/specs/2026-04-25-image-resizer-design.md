# ImageResizer - 漫画图片批量压缩工具设计文档

> 日期: 2026-04-25
> 状态: 已批准

## 1. 项目概述

批量压缩本地漫画图片的桌面 GUI 工具。支持多级子目录递归扫描，提供大小调整、品质控制、格式转换、分辨率设置等功能，具备多配置方案管理和实时处理进度反馈。目标是高速、高效、高并发的图片批处理。

### 技术栈

| 层级 | 技术 |
|------|------|
| GUI 框架 | Tauri (Rust 后端 + 系统 WebView) |
| 前端 | Svelte + TypeScript |
| 图片引擎 | Rust `image` crate |
| 并发处理 | `rayon` (自动检测 CPU 核心数) |
| 目录遍历 | `walkdir` crate |
| 配置存储 | JSON 文件 (`%APPDATA%/ImageResizer/profiles.json`) |

### 支持格式

- JPEG (.jpg/.jpeg)
- PNG (.png)
- WebP (.webp)
- GIF (.gif, 含动图)

---

## 2. 架构设计

### 模块划分

```
Svelte + TypeScript (前端)
├── FileBrowser        目录选择、文件树/列表、格式筛选
├── SettingsPanel      配置方案管理、参数设置
└── ProgressPanel      进度条、文件明细、统计信息

Tauri Commands (IPC 层)
├── scan_directory     扫描目录返回文件列表
├── get_profiles       获取所有配置方案
├── save_profile       保存配置方案
├── delete_profile     删除配置方案
├── start_processing   启动批处理
├── stop_processing    停止批处理

Rust 后端 (Tauri Core)
├── FileScanner        递归扫描目录，过滤图片，返回元信息
├── ImageProcessor     核心处理引擎 (decode → resize → encode)
├── ConfigManager      多配置方案 CRUD，JSON 持久化
└── ProgressTracker    通过 Tauri Events 推送实时进度
```

### 数据流

```
[Svelte 前端]
    │
    ├── Command: scan_directory(path)         → 文件列表
    ├── Command: get_profiles()               → 配置方案列表
    ├── Command: save_profile(profile)        → 保存方案
    ├── Command: delete_profile(name)         → 删除方案
    ├── Command: start_processing(config)      → 触发处理
    │
    ├── Event: progress_update(payload)        ← 实时进度
    │   { total, current, file, original_size, new_size, status }
    │
    └── Event: processing_complete(result)     ← 完成通知
        { total_files, success, failed, failed_files, total_saved_bytes }
```

---

## 3. UI 布局设计

单窗口应用，左右分栏布局：

```
┌──────────────────────────────────────────────────────────┐
│  ImageResizer                                     — □ ×  │
├────────────────────┬─────────────────────────────────────┤
│  源目录            │  配置方案: [常用 ▾] [+] [✎] [✕]     │
│  [📁 选择目录...]  ├─────────────────────────────────────┤
│                    │  ── 调整大小 ──────────────────────  │
│  ┌──────────────┐  │  宽度: [100] [百分比 ▾]   ℹ          │
│  │ 📁 vol1      │  │  高度: [100] [百分比 ▾]   ℹ          │
│  │ 📁 vol2      │  │  模式: [合适大小 ▾]         ℹ          │
│  │ 📄 001.jpg   │  │  ☑ 保持宽高比                     │
│  │ 📄 002.jpg   │  ├─────────────────────────────────────┤
│  │ 📄 003.png   │  │  ── 输出设置 ──────────────────────  │
│  │ ...          │  │  操作: [重新调整 ▾]         ℹ       │
│  │ 共 256 个文件 │  │  输出目录: [<与原文件同目录> ▾]      │
│  └──────────────┘  ├─────────────────────────────────────┤
│                    │  ── 品质与格式 ────────────────────  │
│  筛选: [全部 ▾]    │  格式: [与原文件相同 ▾]       ℹ       │
│                    │  ☑ 品质   ○ 大小    □ 保持原品质     │
│                    │  品质值: [40] %                    │
│                    │  ☐ 调整分辨率 [96] DPI              │
│                    ├─────────────────────────────────────┤
│                    │  ── 处理进度 ──────────────────────  │
│                    │  [████████████░░░░░░░] 67% (172/256) │
│                    │  文件            原始    压缩后  节省  │
│                    │  vol1/001.jpg   2.1MB  680KB  68%    │
│                    │  vol1/002.jpg   1.8MB  520KB  71%    │
│                    │  vol1/003.png   3.2MB  1.1MB  66%    │
│                    │  ...                              │
│                    │  总计节省: 1.2 GB                   │
│                    ├─────────────────────────────────────┤
│                    │  [⏹ 停止]              [▶ 开始压缩]  │
└────────────────────┴─────────────────────────────────────┘
```

### UI 组件说明

- **左侧面板**: 源目录选择器 + 文件树/列表 + 格式筛选下拉
- **右侧上部**: 设置面板，分为三个区块
  - 调整大小: 宽高、单位、模式、宽高比
  - 输出设置: 操作模式、输出目录
  - 品质与格式: 格式转换、品质/大小控制、DPI 设置
- **右侧下部**: 处理进度面板
  - 总进度条 (百分比 + 已处理/总数)
  - 文件明细表格 (文件名、原始大小、压缩后大小、节省比例)
  - 统计汇总 (总节省空间)
- **底部**: 停止 / 开始压缩 按钮

---

## 4. 图片处理引擎

### Rust Crate 依赖

| Crate | 用途 |
|-------|------|
| `image` | 图片解码/编码/缩放核心 |
| `rayon` | CPU 并行处理 |
| `walkdir` | 高效递归目录遍历 |
| `serde` / `serde_json` | 配置序列化 |
| `glob` | 文件格式匹配 |

### 处理流水线

```
读取文件 → 解码 → resize (按模式) → 编码 (目标格式+品质) → 写入输出
```

### 缩放模式

| 模式 | 说明 |
|------|------|
| Fit (合适大小) | 等比缩放到目标框内，不超出 |
| Fill (填充) | 等比缩放并裁剪填满目标框 |
| Stretch (拉伸) | 直接拉伸到目标尺寸，不保持比例 |
| ShrinkOnly (仅缩小) | 只缩小不放大，大于目标才缩放 |

### 尺寸单位

- **百分比**: 相对原始尺寸 (如 50% 表示宽高各缩半)
- **像素**: 绝对像素值 (如 1920x1080)

### 品质控制

| 格式 | 品质参数 | 说明 |
|------|---------|------|
| JPEG | quality 1-100 | 标准 JPEG 品质参数 |
| PNG | 压缩级别 0-9 | V1 仅支持无损压缩 |
| WebP | quality 1-100 | 支持 lossy 和 lossless |
| GIF | 调色板颜色数 | 控制颜色数量来控制大小 |

品质控制模式:
- **品质模式**: 用户指定品质百分比 (1-100)
- **目标大小模式**: 用户指定目标文件大小 (KB)，引擎自动计算合适的品质
- **保持原始品质**: 不调整品质，仅做尺寸/格式/DPI 变更

### 并发模型

```
rayon::par_iter() 自动按 CPU 核心数分片
每片独立处理一个文件
通过 channel 将进度事件发送到前端
支持中途停止 (通过 AtomicBool 标志位)
```

### 错误处理

- 单个文件处理失败不中断整体批处理
- 失败文件记录到失败列表
- 处理完成后汇总报告 (成功数、失败数、失败文件列表)
- 支持仅重试失败文件

---

## 5. 配置方案管理

### 数据结构

```rust
struct Profile {
    name: String,
    resize: ResizeSettings,
    output: OutputSettings,
    quality: QualitySettings,
}

struct ResizeSettings {
    width: u32,
    height: u32,
    unit: Unit,             // Percentage | Pixel
    mode: ResizeMode,       // Fit | Fill | Stretch | ShrinkOnly
    keep_aspect_ratio: bool,
}

struct OutputSettings {
    operation: Operation,   // Overwrite | SameDir | CustomDir
    custom_dir: Option<String>,
    format: OutputFormat,   // SameAsOriginal | Jpeg | Png | WebP | Gif
}

struct QualitySettings {
    mode: QualityMode,      // Quality | TargetSize | Original
    quality: u8,            // 1-100
    target_size_kb: Option<u32>,
    adjust_dpi: bool,
    dpi: u32,
}
```

### 存储位置

`%APPDATA%/ImageResizer/profiles.json`

### 内置默认方案

1. **常用**: 宽高 100%，品质 40%，与原文件同格式，同目录输出
2. **高质量**: 宽高 100%，品质 85%，与原文件同格式，自定义目录
3. **极限压缩**: 宽高 50%，品质 20%，输出为 WebP，同目录输出

### 方案操作

- 新建方案 (基于当前设置或从默认模板创建)
- 编辑方案 (修改后保存)
- 删除方案 (至少保留一个)
- 重命名方案
- 导入/导出方案 (JSON 文件)

---

## 6. 文件扫描

### 扫描行为

- 递归扫描指定目录及所有子目录
- 过滤支持的图片格式 (.jpg, .jpeg, .gif, .webp, .png)
- 扫描阶段只读取文件元信息 (路径、大小、扩展名)
- 图片尺寸信息延迟到处理阶段才读取 (避免扫描大目录时的延迟)

### 文件列表展示

- 支持按目录分组 (树形) 或平铺列表视图
- 支持按格式筛选 (全部 / JPG / PNG / WebP / GIF)
- 显示文件总数和总大小

---

## 7. 输出模式

| 模式 | 说明 |
|------|------|
| 覆盖原文件 | 直接替换原始文件 (不可逆) |
| 同目录输出 | 在原文件同目录下生成新文件，带后缀 (如 `_compressed`) |
| 自定义目录 | 输出到用户指定的目录，保持相对目录结构 |

输出目录结构保持与源目录一致 (相对路径映射)。

---

## 8. V1 范围

### 包含

- 目录选择与文件浏览 (递归扫描、格式筛选、文件计数)
- 配置方案管理 (创建/编辑/删除/切换)
- 大小调整 (百分比/像素、4种缩放模式、保持宽高比)
- 品质控制 (品质百分比、目标大小、保持原品质)
- 格式转换 (JPEG/PNG/WebP/GIF)
- 分辨率调整 (DPI 设置)
- 输出模式 (覆盖/同目录/自定义目录)
- 批量处理 (rayon 多核并发、实时进度)
- 处理进度 (进度条、文件明细、大小对比、节省统计)

### V1 不包含 (后续迭代)

- 滤镜效果 (锐化、模糊等)
- 旋转/翻转
- 水印
- 批量重命名模板
- 文件缩略图预览
- PNG 有损压缩 (imagequant 集成)
- 拖拽添加文件

---

## 9. 技术风险与应对

| 风险 | 应对措施 |
|------|---------|
| GIF 动图处理 | `image` crate 对 GIF 编码支持有限，补充 `gif` crate 处理动图帧 |
| PNG 有损压缩 | `imagequant` 是 C 库编译复杂，V1 先只支持 PNG 无损，V2 再引入 |
| 超大图片内存 | 单张超大图可能 OOM，考虑流式处理或限制单张图片最大尺寸 |
| Tauri WebView 兼容性 | Windows 默认使用 WebView2 (Edge)，需确保目标机器已安装 |
