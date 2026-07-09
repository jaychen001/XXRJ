# 项目交接说明

更新时间：2026-07-09
仓库路径：`D:\codex\非标设计选型软件`  
代码目录：`selector-desktop/`  
当前分支：`main`

## 1. 产品目标

这是一个 Windows 离线桌面应用，用于非标机械设计日常选型计算。用户测试后已明确：第一版主界面只保留“选型计算”和计算结果页内“导出报告”，不再暴露 PDF 覆盖矩阵、QA 覆盖检查、知识检索、内部参数库等开发期入口。

核心要求：
- 计算输出必须包含公式、代入值、中间值、结论、风险和可导出报告。
- 用户可见界面不展示 PDF 页码、资料名或“按某 PDF”的痕迹。
- 安全系数必须由用户手动输入或确认，系统不能自动代入。
- 厂家样本和型号推荐底层能力可保留，但第一版不做独立主入口。
- 报告导出放在计算结果页，支持 PDF 和 Excel。

## 2. 规划文档状态

已完成并推送：
- `Product-Spec.md`
- `Design-Brief.md`
- `DEV-PLAN.md`

这些文档是当前单一事实源。继续开发前先读：
1. `AGENTS.md`
2. `.agents/skills/dev-builder/SKILL.md`
3. `Product-Spec.md`
4. `Design-Brief.md`
5. `DEV-PLAN.md`

## 3. Git 状态

已推送到 GitHub：
- `155a16f docs: initialize selection tool planning`
- `468d265 feat: add phase 1 desktop skeleton`
- `5614101 feat: add root pdf ingest and knowledge search`

当前 `main` 本地领先 `origin/main`。最新本地提交：
- `52255f2 fix: simplify selector UI around calculation`

该提交已完成用户测试反馈中的主界面简化和报告入口调整，但 GitHub 推送多次失败，错误表现为连接重置或无法连接到 `github.com:443`。下一位接手者应先尝试重新推送。

当前工作区在 `52255f2` 之后继续开发公式输入重构，尚未提交。

未提交改动要点：
- 伺服/步进：增加外部阻力、垂直负载系数。
- 滚珠丝杠：增加支撑跨距、底径、支撑系数、动载荷、目标寿命，并输出临界转速和寿命风险。
- 气缸：增加外部阻力、垂直负载系数、有效面积系数。
- 真空吸附：增加姿态修正系数。
- 直线导轨：增加目标行走寿命和寿命估算。
- `Product-Spec.md`、`DEV-PLAN.md`、`FORMULA-RESEARCH.md` 已同步本轮状态。

## 4. 已完成阶段

### Phase 1：桌面应用骨架

状态：完成、已提交、已推送。

主要内容：
- Tauri 2 + React + Vite + TypeScript + SQLite 桌面应用骨架。
- 23 章 PDF 覆盖矩阵页面。
- 左侧导航、顶部搜索、右侧追溯面板。
- SQLite 初始化和基础表结构。
- Windows 构建脚本：`scripts/tauri-build.cmd`、`scripts/tauri-dev.cmd`。
- 前端测试、Rust migration 测试、Tauri 打包均通过。

### Phase 2：根目录 PDF 解析与覆盖矩阵

状态：完成、已提交、已推送。

主要内容：
- 使用 `pdf-extract = "0.12.0"` 读取根目录 PDF。
- 生成 23 章覆盖矩阵种子：`selector-desktop/src-tauri/resources/pdf_coverage_matrix.json`。
- 抽取 PDF 页文本、目录项、章节匹配页、知识条目。
- 知识检索支持验收词：`惯量比`、`摩擦系数`、`负载率`。
- 内部参数候选抽取支持待确认、确认入库、忽略。
- 安全系数只作为候选提示，不自动代入计算。
- 新增迁移：`0002_pdf_catalog.sql`。
- 覆盖矩阵页可直接“导入/刷新 PDF 索引”。

Phase 2 复审通过，未遗留 HIGH/MEDIUM。

## 5. 当前未提交进度：Phase 3

Phase 3 目标：计算/规则引擎与案例闭环。

当前已实现但未提交：

后端：
- `selector-desktop/src-tauri/src/engine/`
  - `models.rs`：模块、字段、计算请求、结果、步骤、风险模型。
  - `units.rs`：单位换算，覆盖 mm/m、kg/N、s/min、rpm/rps、W/kW 等。
  - `safety_factor.rs`：安全系数必须手动输入或确认。
  - `modules.rs`：模块定义，含可运行演示模块 `timing-belt-basic`。
  - `formula.rs`：同步带基础演示计算，输出摩擦力、加速度、加速力、等效推力、输出扭矩、需求转速。
  - `rules.rs`：规则选型提示模型占位。
  - `mod.rs`：Tauri 命令 `list_calculation_modules`、`run_calculation`。
- `selector-desktop/src-tauri/src/cases/`
  - `models.rs`：保存案例、案例记录、案例运行记录模型。
  - `repository.rs`：保存、列表、复制、重算、删除。
  - `mod.rs`：Tauri 命令 `save_calculation_case`、`list_calculation_cases`、`duplicate_calculation_case`、`rerun_calculation_case`、`delete_calculation_case`。
- `selector-desktop/src-tauri/src/main.rs`
  - 已注册 Phase 3 新命令。

前端：
- `selector-desktop/src/domain/calculation.ts`
- `selector-desktop/src/shared/api/calculation.ts`
- `selector-desktop/src/features/calculation/`
  - 模块列表。
  - 动态计算表单。
  - 计算结果与过程面板。
