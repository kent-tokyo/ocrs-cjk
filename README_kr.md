# ocrs-cjk

> **이 저장소는 CJK(중국어·일본어·한국어) 문자 인식에 특화된 [ocrs](https://github.com/robertknight/ocrs)의 포크입니다.**
> CJK 알파벳 확장, CJK 인식 텍스트 분절, 그리고 완전한 오프라인 / WebAssembly 호환을 목표로 하며, C/C++ 의존성(Tesseract, OpenCV 등)을 일절 사용하지 않습니다.
> 업스트림(`robertknight/ocrs`)의 변경 사항은 주기적으로 병합됩니다.

---

**ocrs**는 이미지에서 텍스트를 추출하는 Rust 라이브러리 및 CLI 도구입니다(광학 문자 인식, OCR).

다음과 같은 특성을 갖춘 현대적인 OCR 엔진을 목표로 합니다:

 - 스캔 문서, 텍스트가 포함된 사진, 스크린샷 등 다양한 이미지에서 [Tesseract][tesseract] 등 기존 엔진보다 전처리 없이 높은 정확도로 동작(머신러닝을 파이프라인 전반에 적극 활용)
 - WebAssembly를 포함한 다양한 플랫폼에서 손쉽게 컴파일 및 실행 가능
 - 공개적이고 자유로운 라이선스의 데이터셋으로 학습
 - 이해하고 수정하기 쉬운 코드베이스

내부적으로는 [PyTorch][pytorch]로 학습한 신경망 모델을 [ONNX][onnx] 형식으로 내보내 [RTen][rten] 엔진으로 실행합니다. 자세한 내용은 [모델과 데이터셋](#모델과-데이터셋)을 참고하세요.

[onnx]: https://onnx.ai
[pytorch]: https://pytorch.org
[rten]: https://github.com/robertknight/rten
[tesseract]: https://github.com/tesseract-ocr/tesseract

## 상태

ocrs는 현재 얼리 프리뷰 단계입니다. 상용 OCR 엔진보다 오류가 많을 수 있습니다.

## 언어 지원

이 포크는 CJK(중국어·일본어·한국어) 지원을 추가합니다:
- `TextLine::segments()`를 통한 CJK 인식 텍스트 분절
- 알파벳 헬퍼: `hiragana()`, `katakana()`, `cjk_unified()`, `hangul()`, `cjk_alphabet()`, `cjk_alphabet_chars()`
- `cjk_text` 모듈의 UTF-8 안전 바이트 경계 유틸리티

업스트림 ocrs는 라틴 알파벳만 지원합니다. 원래 언어 지원 로드맵은 [upstream issue](https://github.com/robertknight/ocrs/issues/8)를 참고하세요.

> **WASM 제한:** `OcrEngine::recognize_text`는 병렬 처리에 `rayon`을 사용하므로 `wasm32-unknown-unknown`에서 런타임 패닉이 발생합니다. 이는 업스트림에서 상속된 기존 문제입니다. 나머지 API(`detect_words`, `find_text_lines`, `cjk_text` 유틸리티)는 WASM 호환입니다.

## CLI 설치

Rust와 Cargo가 설치되어 있는지 확인한 후 다음을 실행하세요:

```sh
$ cargo install ocrs-cli --locked
```

시스템 클립보드에서 이미지를 읽는 기능을 활성화하려면 `clipboard` feature를 추가합니다:

```sh
$ cargo install ocrs-cli --locked --features clipboard
```

## CLI 사용법

이미지에서 텍스트를 추출하려면:

```sh
$ ocrs image.png
```

처음 실행 시 필요한 모델이 자동으로 다운로드되어 `~/.cache/ocrs`에 저장됩니다.

`clipboard` feature로 설치한 경우, 시스템 클립보드의 이미지에서 텍스트를 추출할 수 있습니다:

```sh
$ ocrs --clipboard
$ ocrs -c  # 단축형
```

### 추가 예시

텍스트를 `content.txt`에 저장:

```sh
$ ocrs image.png -o content.txt
```

텍스트와 레이아웃 정보를 JSON 형식으로 추출:

```sh
$ ocrs image.png --json -o content.json
```

감지된 단어와 줄의 위치를 주석으로 표시한 이미지 생성:

```sh
$ ocrs image.png --png -o annotated.png
```

## 라이브러리 사용법

Rust 라이브러리로 사용하는 방법은 [ocrs crate README](ocrs/)를 참고하세요.

## 모델과 데이터셋

ocrs는 PyTorch로 작성된 신경망 모델을 사용합니다. 모델과 데이터셋의 세부 정보 및 커스텀 모델 학습 도구는 [ocrs-models](https://github.com/robertknight/ocrs-models) 저장소를 참고하세요. 모델은 다른 머신러닝 런타임에서 사용할 수 있도록 ONNX 형식으로도 제공됩니다.

## 개발

라이브러리와 CLI를 로컬에서 빌드하고 실행하려면 최신 안정 버전의 Rust가 필요합니다:

```sh
git clone https://github.com/kent-tokyo/ocrs-cjk.git
cd ocrs-cjk
cargo run -p ocrs-cli -r -- image.png
```

### 테스트

코드 변경 후 단위 테스트와 lint 검사를 실행하려면:

```sh
make check
```

표준 `cargo test` 명령도 직접 사용할 수 있습니다.

E2E 테스트를 실행하려면:

```sh
make test-e2e
```

ML 모델 평가 방법에 대한 자세한 내용은 [ocrs-models](https://github.com/robertknight/ocrs-models) 저장소를 참고하세요.
