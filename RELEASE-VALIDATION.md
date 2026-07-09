# 发布验收记录

## v0.1.0 - 2026-07-09

**对应 Git 提交**：本记录所在提交

**本次发布范围**：
- Windows 桌面离线应用。
- Phase 9 交付内容：结果页报告导出、开发侧覆盖/回归验证、Windows 安装包。
- 打包脚本增强：自动定位 Visual Studio C++ 环境、从任意工作目录启动、清理旧 bundle、输出 SHA256 并执行隐私扫描。

## 打包产物

| 产物 | 文件大小 | SHA256 |
| --- | ---: | --- |
| `selector-desktop/src-tauri/target/release/bundle/msi/Selector Desktop_0.1.0_x64_en-US.msi` | 6,762,496 bytes | `DC0261C73E3019439770A581034146F5B3008F9D2CC7E974D3D4B84A789EA385` |
| `selector-desktop/src-tauri/target/release/bundle/nsis/Selector Desktop_0.1.0_x64-setup.exe` | 4,694,769 bytes | `17ABA07AB462BE5CD712BFC3A982D078CAEF539D18C2AB3C442D128C5A0B8FD8` |

## 验证命令

以下验证均在 2026-07-09 通过。

```powershell
cmd /c pnpm.cmd test
```

结果：7 个前端测试文件通过，23 条前端测试用例通过。

```powershell
cargo test
```

结果：15 条 Rust 后端测试通过，其中包含 `qa::regression_runner::tests::regression_runner_passes_all_fixture_groups`，说明 QA 回归样例执行器已被测试覆盖。

```powershell
cmd /c pnpm.cmd package:windows
```

结果：MSI 安装包和 NSIS 安装包均已生成，脚本自动输出 SHA256 并通过隐私扫描。

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

本次未自动执行安装启动冒烟测试。

原因：当前应用没有独立的测试数据目录开关。直接启动安装后的程序会写入正常的 `%APPDATA%\com.sckj.selector\selector.db`，不符合“测试不写生产数据目录”的规则。后续要补严格安装冒烟测试，应先增加明确的测试数据目录开关，再执行静默安装、启动、数据库初始化和卸载闭环。

## 结论

发布验收状态：**通过，有注意事项**。

当前 Windows 安装包可以用于小组内部试用。注意：目前还没有配置代码签名，所以安装时 Windows 可能会出现未签名软件的安全提示。这个不是功能错误，但正式对外分发前应该补代码签名。
