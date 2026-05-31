<p align="center">
  <a href="#简体中文"><strong>简体中文</strong></a>
  ·
  <a href="#繁體中文"><strong>繁體中文</strong></a>
  ·
  <a href="#english"><strong>English</strong></a>
</p>

# FAOS CLI

CLI for a free and open S-Platform (The world's largest PC gaming platform).

## 简体中文

FAOS CLI 是一个用 Rust 编写的交互式命令行工具，用于批量扫描、检测并更新一组以纯数字命名的 Lua 配置文件。它面向本地目录工作：你提供一个包含大量 `<NUMERIC_FILE_A>.lua`、`<NUMERIC_FILE_B>.lua` 这类文件的目录，工具会自动筛选出文件名为纯数字且扩展名为 `.lua` 的文件，读取每个文件内容，并用正则表达式判断其中是否已经存在对应的 `setStat(文件名数字, 17位账号ID)` 调用。如果目标调用不存在，工具会把这些缺失文件整理成编号列表，允许你用 `1 2 10`、`1-10`、`1 3 5-8 10` 这类复杂选择语法批量选中，然后在所选文件末尾追加标准语句。写入前会检查文件末尾是否已有换行，避免新写入内容和原内容粘连。

### 功能特性

- 账号 ID 必须是 17 位纯数字字符串。
- 首次启动会先选择界面语言，然后再输入账号 ID。
- 语言选择会持久化保存，下次启动自动使用上次语言。
- 账号 ID 会持久化保存，下次扫描可直接复用。
- 可通过参数切换语言或账号。
- 只扫描纯数字文件名的 `.lua` 文件。
- 使用正则检测 `setStat(文件名数字, 17位账号ID)`，账号 ID 带不带双引号都可识别。
- 支持复杂范围选择：空格、多范围、混合输入、自动去重。
- 越界或非法输入会提示重新输入，不会直接崩溃。
- 写入失败、读取失败、权限不足等异常会输出友好日志。
- 提供独立的 `add-appid` 子命令，用于向指定 Lua 文件追加 `addappid(数字)`。
- GitHub Actions 会在 Windows 环境运行测试并构建 `faos-cli.exe`。

### 命令结构

默认不写子命令时等价于执行 `scan`。

```powershell
faos-cli [OPTIONS] [COMMAND]
```

常用全局参数：

```powershell
-d, --dir <DIR>                 指定包含 Lua 文件的目录
-i, --account-id <ACCOUNT_ID>   指定 17 位账号 ID，并保存到本地
    --switch-account            强制重新输入账号 ID
-l, --language <LANGUAGE>       指定界面语言并保存：zh-cn、zh-tw、en
    --switch-language           强制重新选择界面语言
```

可用子命令：

```powershell
scan          扫描并注入缺失的 setStat(...)
add-appid     向选中文件追加 addappid(...)
```

### 首次使用

直接运行：

```powershell
faos-cli
```

首次运行时会先出现语言选择：

```text
请选择界面语言：
1. 简体中文
2. 繁體中文
3. English
请输入语言序号或代码（1/zh-cn，2/zh-tw，3/en）：
```

选择后会保存到本地配置。之后程序会要求输入 17 位账号 ID，再要求输入 Lua 文件目录。账号和语言都会持久化保存。

### 扫描并写入 setStat

推荐显式指定目录：

```powershell
faos-cli scan -d "D:\path\to\lua"
```

首次没有保存账号时会提示输入。你也可以直接传入账号：

```powershell
faos-cli scan -d "D:\path\to\lua" -i <17_DIGIT_ACCOUNT_ID>
```

程序会列出所有缺少目标属性的文件：

```text
1. <NUMERIC_FILE_A>.lua
2. <NUMERIC_FILE_B>.lua
3. <NUMERIC_FILE_C>.lua
```

随后可输入：

```text
1 2 10
1-10
1 3 5-8 10
```

工具会去重、校验范围，并对选中文件追加：

```lua
setStat(<NUMERIC_FILE_ID>, "<17_DIGIT_ACCOUNT_ID>")
```

### 切换账号或语言

重新选择账号：

```powershell
faos-cli scan -d "D:\path\to\lua" --switch-account
```

重新选择语言：

```powershell
faos-cli --switch-language
```

直接指定语言：

```powershell
faos-cli --language en
faos-cli --language zh-tw
faos-cli --language zh-cn
```

### 自定义 AppID 注入

进入 AppID 注入模式：

```powershell
faos-cli add-appid -d "D:\path\to\lua"
```

程序会要求输入 AppID，然后列出目录里的数字 `.lua` 文件。你可以输入序号范围：

```text
1 3 5-8
```

也可以直接输入文件名：

```text
<NUMERIC_FILE_A>.lua <NUMERIC_FILE_B>.lua
```

工具会向选中文件末尾追加：

```lua
addappid(123456)
```

### 本地配置位置

Windows 下默认保存到：

```text
%APPDATA%\faos-cli\language.txt
%APPDATA%\faos-cli\account_id.txt
```

其他环境会优先使用：

```text
$HOME/.faos-cli/
```

### 本地构建

需要安装 Rust stable 工具链：

```powershell
cargo test
cargo build --release
```

Windows 可执行文件生成位置：

```text
target\release\faos-cli.exe
```

### GitHub Actions 构建

仓库已经包含 `.github/workflows/windows-build.yml`。推送到 GitHub 后，Actions 会：

```text
1. Checkout 仓库
2. 安装 Rust stable
3. 运行 cargo test
4. 运行 cargo build --release
5. 上传 faos-cli-windows-x64.exe 作为 artifact
```

你可以在 GitHub 仓库的 Actions 页面下载 `faos-cli-windows-x64` artifact。

如果要发布正式 Release，不需要先下载 artifact 再手动上传。推送一个 `v*` tag 即可自动创建 GitHub Release，并把 `faos-cli-windows-x64.exe` 和 `faos-cli-windows-x64.exe.sha256` 上传为 Release 附件。Release title 会自动使用 `FAOS CLI <tag>`，Release notes 会自动写入资产列表、SHA256 校验值和 PowerShell 校验命令：

```powershell
git tag -a v1.0.0 -m "FAOS CLI v1.0.0"
git push origin v1.0.0
```

下载后可用 PowerShell 校验：

```powershell
(Get-FileHash -Algorithm SHA256 .\faos-cli-windows-x64.exe).Hash.ToLowerInvariant()
Get-Content .\faos-cli-windows-x64.exe.sha256
```

## 繁體中文

FAOS CLI 是一個用 Rust 編寫的互動式命令列工具，用於批次掃描、偵測並更新一組以純數字命名的 Lua 設定檔。它面向本機目錄工作：你提供一個包含大量 `<NUMERIC_FILE_A>.lua`、`<NUMERIC_FILE_B>.lua` 這類檔案的目錄，工具會自動篩選出檔名為純數字且副檔名為 `.lua` 的檔案，讀取每個檔案內容，並用正則表示式判斷其中是否已存在對應的 `setStat(檔名數字, 17位帳號ID)` 呼叫。如果目標呼叫不存在，工具會把這些缺失檔案整理成編號列表，允許你用 `1 2 10`、`1-10`、`1 3 5-8 10` 這類複雜選擇語法批次選中，然後在所選檔案末尾追加標準語句。寫入前會檢查檔案末尾是否已有換行，避免新寫入內容和原內容黏在一起。

### 功能特色

- 帳號 ID 必須是 17 位純數字字串。
- 首次啟動會先選擇介面語言，然後再輸入帳號 ID。
- 語言選擇會持久化保存，下次啟動自動使用上次語言。
- 帳號 ID 會持久化保存，下次掃描可直接複用。
- 可透過參數切換語言或帳號。
- 只掃描純數字檔名的 `.lua` 檔案。
- 使用正則偵測 `setStat(檔名數字, 17位帳號ID)`，帳號 ID 有無雙引號都可識別。
- 支援複雜範圍選擇：空格、多範圍、混合輸入、自動去重。
- 越界或非法輸入會提示重新輸入，不會直接崩潰。
- 寫入失敗、讀取失敗、權限不足等異常會輸出友善日誌。
- 提供獨立的 `add-appid` 子命令，用於向指定 Lua 檔案追加 `addappid(數字)`。
- GitHub Actions 會在 Windows 環境執行測試並建置 `faos-cli.exe`。

### 命令結構

預設不寫子命令時等同於執行 `scan`。

```powershell
faos-cli [OPTIONS] [COMMAND]
```

常用全域參數：

```powershell
-d, --dir <DIR>                 指定包含 Lua 檔案的目錄
-i, --account-id <ACCOUNT_ID>   指定 17 位帳號 ID，並保存到本機
    --switch-account            強制重新輸入帳號 ID
-l, --language <LANGUAGE>       指定介面語言並保存：zh-cn、zh-tw、en
    --switch-language           強制重新選擇介面語言
```

可用子命令：

```powershell
scan          掃描並注入缺失的 setStat(...)
add-appid     向選中檔案追加 addappid(...)
```

### 首次使用

直接執行：

```powershell
faos-cli
```

首次執行時會先出現語言選擇：

```text
請選擇介面語言：
1. 简体中文
2. 繁體中文
3. English
請輸入語言序號或代碼（1/zh-cn，2/zh-tw，3/en）：
```

選擇後會保存到本機設定。之後程式會要求輸入 17 位帳號 ID，再要求輸入 Lua 檔案目錄。帳號和語言都會持久化保存。

### 掃描並寫入 setStat

建議明確指定目錄：

```powershell
faos-cli scan -d "D:\path\to\lua"
```

首次沒有保存帳號時會提示輸入。你也可以直接傳入帳號：

```powershell
faos-cli scan -d "D:\path\to\lua" -i <17_DIGIT_ACCOUNT_ID>
```

程式會列出所有缺少目標屬性的檔案：

```text
1. <NUMERIC_FILE_A>.lua
2. <NUMERIC_FILE_B>.lua
3. <NUMERIC_FILE_C>.lua
```

隨後可輸入：

```text
1 2 10
1-10
1 3 5-8 10
```

工具會去重、校驗範圍，並對選中檔案追加：

```lua
setStat(<NUMERIC_FILE_ID>, "<17_DIGIT_ACCOUNT_ID>")
```

### 切換帳號或語言

重新選擇帳號：

```powershell
faos-cli scan -d "D:\path\to\lua" --switch-account
```

重新選擇語言：

```powershell
faos-cli --switch-language
```

直接指定語言：

```powershell
faos-cli --language en
faos-cli --language zh-tw
faos-cli --language zh-cn
```

### 自訂 AppID 注入

進入 AppID 注入模式：

```powershell
faos-cli add-appid -d "D:\path\to\lua"
```

程式會要求輸入 AppID，然後列出目錄裡的數字 `.lua` 檔案。你可以輸入序號範圍：

```text
1 3 5-8
```

也可以直接輸入檔名：

```text
<NUMERIC_FILE_A>.lua <NUMERIC_FILE_B>.lua
```

工具會向選中檔案末尾追加：

```lua
addappid(123456)
```

### 本機設定位置

Windows 下預設保存到：

```text
%APPDATA%\faos-cli\language.txt
%APPDATA%\faos-cli\account_id.txt
```

其他環境會優先使用：

```text
$HOME/.faos-cli/
```

### 本機建置

需要安裝 Rust stable 工具鏈：

```powershell
cargo test
cargo build --release
```

Windows 可執行檔生成位置：

```text
target\release\faos-cli.exe
```

### GitHub Actions 建置

倉庫已包含 `.github/workflows/windows-build.yml`。推送到 GitHub 後，Actions 會：

```text
1. Checkout 倉庫
2. 安裝 Rust stable
3. 執行 cargo test
4. 執行 cargo build --release
5. 上傳 faos-cli-windows-x64.exe 作為 artifact
```

你可以在 GitHub 倉庫的 Actions 頁面下載 `faos-cli-windows-x64` artifact。

如果要發布正式 Release，不需要先下載 artifact 再手動上傳。推送一個 `v*` tag 即可自動建立 GitHub Release，並把 `faos-cli-windows-x64.exe` 和 `faos-cli-windows-x64.exe.sha256` 上傳為 Release 附件。Release title 會自動使用 `FAOS CLI <tag>`，Release notes 會自動寫入資產列表、SHA256 校驗值和 PowerShell 校驗命令：

```powershell
git tag -a v1.0.0 -m "FAOS CLI v1.0.0"
git push origin v1.0.0
```

下載後可用 PowerShell 校驗：

```powershell
(Get-FileHash -Algorithm SHA256 .\faos-cli-windows-x64.exe).Hash.ToLowerInvariant()
Get-Content .\faos-cli-windows-x64.exe.sha256
```

## English

FAOS CLI is an interactive command-line tool written in Rust for scanning, detecting, and updating numeric Lua configuration files in batches. It works against a local directory: you provide a folder containing files such as `<NUMERIC_FILE_A>.lua` and `<NUMERIC_FILE_B>.lua`, and the tool automatically filters files whose names are digits only and whose extension is `.lua`. It reads each file and uses a regular expression to check whether the matching `setStat(file-name-number, 17-digit-account-id)` call already exists. If the target call is missing, the tool prints the missing files as a numbered list, accepts complex selection input such as `1 2 10`, `1-10`, or `1 3 5-8 10`, deduplicates and validates the selection, then appends the standard statement to the selected files. Before writing, it checks whether the file already ends with a newline so the new code never sticks to the previous content.

### Features

- Account ID must be a 17-digit numeric string.
- First launch asks for interface language before asking for the account ID.
- Language selection is persisted and reused on the next launch.
- Account ID is persisted and can be reused on later scans.
- Language and account can be changed through CLI flags.
- Only numeric `.lua` file names are scanned.
- Regex detection supports `setStat(file_id, account_id)` with or without quotes around the account ID.
- Complex selection input supports spaces, inclusive ranges, mixed input, and automatic deduplication.
- Invalid or out-of-range input asks you to retry instead of crashing.
- File read, write, permission, and detection failures are reported with friendly logs.
- Dedicated `add-appid` subcommand appends `addappid(number)` to selected Lua files.
- GitHub Actions tests and builds `faos-cli.exe` on Windows.

### Command Layout

When no subcommand is provided, the tool behaves as if `scan` was used.

```powershell
faos-cli [OPTIONS] [COMMAND]
```

Common global options:

```powershell
-d, --dir <DIR>                 Directory containing Lua files
-i, --account-id <ACCOUNT_ID>   17-digit account ID, saved locally after validation
    --switch-account            Force entering a new account ID
-l, --language <LANGUAGE>       Set and save interface language: zh-cn, zh-tw, en
    --switch-language           Force choosing the interface language again
```

Available subcommands:

```powershell
scan          Scan and inject missing setStat(...)
add-appid     Append addappid(...) to selected files
```

### First Run

Run:

```powershell
faos-cli
```

On first launch, language selection appears first:

```text
Select interface language:
1. 简体中文
2. 繁體中文
3. English
Enter language number or code (1/zh-cn, 2/zh-tw, 3/en):
```

After selection, the language is saved locally. The program then asks for the 17-digit account ID and the Lua directory path. Both language and account ID are persisted.

### Scan And Write setStat

Recommended usage with an explicit directory:

```powershell
faos-cli scan -d "D:\path\to\lua"
```

If no account ID has been saved, the tool prompts for it. You can also provide it directly:

```powershell
faos-cli scan -d "D:\path\to\lua" -i <17_DIGIT_ACCOUNT_ID>
```

The tool lists every file missing the target entry:

```text
1. <NUMERIC_FILE_A>.lua
2. <NUMERIC_FILE_B>.lua
3. <NUMERIC_FILE_C>.lua
```

Then enter a selection:

```text
1 2 10
1-10
1 3 5-8 10
```

The selected files receive:

```lua
setStat(<NUMERIC_FILE_ID>, "<17_DIGIT_ACCOUNT_ID>")
```

### Switch Account Or Language

Choose a new account ID:

```powershell
faos-cli scan -d "D:\path\to\lua" --switch-account
```

Choose language again:

```powershell
faos-cli --switch-language
```

Set language directly:

```powershell
faos-cli --language en
faos-cli --language zh-tw
faos-cli --language zh-cn
```

### Custom AppID Injection

Start AppID injection mode:

```powershell
faos-cli add-appid -d "D:\path\to\lua"
```

The tool asks for an AppID, then lists numeric `.lua` files in the directory. You can choose by range:

```text
1 3 5-8
```

Or by direct file name:

```text
<NUMERIC_FILE_A>.lua <NUMERIC_FILE_B>.lua
```

The selected files receive:

```lua
addappid(123456)
```

### Local Config Location

On Windows, config is saved under:

```text
%APPDATA%\faos-cli\language.txt
%APPDATA%\faos-cli\account_id.txt
```

On other environments, the tool prefers:

```text
$HOME/.faos-cli/
```

### Local Build

Install the Rust stable toolchain, then run:

```powershell
cargo test
cargo build --release
```

Windows executable output:

```text
target\release\faos-cli.exe
```

### GitHub Actions Build

The repository includes `.github/workflows/windows-build.yml`. After pushing to GitHub, Actions will:

```text
1. Checkout the repository
2. Install Rust stable
3. Run cargo test
4. Run cargo build --release
5. Upload faos-cli-windows-x64.exe as an artifact
```

Download the `faos-cli-windows-x64` artifact from the repository's Actions page.

For an official GitHub Release, you do not need to download the artifact and upload it manually. Push a `v*` tag and the workflow will create the GitHub Release automatically, then attach `faos-cli-windows-x64.exe` and `faos-cli-windows-x64.exe.sha256`. The Release title is generated as `FAOS CLI <tag>`, and the Release notes include the asset list, SHA256 checksum, and PowerShell verification command:

```powershell
git tag -a v1.0.0 -m "FAOS CLI v1.0.0"
git push origin v1.0.0
```

After downloading, verify with PowerShell:

```powershell
(Get-FileHash -Algorithm SHA256 .\faos-cli-windows-x64.exe).Hash.ToLowerInvariant()
Get-Content .\faos-cli-windows-x64.exe.sha256
```
