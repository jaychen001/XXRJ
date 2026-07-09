# 发布验收记录

## v0.1.0 - 2026-07-09

**对应 Git 提交**：本记录所在提交

**本次发布范围**：
- Windows 桌面离线应用。
- Phase 9 交付内容：结果页报告导出、开发侧覆盖/回归验证、Windows 安装包。
- 打包脚本增强：自动定位 Visual Studio C++ 环境、从任意工作目录启动、清理旧 bundle、输出 SHA256 并执行隐私扫描。
- 冒烟测试增强：通过 `SELECTOR_DESKTOP_DATA_DIR` 将数据库写入隔离临时目录。

## 打包产物

| 产物 | 文件大小 | SHA256 |
| --- | ---: | --- |
| `selector-desktop/src-tauri/target/release/bundle/msi/Selector Desktop_0.1.0_x64_en-US.msi` | 6,754,304 bytes | `0B352524E558B3C277EA957B2396FF7A5C9010BAE5EA8EF97683AA1B8503CF69` |
| `selector-desktop/src-tauri/target/release/bundle/nsis/Selector Desktop_0.1.0_x64-setup.exe` | 4,691,458 bytes | `AD25B9829869E3A675695D9FBD849B20CB1B33818BC64A26C764794E97C367B4` |

## 验证命令

以下验证均在 2026-07-09 通过。

```powershell
cmd /c pnpm.cmd test
```

结果：7 个前端测试文件通过，23 条前端测试用例通过。

```powershell
cargo test
```

结果：18 条 Rust 后端测试通过，其中包含 `qa::regression_runner::tests::regression_runner_passes_all_fixture_groups`，并覆盖数据库隔离目录解析。

```powershell
cmd /c pnpm.cmd package:windows
```

结果：MSI 安装包和 NSIS 安装包均已生成，脚本自动输出 SHA256 并通过隐私扫描。

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File selector-desktop/scripts/smoke-release-app.ps1 -StartupSeconds 5
```

结果：release exe 启动成功，临时数据库创建在 `.codex/release-smoke/data/selector.db`，进程已关闭，临时目录已清理。

## 隐私审计

审计目录：

```text
selector-desktop/src-tauri/target/release/bundle
```

审计结果：
- `selector-desktop/scripts/validate-windows-package.ps1` 已自动扫描打包目录。
- 打包产物中未发现 `.env`、`.db`、`.sqlite`、凭据、密钥、令牌、会话文件或 `selector.db`。
- 未发现开发路径、用户路径或常见 API Key 标记：
  - `D:\codex`
  - `C:\Users`
  - `Users/`
  - `SCKJ`
  - `sk-ant-`
  - `sk-proj-`
  - `ANTHROPIC_API_KEY`
  - `OPENAI_API_KEY`

## 安装包冒烟测试

测试安装包：

```text
selector-desktop/src-tauri/target/release/bundle/nsis/Selector Desktop_0.1.0_x64-setup.exe
```

本次已执行 release exe 启动冒烟测试，未执行静默安装/卸载闭环。

执行方式：
- 设置 `SELECTOR_DESKTOP_DATA_DIR` 到 `.codex/release-smoke/data`。
- 启动 `selector-desktop/src-tauri/target/release/selector-desktop.exe`。
- 等待 5 秒，确认进程未退出。
- 确认 `selector.db` 在隔离目录创建。
- 停止进程并清理临时目录。

观察结果：
- release exe 冒烟测试退出码：`0`。
- 数据库初始化位置：`.codex/release-smoke/data/selector.db`。
- 未写入正常 `%APPDATA%\com.sckj.selector\selector.db`。

## 结论

发布验收状态：**通过，有注意事项**。

当前 Windows 安装包可以用于小组内部试用。注意：目前还没有配置代码签名，所以安装时 Windows 可能会出现未签名软件的安全提示。这个不是功能错误，但正式对外分发前应该补代码签名。