- `selector-desktop/src/features/cases/`
  - 案例库列表、搜索、复制、重算、删除。
- `selector-desktop/src/app/App.tsx`
  - `calculation` 和 `cases` 路由已接入，不再是占位页。
- `selector-desktop/src/app/app-test-fixtures.ts`
  - 前端测试 mock 数据和 Tauri invoke 模拟。
- `selector-desktop/src/app/App.test.tsx`
  - 覆盖计算和案例库交互。

Phase 3 目前是“功能已写完第一轮，本地验证通过，但审查未完成”的状态。

## 6. Phase 3 已通过的本地验证

以下验证在 Phase 3 当前工作区代码上已经跑过并通过：

```powershell
cmd /c pnpm.cmd run typecheck
```

结果：TypeScript 零错误。

```powershell
cmd /c pnpm.cmd run test
```

结果：1 个测试文件，10 个测试通过。

```powershell
cmd /c pnpm.cmd run build
```

结果：Vite production build 成功。

```powershell
& $env:COMSPEC /c 'call "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && set "PATH=%USERPROFILE%\.cargo\bin;%PATH%" && cargo test'
```

结果：6 个 Rust 测试通过。

```powershell
cmd /c pnpm.cmd run tauri:build
```

结果：release exe、MSI、NSIS 均构建成功。

文件长度扫描也已通过：没有 `*.rs`、`*.ts`、`*.tsx`、`*.css` 超过 300 行。

## 7. Phase 3 尚未完成的门槛

还没完成：
- Phase 3 code-reviewer 复审。
- 根据复审意见修复。
- 提交 Phase 3 commit。
- 推送 GitHub。

原本已启动 code-reviewer，但用户中途要求整理交接文档，审查未完成。接手模型应该重新派发 code-reviewer，不要沿用旧审查状态。

建议提交信息：

```text
feat: add calculation engine and case workflow
```

## 8. 继续接手的建议步骤

1. 先确认工作区：

```powershell
git status --short --branch
git diff --stat
```

2. 重新跑完整验证：

```powershell
cmd /c pnpm.cmd run typecheck
cmd /c pnpm.cmd run test
& $env:COMSPEC /c 'call "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && set "PATH=%USERPROFILE%\.cargo\bin;%PATH%" && cargo test'
cmd /c pnpm.cmd run build
cmd /c pnpm.cmd run tauri:build
```

3. 派发 code-reviewer 审查 Phase 3。

审查重点：
- 安全系数未确认时是否阻止计算并定位字段。
- 结果是否包含公式、代入值、中间值、结论、风险、来源。
- 案例库是否保存完整输入和结果快照。
- 复制案例是否不覆盖原案例。
- 重新计算是否基于已保存输入快照。
- 删除是否有二次确认。
- 单位换算覆盖是否满足 REQ-002。
- 规则模型是否足够支撑后续 Phase 4-7。

4. 修复审查问题后提交并推送：

```powershell
git add selector-desktop
git commit -m "feat: add calculation engine and case workflow"
git push origin main
```

5. Phase 3 完成后进入 Phase 4。

## 9. Phase 4 下一步方向

Phase 4：驱动与线性传动章节包。

优先实现：
- 电机篇。
- 伺服/步进计算。
- 丝杆篇。
- 同步带。
- 减速机。
- 直线模组。

验收重点：
- 覆盖矩阵中对应章节显示已实现。
- 同步带例题输出摩擦力、加速力、输出扭矩、需求转速。
- 丝杆伺服例题输出直动惯量、角加速度、加速力矩、匀速力矩、总力矩、需求转速。
- 每个结果包含 PDF 来源页码和安全系数输入记录。

Phase 3 的 `timing-belt-basic` 只是演示模块，不等于 Phase 4 的完整同步带章节包。接手模型不能把它当成“同步带已完成”。

## 10. 开发环境注意事项

Windows PowerShell 下不要用 `&&`，会报错。使用分号和 `$LASTEXITCODE`。

前端命令使用：

```powershell
cmd /c pnpm.cmd run test
cmd /c pnpm.cmd run typecheck
cmd /c pnpm.cmd run build
cmd /c pnpm.cmd run tauri:build
```

Rust 命令需要加载 VS Build Tools：

```powershell
& $env:COMSPEC /c 'call "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul && set "PATH=%USERPROFILE%\.cargo\bin;%PATH%" && cargo test'
```

开发服务器如需启动：

```powershell
cmd /c pnpm.cmd run dev -- --host 127.0.0.1
```

注意：
- 不要删除或重置用户未确认的文件。
- 不要使用 `git reset --hard`。
- 修改代码用 `apply_patch`。
- 每个 Phase 结束必须过 code-reviewer、测试、编译、功能验证。

## 11. 当前主要风险

- Phase 3 还没有完成 code-reviewer 审查。
- Phase 3 的规则选型模型目前是结构和提示级实现，完整规则要在后续章节包落地。
- 案例库当前覆盖保存、搜索、复制、重算、删除；“编辑案例”在 Product-Spec REQ-004 中出现，但 DEV-PLAN Phase 3 验收没有把编辑列为必过项，后续若严格对齐 Spec 需要补编辑。
- `timing-belt-basic` 是引擎演示模块，不是完整 PDF 同步带章节实现。
- 根目录未跟踪 docx 用途不明，接手时先确认。
