# tools/verify —— 书稿 ↔ 代码 一致性检查

从随书校验工具移植而来（原始出处：修订项目 `verify_tools/`，本轮已同步全部脚本）。

## 能查什么

| 脚本 | 作用 |
|---|---|
| `syntax_check.py <md>` | 对书稿每个 ```` ```rust ```` / ```` ```rust,ignore ```` 代码块逐块跑 `rustc --crate-type lib`，分类 PASS（可独立编译）/ CONTEXT（疑似依赖上下文缺符号）/ SUSPECT（结构性问题，疑似真 bug） |
| `classify_suspects.py <md> [json]` | 把 SUSPECT 按块首语法再拆成“教学片段（省略外壳，预期如此）”与“真可疑（REAL-SUSPECT，应人工复核）” |
| `crosscheck.py` | 符号定义/引用交叉核验（配合 `_verify_out` 产物） |
| `run_all.sh [书稿.md]` | 一键跑前两步；默认书稿 `book/scratch/book-draft.md` |

## 用法

```bash
# 推荐入口（CI 与本地同用）
tools/verify/run_all.sh            # 默认书稿

# 手动分步
python3 tools/verify/syntax_check.py  book/scratch/book-draft.md
python3 tools/verify/classify_suspects.py book/scratch/book-draft.md
```

## 输出

书稿同目录下生成 `_verify_out/syntax_check.json`（被 gitignore）。SUSPECT 中
**REAL-SUSPECT** 为高优先级——历史勘误中“书中引用符号但全书无定义/变体缺失”一类，
正是靠这个链路抓出来的。

## 与 CI 的关系

`.github/workflows/ci.yml` 的 `verify-blocks` 作业在每次 PR 中执行 `run_all.sh`，
并把 `syntax_check.json` 上传为 artifact 供人工检查。当前策略为**只报告、不拦截**——
REAL-SUSPECT 集合里既有真问题，也有大量“上下文省略”的教学片段（本书稿已知 36 项，
逐项人工核过，与前序修订一致）。待 `code/` 落地、分类器规则收紧后，再把它改为硬失败，
防止“书稿改了、代码定义没跟上”。
