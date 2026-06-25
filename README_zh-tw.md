# ocrs-cjk

> **Pure Rust CJK OCR 引擎 — 將掃描 PDF 轉換為可搜尋 PDF。PaddleOCR 偵測、ONNX 辨識、WASM 安全、離線優先。**

[ocrs](https://github.com/robertknight/ocrs) 的 Fork，全面支援 CJK（中文、日文、韓文）：完整 PaddleOCR 模型支援、CJK 感知分詞、信心分數、可搜尋 PDF 輸出（ToUnicode CMap）、結構化輸出格式（hOCR、ALTO XML、JSON）。零 C/C++ 依賴，原生 `wasm32-unknown-unknown` 支援。上游（`robertknight/ocrs`）的更新會定期合併進來。

**語言:** [English](README.md) | [日本語](README_ja.md) | [简体中文](README_zh.md) | [繁體中文](README_zh-tw.md) | [한국어](README_kr.md)

[![CI](https://github.com/kent-tokyo/ocrs-cjk/actions/workflows/ci.yml/badge.svg)](https://github.com/kent-tokyo/ocrs-cjk/actions/workflows/ci.yml)

---

**ocrs** 是一個用於從圖像中擷取文字的 Rust 函式庫和 CLI 工具，即光學字元辨識（OCR）。

專案目標是構建一個現代化的 OCR 引擎，具備以下特性：

 - 在掃描文件、含文字的照片、截圖等各類圖像上表現良好，相比 [Tesseract][tesseract] 等傳統引擎所需前處理更少（透過在流水線中更廣泛地使用機器學習實現）
 - 易於在多種平台上編譯和執行，包括 WebAssembly
 - 使用開放且授權寬鬆的資料集進行訓練
 - 程式碼庫易於理解和修改

底層使用在 [PyTorch][pytorch] 中訓練的神經網路模型，匯出為 [ONNX][onnx] 格式後由 [RTen][rten] 引擎執行。詳情請參閱[模型與資料集](#模型與資料集)。

[onnx]: https://onnx.ai
[pytorch]: https://pytorch.org
[rten]: https://github.com/robertknight/rten
[tesseract]: https://github.com/tesseract-ocr/tesseract

## 狀態

ocrs 目前處於早期預覽階段，辨識錯誤率高於商業 OCR 引擎。

## 語言支援

本 Fork 已擴展 CJK（中文、日文、韓文）支援：
- **完整 PaddleOCR 模型支援**：偵測（DB 模型，3 通道 RGB，動態尺寸）與辨識（PP-OCRv5 ONNX）均從模型元資料自動辨識
- **結構化輸出**：`--hocr`（hOCR HTML）、`--alto`（ALTO v4 XML）、`-j`（JSON）— 均含逐詞邊界框與信心分數
- **信心分數**：透過 `TextItem::confidence()` 取得字元級與詞級辨識信心度
- **CJK 感知分詞**：`TextLine::segments()` 無需空格即可在文字邊界（拉丁 ↔ CJK）進行分割
- **字母表輔助函式**：`hiragana()`, `katakana()`, `cjk_unified()`, `hangul()`, `cjk_alphabet()`, `cjk_alphabet_chars()`
- **UTF-8 安全**：所有字串處理均使用字元邊界感知方法（`char_indices`、`chars`），不進行位元組切片
- **WASM 完全相容**：`recognize_text` 的 rayon panic 已修復 — 在 `wasm32-unknown-unknown` 上完整流水線可執行

上游 ocrs 僅支援拉丁字母。原始語言支援路線圖請參閱 [upstream issue](https://github.com/robertknight/ocrs/issues/8)。

## 與其他 OCR 解決方案的比較

| 解決方案 | 執行時 | CJK (JA/ZH/KO) | 原生 WASM | 無 C/C++ | 離線 | hOCR/ALTO | 授權 |
|---|---|---|---|---|---|---|---|
| **ocrs-cjk**（本 Fork） | Pure Rust | Yes / Yes / Yes | Yes | Yes | Yes | Yes | Apache-2.0 / MIT |
| [ocrs](https://github.com/robertknight/ocrs)（上游） | Pure Rust | No（僅拉丁字母） | Yes | Yes | Yes | No | Apache-2.0 / MIT |
| [Tesseract](https://github.com/tesseract-ocr/tesseract) | C++（`tesseract-sys` FFI） | Yes / Yes / Yes | 部分¹ | No | Yes | Yes | Apache-2.0 |
| [PaddleOCR](https://github.com/PaddlePaddle/PaddleOCR) | Python / C++ | Yes / Yes / Yes | 部分² | No | Yes | No | Apache-2.0 |
| [EasyOCR](https://github.com/JaidedAI/EasyOCR) | Python / PyTorch | Yes / Yes / Yes | No | No | Yes | No | Apache-2.0 |
| [RapidOCR](https://github.com/RapidAI/RapidOCR) | Python / ONNX | Yes / Yes / Unknown | No | No | Yes | No | Apache-2.0 |
| [manga-ocr](https://github.com/kha-white/manga-ocr) | Python / PyTorch | 僅日文 | 非官方³ | 可選 | Yes | No | Apache-2.0 |

¹ `tesseract-wasm` 為獨立 JS 專案；CJK tessdata 需單獨載入；非原生 `wasm32-unknown-unknown`。  
² PaddleOCR 有 JS 瀏覽器 SDK，但並非 Rust 原生 WASM。  
³ 社群製作的 Chrome 擴充功能，非生產級別。

**ocrs-cjk 是唯一同時具備 Pure Rust、零 C/C++ 依賴、原生 `wasm32-unknown-unknown`、完全離線、CJK 辨識能力的解決方案。**

### 精度參考（PP-OCRv5，PaddleOCR 內部基準）
- 簡體中文（印刷體）：辨識率約 90%
- 日文：辨識率約 74%
- ocrs-cjk 實測 CER（合成圖像）：平假名／片假名／漢字／簡體中文／混合 CJK+Latin／長文 = 0%；繁體中文罕見字形（`臺灣` 等）約 67%

> **繁體中文注意事項：** PP-OCRv5 對常用繁體字（香港、澳門等）辨識良好，但對罕見字形（如 `臺`）準確率較低。若主要處理繁體中文，建議改用繁體中文專用的 ONNX 辨識模型，替換步驟與下方相同，僅需替換模型檔案與對應字典。

## 使用外部模型進行 CJK OCR

端到端 CJK OCR 需要兩個模型協同工作：

| 階段 | 作用 | 狀態 |
|---|---|---|
| **偵測模型** | 定位圖像中的文字區域 | [!] 可使用 ocrs 內建的拉丁文訓練模型（CJK 偵測精度未經驗證）；PaddleOCR 格式的偵測模型尚不支援 |
| **辨識模型** | 讀取偵測區域中的字元 | Yes 已支援 PaddleOCR ONNX 格式（自動偵測 3 通道輸入與 batch-first 輸出） |

本儲存庫不包含 CJK 訓練模型，需要自行取得。

### 第一步 — 下載辨識模型

[PP-OCRv5](https://github.com/PaddlePaddle/PaddleOCR) 在單一模型中支援簡體中文、繁體中文、日文和英文。Hugging Face 上提供了預轉換的 ONNX 檔案，執行以下 Python 腳本即可下載：

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

### 第二步 — 提取字元字典

辨識模型輸出標籤索引，需要將對應的字元清單作為 `OcrEngineParams::alphabet` 傳入。執行以下腳本從 YAML 設定中提取並產生 `alphabet.txt`：

```python
import yaml

with open("models/PP-OCRv5_server_rec_infer.yml") as f:
    cfg = yaml.safe_load(f)

chars = cfg["PostProcess"]["character_dict"]

# 部分條目（如國旗 emoji）包含兩個 Unicode 碼點。
# ocrs 將一個標籤對應到一個字元，因此將其截斷為第一個碼點。
# 這些條目不會出現在 CJK OCR 輸出中，不影響實際使用。
fixed = [c[0] if len(c) > 1 else c for c in chars]

# PaddleOCR 預設在末尾附加空格標籤（use_space_char=True）。
fixed.append(" ")

with open("models/alphabet.txt", "w", encoding="utf-8") as f:
    f.write("".join(fixed))

print(f"已寫入 {len(fixed)} 個字元 → models/alphabet.txt")
```

> **已驗證：** PP-OCRv5 字典含 18383 個字元 + 空格 = 18384 個字元。
> `18384 + 1 (CTC blank) = 18385`，與模型輸出維度完全符合。

### 第三步 — 透過 CLI 執行

啟用 ONNX 支援進行建置，然後使用 `--alphabet-file` 傳入字典檔案（避免大字元集的 shell 轉義問題）：

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

// 載入 PaddleOCR 辨識模型（通道數和輸出布局從 input_shape 自動偵測）
let rec_model = Model::load_file("models/PP-OCRv5_server_rec_infer.onnx")?;

// 載入第二步產生的字典檔案
let alphabet = std::fs::read_to_string("models/alphabet.txt")?;

let engine = OcrEngine::new(OcrEngineParams {
    recognition_model: Some(rec_model),
    // 省略 detection_model 則使用內建的拉丁文訓練模型。
    alphabet: Some(&alphabet),
    ..Default::default()
})?;
```

### 已知限制

- **偵測模型**：內建偵測模型基於拉丁文訓練。實測與 PP-OCRv5 搭配可完成 CJK OCR，但對複雜版面的精度不保證。PaddleOCR 格式偵測模型的支援已納入計畫。
- **ONNX 功能旗標**：載入 `.onnx` 檔案需要啟用 `--features onnx`（rten 預設格式為 `.rten`）。
- **WASM**：`recognize_text` 在 `wasm32-unknown-unknown` 上會發生執行時 panic（上游 `rayon` 問題）。
- **字典不符**：若 `alphabet` 字串的順序或長度與模型訓練字典不一致，辨識結果將出現亂碼。請務必使用從模型 YAML 設定中提取的字典。
- **繁體中文罕見字形**：PP-OCRv5 對 `臺`、`灣` 等罕見繁體字形的辨識率較低（實測約 67% CER）。建議改用繁體中文專用模型以提升精度。

## CLI 安裝

首先確認已安裝 Rust 和 Cargo，然後執行：

```sh
$ cargo install ocrs-cli --locked
```

若需啟用從系統剪貼簿讀取圖像的功能，新增 `clipboard` feature：

```sh
$ cargo install ocrs-cli --locked --features clipboard
```

## CLI 使用方法

從圖像中擷取文字：

```sh
$ ocrs image.png
```

首次執行時，所需模型將自動下載並儲存至 `~/.cache/ocrs`。

若安裝時啟用了 `clipboard` feature，可從系統剪貼簿中的圖像擷取文字：

```sh
$ ocrs --clipboard
$ ocrs -c  # 簡寫形式
```

### 更多範例

擷取文字並寫入 `content.txt`：

```sh
$ ocrs image.png -o content.txt
```

以 JSON 格式擷取文字和版面資訊：

```sh
$ ocrs image.png --json -o content.json
```

產生標註了偵測到的詞語和行位置的圖像：

```sh
$ ocrs image.png --png -o annotated.png
```

## 作為函式庫使用

作為 Rust 函式庫的使用方法，請參閱 [ocrs crate README](ocrs/)。

## 模型與資料集

ocrs 使用基於 PyTorch 編寫的神經網路模型。關於模型、資料集的詳細資訊以及自訂模型訓練工具，請參閱 [ocrs-models](https://github.com/robertknight/ocrs-models) 儲存庫。模型也以 ONNX 格式提供，可用於其他機器學習執行時。

## 開發

在本地建置和執行函式庫及 CLI 工具需要安裝最新穩定版 Rust：

```sh
git clone https://github.com/kent-tokyo/ocrs-cjk.git
cd ocrs-cjk
cargo run -p ocrs-cli -r -- image.png
```

### 測試

修改程式碼後，執行單元測試和 lint 檢查：

```sh
make check
```

也可以直接執行標準的 `cargo test` 指令。

執行端到端測試：

```sh
make test-e2e
```

有關 ML 模型評估方法的詳細資訊，請參閱 [ocrs-models](https://github.com/robertknight/ocrs-models) 儲存庫。
