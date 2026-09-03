#!/usr/bin/env bash
# 打一个 Part checkpoint tag（把“这一 Part 结束的可编译状态”固定下来）。
# 前言承诺的 tag 链：part0..part5，与书的章节一一对应。
#
# 用法:
#   scripts/checkpoint-tag.sh part0 "第 1-2 章结束"
#   scripts/checkpoint-tag.sh part3 "第 10-12 章结束（安全篇完成）"
#
# 注意：tag 具有不可变性——打错不能改只能删；每次 Part 验收全绿后再执行。
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

TAG="${1:?usage: checkpoint-tag.sh <part0|..|part5> <message>}"
shift
MSG="$*"
[[ "$TAG" =~ ^part[0-5]$ ]] || { echo "tag 必须形如 part0..part5，收到: $TAG" >&2; exit 1; }
[[ -n "$MSG" ]] || MSG="checkpoint $TAG"
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "tag $TAG 已存在，拒绝覆盖: $(git rev-parse --short "$TAG")" >&2; exit 1
fi

# 骨架阶段守卫：code/ 尚无 crate 时不打“可编译”承诺的 tag
if [[ ! -d code/crates ]]; then
  echo "code/crates 还不存在——先按 code/README.md 把本 Part 代码填实，再来打 tag。" >&2
  exit 1
fi

echo "== 提交前检查 =="
(cd code && cargo fmt --check && cargo clippy -D warnings && cargo test --workspace)

echo "== 提交并打 tag: $TAG =="
git add -A
git commit -m "chore(part): checkpoint ${TAG} — ${MSG}"
git tag -a "$TAG" -m "${MSG}"
echo "完成: $(git rev-parse --short "$TAG")"
