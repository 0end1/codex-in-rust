## 关联 issue
<!-- Closes #N / Fixes #N / 无 -->

## 改动类型
- [ ] 书稿勘误（book/）
- [ ] 代码修复或补全（code/）
- [ ] 工具/CI（tools/、.github/）
- [ ] 文档/工程化（README 等）

## 改动摘要
<!-- 3~5 句说明改了什么、为什么 -->

## 自查清单
- [ ] 涉及书稿时：本地跑过 `tools/verify/run_all.sh`，REAL-SUSPECT 已说明或清零
- [ ] 涉及代码时：`cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace` 通过
- [ ] 正文与代码改动保持同 PR（本仓库反对“代码改了、书没跟”的脱节 PR）
- [ ] 没有无关文件混入（改一行别带十行）
- [ ] 改动对应的“读者视角影响”已在 PR 描述里写明

## 备注
<!-- 给 reviewer 的话、未尽事项 -->
