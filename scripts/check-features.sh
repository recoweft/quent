#!/usr/bin/env bash
set -euo pipefail
features=(postgres mysql runtime-tokio tls-rustls)
for ((mask=0; mask<16; mask++)); do
  selected=()
  for ((bit=0; bit<4; bit++)); do
    if (( mask & (1 << bit) )); then selected+=("${features[bit]}"); fi
  done
  joined=$(IFS=,; echo "${selected[*]}")
  cargo check -p quent --no-default-features --features "$joined" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}manifest" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}performance-plan" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}operation-spec" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}manifest,operation-spec" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}codegen" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}review" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}source-review" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}source-review,review" --locked "$@"
  cargo check -p quent-cli --no-default-features --features "${joined:+$joined,}source-review" --locked "$@"
  cargo check -p quent-cli --no-default-features --features "${joined:+$joined,}source-review,review" --locked "$@"
  cargo check -p quent-cli --no-default-features --features "${joined:+$joined,}review" --locked "$@"
  cargo check -p quent-cli --no-default-features --features "${joined:+$joined,}manifest" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}review-ci" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}review-ci,source-review" --locked "$@"
  cargo check -p quent-cli --no-default-features --features "${joined:+$joined,}review-ci" --locked "$@"
  cargo check -p quent-cli --no-default-features --features "${joined:+$joined,}review-ci,source-review" --locked "$@"
  cargo check -p quent --no-default-features --features "${joined:+$joined,}mcp" --locked "$@"
  cargo check -p quent-cli --no-default-features --features "${joined:+$joined,}mcp" --locked "$@"
done

cargo test -p quent-cli --no-default-features --features manifest --locked "$@"

cargo test -p quent-cli --no-default-features --features review --locked "$@"
cargo check -p quent --no-default-features --features review,codegen --locked "$@"
cargo check -p quent-cli --no-default-features --features review,manifest --locked "$@"

cargo check -p quent --no-default-features --features source-review,codegen --locked "$@"
cargo check -p quent-cli --no-default-features --features source-review,manifest --locked "$@"

cargo check -p quent --no-default-features --features review-ci,codegen --locked "$@"
cargo check -p quent-cli --no-default-features --features review-ci,manifest --locked "$@"
cargo test -p quent-cli --no-default-features --features review-ci --locked "$@"

cargo check -p quent --no-default-features --features mcp,codegen --locked "$@"
cargo check -p quent-cli --no-default-features --features mcp,manifest --locked "$@"
