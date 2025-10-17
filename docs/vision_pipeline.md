# Vision Pipeline Notes

현재 저장되는 데이터
- 전체 프레임: `captures/` 아래 `frame_*.png`
- 격자 타일: `captures/tiles/` 아래 `f{file}_r{rank}_timestamp.png`

`assets/templates/`에 `blue_soldier.png` 와 같이 `{owner}_{piece}.png` 형식의 템플릿을 배치하면 간단한 평균 차이 기반 매칭으로 기물이 추론됩니다. 점수(0~255)를 255로 나눈 값이 `vision.confidence_threshold` 보다 작아야 기물로 인정됩니다.

TODO
- 타일 디렉터리에서 기물별 템플릿을 구성하고 `assets/templates/`에 저장
- 템플릿 매칭(예: SIFT/NCC) 또는 경량 CNN 등을 활용해 `TemplateMatchingRecognizer`를 실제 인식기로 교체
