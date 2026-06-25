# ocrs-cjk

> **Pure Rust CJK OCR 引擎 — 将扫描 PDF 转换为可搜索 PDF。PaddleOCR 检测、ONNX 识别、WASM 兼容、离线优先。**

[ocrs](https://github.com/robertknight/ocrs) 的 Fork，全面支持 CJK（中文、日文、韩文）：完整 PaddleOCR 模型支持、CJK 感知分词、置信度分数、可搜索 PDF 输出（ToUnicode CMap）、结构化输出格式（hOCR、ALTO XML、JSON）。零 C/C++ 依赖，原生 `wasm32-unknown-unknown` 支持。上游（`robertknight/ocrs`）的更新会定期合并进来。

**语言:** [English](README.md) | [日本語](README_ja.md) | [简体中文](README_zh.md) | [繁體中文](README_zh-tw.md) | [한국어](README_kr.md)

[![CI](https://github.com/kent-tokyo/ocrs-cjk/actions/workflows/ci.yml/badge.svg)](https://github.com/kent-tokyo/ocrs-cjk/actions/workflows/ci.yml)

---

**ocrs** 是一个用于从图像中提取文字的 Rust 库和 CLI 工具，即光学字符识别（OCR）。

项目目标是构建一个现代化的 OCR 引擎，具备以下特性：

 - 在扫描文档、含文字的照片、截图等各类图像上表现良好，相比 [Tesseract][tesseract] 等传统引擎所需预处理更少（通过在流水线中更广泛地使用机器学习实现）
 - 易于在多种平台上编译和运行，包括 WebAssembly
 - 使用开放且许可宽松的数据集进行训练
 - 代码库易于理解和修改

底层使用在 [PyTorch][pytorch] 中训练的神经网络模型，导出为 [ONNX][onnx] 格式后由 [RTen][rten] 引擎执行。详情请参阅[模型与数据集](#模型与数据集)。

[onnx]: https://onnx.ai
[pytorch]: https://pytorch.org
[rten]: https://github.com/robertknight/rten
[tesseract]: https://github.com/tesseract-ocr/tesseract

## 状态

ocrs 目前处于早期预览阶段，识别错误率高于商业 OCR 引擎。

## 语言支持

本 Fork 已扩展 CJK（中文、日文、韩文）支持：
- **完整 PaddleOCR 模型支持**：检测（DB 模型，3 通道 RGB，动态尺寸）与识别（PP-OCRv5 ONNX）均从模型元数据自动识别
- **结构化输出**：`--hocr`（hOCR HTML）、`--alto`（ALTO v4 XML）、`-j`（JSON）— 均含逐词边界框与置信度
- **置信度分数**：通过 `TextItem::confidence()` 获取字符级与词级识别置信度
- **CJK 感知分词**：`TextLine::segments()` 无需空格即可在脚本边界（拉丁 ↔ CJK）进行分割
- **字母表辅助函数**：`hiragana()`, `katakana()`, `cjk_unified()`, `hangul()`, `cjk_alphabet()`, `cjk_alphabet_chars()`
- **UTF-8 安全**：所有字符串操作均使用字符边界感知方法（`char_indices`、`chars`），不进行字节切片
- **WASM 完全兼容**：`recognize_text` 的 rayon panic 已修复 — 在 `wasm32-unknown-unknown` 上完整流水线可运行

上游 ocrs 仅支持拉丁字母。原始语言支持路线图请参阅 [upstream issue](https://github.com/robertknight/ocrs/issues/8)。

## 与其他 OCR 解决方案的对比

| 解决方案 | 运行时 | CJK (JA/ZH/KO) | 原生 WASM | 无 C/C++ | 离线 | hOCR/ALTO | 许可证 |
|---|---|---|---|---|---|---|---|
| **ocrs-cjk**（本 Fork） | Pure Rust | Yes / Yes / Yes | Yes | Yes | Yes | Yes | Apache-2.0 / MIT |
| [ocrs](https://github.com/robertknight/ocrs)（上游） | Pure Rust | No（仅拉丁字母） | Yes | Yes | Yes | No | Apache-2.0 / MIT |
| [Tesseract](https://github.com/tesseract-ocr/tesseract) | C++（`tesseract-sys` FFI） | Yes / Yes / Yes | 部分¹ | No | Yes | Yes | Apache-2.0 |
| [PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR) | Python / C++ | Yes / Yes / Yes | 部分² | No | Yes | No | Apache-2.0 |
| [EasyOCR](https://github.com/JaidedAI/EasyOCR) | Python / PyTorch | Yes / Yes / Yes | No | No | Yes | No | Apache-2.0 |
| [RapidOCR](https://github.com/RapidAI/RapidOCR) | Python / ONNX | Yes / Yes / Unknown | No | No | Yes | No | Apache-2.0 |
| [manga-ocr](https://github.com/kha-white/manga-ocr) | Python / PyTorch | 仅日语 | 非官方³ | 可选 | Yes | No | Apache-2.0 |

¹ `tesseract-wasm` 为独立 JS 项目；CJK tessdata 需单独加载；非原生 `wasm32-unknown-unknown`。  
² PaddleOCR 有 JS 浏览器 SDK，但并非 Rust 原生 WASM。  
³ 社区制作的 Chrome 扩展，非生产级别。

**ocrs-cjk 是唯一同时具备 Pure Rust、零 C/C++ 依赖、原生 `wasm32-unknown-unknown`、完全离线、CJK 识别能力的解决方案。**

### 精度参考（PP-OCRv5，PaddleOCR 内部基准）
- 简体中文（印刷体）：识别率约 90%
- 日语：识别率约 74%
- ocrs-cjk 实测 CER（合成图像）：平假名/片假名/汉字/简体中文/混合 CJK+Latin/长文 = 0%；繁体中文罕见字形（`臺灣` 等）≈ 67%

## 使用外部模型进行 CJK OCR

端到端 CJK OCR 需要两个模型协同工作：

| 阶段 | 作用 | 状态 |
|---|---|---|
| **检测模型** | 定位图像中的文本区域 | Yes 已支持 PaddleOCR DB 检测模型（3 通道 RGB、动态尺寸、ImageNet 归一化 — 从模型元数据自动检测）。内置拉丁文训练模型可作为后备使用 |
| **识别模型** | 读取检测区域中的字符 | Yes 已支持 PaddleOCR ONNX 格式（自动检测 3 通道输入与 batch-first 输出） |

本仓库不包含 CJK 训练模型，需要自行获取。

### 第一步 — 下载识别模型

[PP-OCRv5](https://github.com/PaddlePaddle/PaddleOCR) 在单个模型中支持简体中文、繁体中文、日语和英语。Hugging Face 上提供了预转换的 ONNX 文件，运行以下 Python 脚本即可下载：

```sh
pip install huggingface-hub pyyaml
```

```python
from huggingface_hub import hf_hub_download

hf_hub_download(
    repo_id="marsena/paddleocr-onnx-models",
    filename="PP-OCRv5_server_rec_infer.onnx",
    local_dir="./models",
)
hf_hub_download(
    repo_id="marsena/paddleocr-onnx-models",
    filename="PP-OCRv5_server_rec_infer.yml",
    local_dir="./models",
)
```

### 第二步 — 提取字符字典

识别模型输出标签索引，需要将对应的字符列表作为 `OcrEngineParams::alphabet` 传入。运行以下脚本从 YAML 配置中提取并生成 `alphabet.txt`：

```python
import yaml

with open("models/PP-OCRv5_server_rec_infer.yml") as f:
    cfg = yaml.safe_load(f)

chars = cfg["PostProcess"]["character_dict"]

# 部分条目（如国旗 emoji ）包含两个 Unicode 码点。
# ocrs 将一个标签映射到一个字符，因此将其截断为第一个码点。
# 这些条目不会出现在 CJK OCR 输出中，不影响实际使用。
fixed = [c[0] if len(c) > 1 else c for c in chars]

# PaddleOCR 默认在末尾追加空格标签（use_space_char=True）。
fixed.append(" ")

with open("models/alphabet.txt", "w", encoding="utf-8") as f:
    f.write("".join(fixed))

print(f"已写入 {len(fixed)} 个字符 → models/alphabet.txt")
```

> **已验证：** PP-OCRv5 字典含 18383 个字符 + 空格 = 18384 个字符。
> `18384 + 1 (CTC blank) = 18385`，与模型输出维度完全匹配。

### 第三步 — 通过 CLI 运行

启用 ONNX 支持进行构建，然后使用 `--alphabet-file` 传入字典文件（避免大字符集的 shell 转义问题）：

```sh
cargo build -p ocrs-cli --release --features onnx

./target/release/ocrs \
  --rec-model  models/PP-OCRv5_server_rec_infer.onnx \
  --alphabet-file models/alphabet.txt \
  image.png
```

### 第四步 — 在 Rust 中使用

```rust
use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;

// 加载 PaddleOCR 识别模型（通道数和输出布局从 input_shape 自动检测）
let rec_model = Model::load_file("models/PP-OCRv5_server_rec_infer.onnx")?;

// 加载第二步生成的字典文件
let alphabet = std::fs::read_to_string("models/alphabet.txt")?;

let engine = OcrEngine::new(OcrEngineParams {
    recognition_model: Some(rec_model),
    // 省略 detection_model 则使用内置的拉丁文训练模型。
    alphabet: Some(&alphabet),
    ..Default::default()
})?;
```

### 已知限制

- **检测模型**：内置检测模型基于拉丁文训练。实测与 PP-OCRv5 配合可完成 CJK OCR，但对复杂版面的精度不保证。PaddleOCR 格式检测模型的支持已纳入计划。
- **ONNX 功能标志**：加载 `.onnx` 文件需要启用 `--features onnx`（rten 默认格式为 `.rten`）。
- **WASM**：`recognize_text` 在 `wasm32-unknown-unknown` 上会发生运行时 panic（上游 `rayon` 问题）。
- **字典不匹配**：若 `alphabet` 字符串的顺序或长度与模型训练字典不一致，识别结果将出现乱码。请务必使用从模型 YAML 配置中提取的字典。

## CLI 安装

首先确保已安装 Rust 和 Cargo，然后执行：

```sh
$ cargo install ocrs-cli --locked
```

若需启用从系统剪贴板读取图像的功能，添加 `clipboard` feature：

```sh
$ cargo install ocrs-cli --locked --features clipboard
```

## CLI 使用方法

从图像中提取文字：

```sh
$ ocrs image.png
```

首次运行时，所需模型将自动下载并保存至 `~/.cache/ocrs`。

若安装时启用了 `clipboard` feature，可从系统剪贴板中的图像提取文字：

```sh
$ ocrs --clipboard
$ ocrs -c  # 简写形式
```

### 更多示例

提取文字并写入 `content.txt`：

```sh
$ ocrs image.png -o content.txt
```

以 JSON 格式提取文字和版面信息：

```sh
$ ocrs image.png --json -o content.json
```

生成标注了检测到的词语和行位置的图像：

```sh
$ ocrs image.png --png -o annotated.png
```

## 作为库使用

作为 Rust 库的使用方法，请参阅 [ocrs crate README](ocrs/)。

## 模型与数据集

ocrs 使用基于 PyTorch 编写的神经网络模型。关于模型、数据集的详细信息以及自定义模型训练工具，请参阅 [ocrs-models](https://github.com/robertknight/ocrs-models) 仓库。模型也以 ONNX 格式提供，可用于其他机器学习运行时。

## 开发

在本地构建和运行库及 CLI 工具需要安装最新稳定版 Rust：

```sh
git clone https://github.com/kent-tokyo/ocrs-cjk.git
cd ocrs-cjk
cargo run -p ocrs-cli -r -- image.png
```

### 测试

修改代码后，运行单元测试和 lint 检查：

```sh
make check
```

也可以直接运行标准的 `cargo test` 命令。

运行端到端测试：

```sh
make test-e2e
```

有关 ML 模型评估方法的详细信息，请参阅 [ocrs-models](https://github.com/robertknight/ocrs-models) 仓库。
