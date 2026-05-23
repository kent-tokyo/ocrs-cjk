# ocrs-cjk

> **本项目是 [ocrs](https://github.com/robertknight/ocrs) 的 Fork，专注于 CJK（中文、日文、韩文）文字识别。**
> 目标是在 ocrs 基础上扩展 CJK 字符集、CJK 感知文本分词，并实现完全离线 / WebAssembly 兼容——不依赖任何 C/C++ 库（无 Tesseract，无 OpenCV）。
> 上游（`robertknight/ocrs`）的更新会定期合并进来。

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
- 通过 `TextLine::segments()` 实现 CJK 感知文本分词
- 字母表辅助函数：`hiragana()`, `katakana()`, `cjk_unified()`, `hangul()`, `cjk_alphabet()`, `cjk_alphabet_chars()`
- `cjk_text` 模块中的 UTF-8 安全字节边界工具

上游 ocrs 仅支持拉丁字母。原始语言支持路线图请参阅 [upstream issue](https://github.com/robertknight/ocrs/issues/8)。

> **WASM 限制：** `OcrEngine::recognize_text` 使用 `rayon` 进行并行处理，在 `wasm32-unknown-unknown` 目标上会发生运行时 panic。这是从上游继承的已知问题。其余 API（`detect_words`、`find_text_lines`、`cjk_text` 工具）均兼容 WASM。

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
