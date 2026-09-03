#!/usr/bin/env bash
# 书稿 → 代码块一致性检查入口（语法层）：
#   1) 对书稿全部 ```rust/```rust,ignore 代码块做逐块 rustc 语法解析并分类
#   2) 对 SUSPECT 再分“教学片段 / 真可疑”，真可疑即疑似书稿 bug
# 用法: tools/verify/run_all.sh [书稿.md]
#   （默认指向 book/scratch/book-draft.md；产物 *_verify_out/syntax_check.json 在书稿同目录下）
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MD_DEFAULT="$HERE/../../book/scratch/book-draft.md"
MD="${1:-$MD_DEFAULT}"
MD="$(cd "$(dirname "$MD")" && pwd)/$(basename "$MD")"
[[ -f "$MD" ]] || { echo "找不到书稿: $MD" >&2; exit 1; }

echo "[1/2] 逐块 rustc 语法检查: $MD"
python3 "$HERE/syntax_check.py" "$MD"
echo
echo "[2/2] SUSPECT 二次分类（教学片段 vs 真可疑）"
python3 "$HERE/classify_suspects.py" "$MD"
echo
echo "产物: $(dirname "$MD")/_verify_out/syntax_check.json"
