# zhconv-typst

[English](README.md) | 简体中文

使用 [zhconv-rs](https://github.com/Gowee/zhconv-rs) 在简体、繁体与地区变体之间转换中文字符串与嵌套内容。

## 已发布包与开发版本

当前 Universe 上的已发布包为 `@preview/zhconv:0.3.1`。本源码树准备的是更新版包，**不是**那个已发布版本。清单版本**跟随 Rust 核心的 workspace 版本**（当前为 `0.5.0`，对应 0.5 开发线）；构建工具强制 `typst.toml` 与核心版本同步，因此 Typst 包版本始终标识所捆绑的核心。核心依赖来自本 checkout（当前为 0.5 开发版），而非 crates.io 的 0.5 发布版。

本地构建并安装此开发包后：

```typst
#import "@local/zhconv:0.5.0": convert, convert-content, convert-wikitext

#convert("汉字转换", "zh-Hant") // 漢字轉換
#convert-content([柳外輕雷池上雨], "zh-Hans")
#convert-wikitext("-{zh-hans:甲;zh-hant:乙;}-", "zh-Hant") // 乙
```

## API 与转换行为

公开 API 围绕语义化命名与显式契约重新设计：

- `convert(text, target)` —— 唯一无歧义的主入口（`str -> str`）。
- `convert-wikitext(text, target)` —— 感知 MediaWiki 变体标记的转换，独立入口，使主签名不携带规则开关。
- `convert-content(doc, target)` —— 对 content 树的 best-effort 遍历（沿用既有的 `children`、`text`、`body` 处理）；其他值原样通过。其 best-effort 性质是契约的一部分（见下）。
- `detect(text)` —— 粗粒度书写检测，返回 `"zh-Hans"` 或 `"zh-Hant"`。
- `variants` —— 支持的目标标签列表。

旧名称映射如下：`zhconv-str(text, target, wikitext: false)` → `convert` / `convert-wikitext`；`zhconv(document, target, wikitext: false)` → `convert-content`（`wikitext` 参数移除；字符串场景请用 `convert-wikitext`）；`is-hans-str(text)` → `detect(text) == "zh-Hans"`。

目标标签大小写与下划线不敏感（`ZH-HANT`、`zh_hant` 均可）：`zh-Hans`、`zh-Hant`、`zh-TW`、`zh-HK`、`zh-MO`、`zh-CN`、`zh-SG`、`zh-MY`。上游的 `zh` 目标也被接受（纯文本模式下不做书写转换）。未知目标与字节级插件边界上的非法 UTF-8 会报错。wikitext 的传输标志必须恰为一个字节（0 或 1）。

标准构建显式启用 **OpenCC、压缩与 CJK 兼容字归一化**，核心默认特性关闭，不内嵌 MediaWiki 转换表。这与旧的已发布包不同：部分输出会变化。

- Hans/Hant 选择书写转换规则；TW/HK 额外选择各自的地区字符规则。MO 与 HK 共用规则；SG/MY 与 CN 共用规则。
- OpenCC 地区词组词典（`opencc-twp` / `opencc-hkp`，均为 Apache-2.0 的 OpenCC 数据）**已启用**：`zh-TW` 执行完整的台湾本地化（`阿拉伯联合酋长国` → `阿拉伯聯合大公國`、`软件在服务器上运行` → `軟體在伺服器上執行`），`zh-HK` 应用香港用词（`滑鼠`）。仅需书写转换、不做地区替换时使用 `zh-Hant`。上游目前不支持运行时选择地区词组子集（词典为编译期开关）；该后续选项已在 #15 跟踪。
- 这些是 zhconv 扁平化的 OpenCC 自动机，不是 OpenCC 原始的多阶段流水线，个别转换可能不同。转换不保证普遍可逆，也不保证在所有语境下语言学正确。
- 不内嵌 MediaWiki 数据集的情况下仍支持 wikitext 语法，经独立的 `convert-wikitext` 入口。
- `convert-content` 是便捷功能，不是对任意 Typst content 的无损改写。特别地，显式文本样式可能丢失，跨 content 节点的词组会被分别转换。在意词组上下文或样式时，请先转换完整字符串再施加格式。
- 本 API 不解析占位符、URL、源代码或标识符；其中的中文文本会被转换。调用方包需自行保护此类语法。

## 迁移示例与已知限制

以下差异在 Typst 0.15.1 下对照已发布 0.3.1 的 WASM 观察到，同时反映词典选择与核心变化，并非 API 变化。12 条语料 × 8 个目标的冒烟集不构成质量评分。

| 输入 / 目标 | 已发布 0.3.1 | 本构建 |
| --- | --- | --- |
| `软件在服务器上运行，鼠标和内存。` / TW | 軟體在伺服器上運行，滑鼠和內存。 | 軟體在伺服器上執行，滑鼠和記憶體。 |
| `软件在服务器上运行，鼠标和内存。` / HK | 軟件在伺服器上運行，鼠標和內存。 | 軟件在伺服器上運行，滑鼠和內存。 |
| `阿拉伯联合酋长国` / TW | 阿拉伯聯合大公國 | 阿拉伯聯合大公國 |
| `神 神 﨑 崎` / Hans | 神 神 﨑 崎 | 神 神 﨑 崎 |

启用地区词组词典后，TW/HK 技术词汇得到恢复，部分比 0.3.1 更进一步（TW `運行` → `執行`、`內存` → `記憶體`；0.3.1 的 HK 输出完全没有 `滑鼠`）。其余观察到的差异来自 CJK 兼容字归一化（新启用的核心特性）与 0.5 开发线上的词典重构。消息模板用户注意：非 ASCII 占位符名会像普通中文一样被转换（如 TW 下 `{用戶名稱}` → `{使用者名稱}`）；`{field}` 等 ASCII 占位符不受影响。

当前核心还有这些测试暴露的已知转换限制：

- TW 下，`簡繁混排：后台與後臺，里面和裡面。` 转为 `簡繁混排：後臺與後臺，裡面和裡麵。`，最后一个 `面` 被错误改动。
- HK 下，`理发以后发现头发干了，干活的人在后台。` 转为 `理髮以後發現頭髮幹了，幹活的人在後枱。`。`干了` 与 `后台` 的语境读音未被可靠消歧。

这些场景的原生/WASM 一致只是集成检查，不代表输出正确。核心变化时应复查这些示例，发布前解决或明确接受限制。本绑定不施加临时替换来掩盖核心/词典行为。

## 构建与测试

要求：本仓库的 Git checkout、rustup、Python 3.11+、Typst 0.14.0+。`rust-toolchain.toml` 固定 Rust 工具链与 Wasm 目标；`Cargo.lock` 固定依赖解析。不需要 WASI stubber、wasm-bindgen CLI、Node.js 或 Python 依赖。在 `typst/` 下运行：

```sh
python3 tools/build.py
python3 tests/run.py
# 用另一个编译器可执行文件测试：
python3 tests/run.py --typst /path/to/typst
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings
```

构建器检查解析后的依赖特性是否含 MediaWiki、验证 Wasm 仅导入两个 Typst 协议函数，并按明确清单组装 `dist/package/`；同时复制依赖许可文本并写入含溯源与哈希的 `build-info.json`。测试运行器检查非法插件输入、人工审阅的语义断言，以及与同配置原生 Rust 实现的 96 组对照。它只把打包产物安装到临时 package path，并在源码 checkout 之外编译，包括 content 示例与预期错误用例。原生一致验证的是绑定，不是语言学正确性；小规模语义语料是分开的。

手动安装：把 `dist/package/` 复制到 `<Typst 数据目录>/typst/packages/local/zhconv/0.5.0/`（或 `typst info` 报告的 package path），然后使用上面的 `@local` 导入。测试运行器不要求持久安装。源码树开发时，把构建出的 WASM 复制到 `zhconv.typ` 旁即可编译 `example.typ`。

## 发布检查单

1. 复查核心/数据修订与语义断言的变化。对变化的输出要调查原因，不要盲目重新生成期望值。在 CI 中同时检查最低与当前 Typst 版本。
2. 复查依赖许可与生成的溯源信息。保持清单、LICENSE 文件与词典配置一致。源码级特性变化会同时影响输出与二进制许可。
3. 与维护者商定包版本，同步更新清单与本地导入/测试，并从干净的已提交 checkout 重新构建。再用干净的 Cargo target 目录在固定工具链下重复构建并比较 WASM 哈希。哈希不承诺跨工具链版本稳定。
4. 只提交 `dist/package/` 的内容到 `typst/packages` 对应版本目录，包括 WASM、声明与许可文本。绝不复制早前构建的陈旧 WASM。已发布版本不受影响。

脚本不发布包。发布由维护者单独审查批准。代码/数据许可划分见 [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md)（英文），维护讨论见 [issue #15](https://github.com/Gowee/zhconv-rs/issues/15)。
