# 发布验收记录

## v0.1.0 - 2026-07-08

**对应 Git 提交**：`4e93953`

**本次发布范围**：
- Windows 桌面离线应用。
- Phase 9 交付内容：报告导出、QA 覆盖审计、回归样例执行器、Windows 安装包。

## 打包产物

| 产物 | 文件大小 | SHA256 |
| --- | ---: | --- |
| `selector-desktop/src-tauri/target/release/bundle/msi/Selector Desktop_0.1.0_x64_en-US.msi` | 6,606,848 bytes | `73DD856DF6C16C6ED403FB697B54672E58BB3AE6A28A61433A4E410A3CB3DD7D` |
| `selector-desktop/src-tauri/target/release/bundle/nsis/Selector Desktop_0.1.0_x64-setup.exe` | 4,598,562 bytes | `A043CB79CF2B36ACE45F91287EA76520A168F35036EE3DED0310248F05322BC0` |

## 验证命令

以下验证均在 2026-07-08 通过。

```powershell
cmd /c pnpm.cmd test
```

结果：7 个前端测试文件通过，19 条前端测试用例通过。

```powershell
cargo test
```

结果：15 条 Rust 后端测试通过，其中包含 `qa::regression_runner::tests::regression_runner_passes_all_fixture_groups`，说明 QA 回归样例执行器已被测试覆盖。

```powershell
cmd /c pnpm.cmd package:windows
```

结果：MSI 安装包和 NSIS 安装包均已生成。

## 隐私审计

审计目录：

```text
selector-desktop/src-tauri/target/release/bundle
```

审计结果：
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

测试步骤：
- 静默安装到 `.codex/release-smoke/install`。
- 启动安装后的 `selector-desktop.exe`。
- 确认应用进程能启动，并在测试窗口期内持续运行。
- 确认本地 SQLite 数据库能初始化。
- 测试结束后静默卸载。

观察结果：
- 安装器退出码：`0`。
- 安装后的应用进程能启动，并在冒烟测试窗口期内保持运行。
- 卸载器退出码：`0`。
- 数据库初始化位置：`%APPDATA%\com.sckj.selector\selector.db`。
- 数据库健康状态：已应用 2 个 migration，存在 18 张应用表。

说明：Tauri 会把应用数据目录解析到 Windows 正常的 Roaming 应用数据路径。即使启动进程时临时设置了 `APPDATA` 和 `LOCALAPPDATA`，本次测试中数据库仍落到了正常的 `%APPDATA%\com.sckj.selector\selector.db`。如果以后要做严格隔离的自动化安装冒烟测试，需要在应用里增加明确的测试数据目录开关。

## 结论

发布验收状态：**通过，有注意事项**。

当前 Windows 安装包可以用于小组内部试用。注意：目前还没有配置代码签名，所以安装时 Windows 可能会出现未签名软件的安全提示。这个不是功能错误，但正式对外分发前应该补代码签名。
