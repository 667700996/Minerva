# Minerva CLI 사용 가이드

## 기본 실행

```
cargo run -p minerva-cli
```

- `configs/dev.toml`을 기본 설정으로 로드합니다.
- 실행 시 터미널 UI가 열리며, `q` 또는 `Esc` 키로 종료할 수 있습니다.
- 기본 설정은 한 번의 턴(`max_retries = 1`)과 기본 진형 `마상상마` (`FormationPreset::MasangSangMa`)를 사용합니다.

## 구성 파일 지정

```
cargo run -p minerva-cli -- <path/to/config.toml>
```

또는 환경 변수 `MINERVA_CONFIG`로 TOML 경로를 지정할 수 있습니다.

## 실행 옵션

```
cargo run -p minerva-cli -- --max-retries 3 --formation MasangMasang
```

- `--max-retries N` : 대국 턴 루프 반복 횟수(기본 1).
- `--formation PRESET` : 시작 진형을 지정합니다.  
  사용 가능한 값은 `MasangMasang`, `SangMasangMa`, `MasangSangMa`, `SangMaMaSang` 입니다(대소문자 무시).
- `--controller MODE` : `adb`(기본) 또는 `mock` 중 선택해 실제 에뮬레이터/ADB 제어 여부를 결정합니다.

## 터미널 UI

실행 중 TUI는 라이프사이클, 엔진 결정, 텔레메트리 이벤트를 실시간으로 표시합니다.  
상단 요약 패널에는 마지막 이벤트 상태가, 하단 리스트에는 최근 로그가 역순으로 나타납니다.
