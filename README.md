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
    --cli                       强制使用 CLI 模式（不使用 TUI）
```

可用子命令：

```powershell
scan          扫描并注入缺失的 setStat(...)
add-appid     向选中文件追加 addappid(...)
tui           强制启动 TUI 界面（默认）
```

### TUI 交互界面（默认模式）

不带任何命令行参数启动时，进入 TUI（终端用户界面）模式：

```powershell
faos-cli
```

#### 界面布局

屏幕从上到下依次为：

- **标题栏** — 显示程序名、当前语言、Ctrl+S 设置、Q 退出
- **目录行** — 当前 Lua 文件所在目录路径
- **内容区** — 左右分栏，两个带边框的面板：

```
┌─ 文件列表 ───────────┐  ┌─ 操作面板 ──────────────┐
│ ○ 100.lua            │  │ F1:扫描setStat         │
│ ● 101.lua            │  │ F2:添加AppID           │
│ ○ 102.lua            │  │ ───────────────────────│
│ ...                  │  │ 账号ID/AppID: 输入框    │
│                      │  │                        │
│                      │  │ → Enter 执行           │
└──────────────────────┘  └────────────────────────┘
```

- **命令栏** — 底部显示当前状态和快捷键提示

#### 全部按键绑定

| 按键 | 作用范围 | 功能 |
|------|----------|------|
| **全局** | | |
| `F1` | 任何位置 | 切换到 **扫描 setStat** 模式 |
| `F2` | 任何位置 | 切换到 **添加 AppID** 模式 |
| `F5` | 任何位置 | 重新加载 Lua 文件列表 |
| `Ctrl+S` | 主界面 | 打开设置面板 |
| `Ctrl+Q` / `Q` | 任何位置 | 退出程序 |
| `Tab` | 主界面 | 切换焦点面板（文件列表 ↔ 操作面板） |
| `Esc` | 主界面 | 焦点在操作面板时返回文件列表；焦点在文件列表时退出 |
| | | |
| **文件列表面板** | | |
| `↑` / `k` | 文件列表 | 光标上移 |
| `↓` / `j` | 文件列表 | 光标下移 |
| `Space` / `Enter` | 文件列表 | 切换当前文件的选中状态（●/○） |
| `A` | 文件列表 | 全选所有文件 |
| `N` | 文件列表 | 取消全选 |
| `鼠标滚轮` | 文件列表 | 滚动文件列表 |
| `鼠标点击` | 文件列表 | 焦点切换到文件列表 |
| | | |
| **操作面板** | | |
| `0-9` / 字符输入 | 操作面板 | 输入账号 ID（Scan 模式）或 AppID（AddAppid 模式） |
| `Backspace` | 操作面板 | 删除最后输入的字符 |
| `Enter` | 操作面板 | 执行当前操作（扫描写入 / 注入 AppID） |
| | | |
| **设置面板** | | |
| `L` | 设置面板 | 打开语言选择界面 |
| `D` | 设置面板 | 打开目录输入界面 |
| `Esc` / `Q` | 设置面板 | 返回主界面 |
| | | |
| **目录输入界面** | | |
| 字符输入 | 目录输入 | 输入目录路径 |
| `Backspace` | 目录输入 | 删除最后输入的字符 |
| `Enter` | 目录输入 | 确认目录并返回主界面 |
| `Esc` | 目录输入 | 取消修改，返回设置面板 |
| | | |
| **语言选择界面** | | |
| `↑` / `k` | 语言选择 | 光标上移 |
| `↓` / `j` | 语言选择 | 光标下移 |
| `Enter` | 语言选择 | 确认选择当前高亮的语言 |
| `1` / `2` / `3` | 语言选择 | 直接选择 简体中文 / 繁體中文 / English |
| `Esc` / `Q` | 语言选择 | 退出程序 |

#### 详细操作流程

**1. 首次启动**

直接运行 `faos-cli`，先选择语言（↑↓ 移动，Enter 确认），进入主界面后自动加载上次保存的配置（账号 ID、目录）。如果尚未保存过目录，需要先设置目录（`Ctrl+S` → `D` 输入路径）。

**2. 扫描 setStat（F1 模式）**

按 `F1` 切换到扫描模式。左侧文件列表只显示**尚未包含 `setStat`** 的文件（扫描前需已填入账号 ID）。选中目标文件（`Space` 切换 ●/○），按 `Tab` 切换到操作面板，确认账号 ID 无误后按 `Enter` 执行。写入完成后重新扫描，已写入的文件会从列表消失。

**3. 添加 AppID（F2 模式）**

按 `F2` 切换到添加 AppID 模式。左侧文件列表显示目录中**所有**数字命名的 `.lua` 文件。选中文件后切换到操作面板，输入 AppID（纯数字），按 `Enter` 执行。工具会向每个选中文件追加 `addappid(<AppID>)`。

**4. 更改目录**

`Ctrl+S` 打开设置，按 `D` 进入目录输入界面，输入完整路径（如 `D:\path\to\lua`），`Enter` 确认。路径会自动规范化和持久化保存。

**5. 重新加载文件**

任何时候按 `F5` 可重新扫描目录。切换操作模式（`F1`/`F2`）也会自动更新文件列表过滤条件。

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
    --cli                       強制使用 CLI 模式（不使用 TUI）
```

可用子命令：

```powershell
scan          掃描並注入缺失的 setStat(...)
add-appid     向選中檔案追加 addappid(...)
tui           強制啟動 TUI 介面（預設）
```

### TUI 互動介面（預設模式）

不帶任何命令列參數啟動時，進入 TUI（終端使用者介面）模式：

```powershell
faos-cli
```

#### 介面佈局

螢幕從上到下依次為：

- **標題列** — 顯示程式名、目前語言、Ctrl+S 設定、Q 退出
- **目錄行** — 目前 Lua 檔案所在目錄路徑
- **內容區** — 左右分欄，兩個帶邊框的面板：

```
┌─ 檔案列表 ───────────┐  ┌─ 操作面板 ──────────────┐
│ ○ 100.lua            │  │ F1:掃描setStat         │
│ ● 101.lua            │  │ F2:新增AppID           │
│ ○ 102.lua            │  │ ──────────────────────│
│ ...                  │  │ 帳號ID/AppID: 輸入框   │
│                      │  │                       │
│                      │  │ → Enter 執行           │
└──────────────────────┘  └────────────────────────┘
```

- **命令列** — 底部顯示目前狀態和快速鍵提示

#### 全部按鍵綁定

| 按鍵 | 作用範圍 | 功能 |
|------|----------|------|
| **全域** | | |
| `F1` | 任何位置 | 切換到 **掃描 setStat** 模式 |
| `F2` | 任何位置 | 切換到 **新增 AppID** 模式 |
| `F5` | 任何位置 | 重新載入 Lua 檔案列表 |
| `Ctrl+S` | 主介面 | 開啟設定面板 |
| `Ctrl+Q` / `Q` | 任何位置 | 離開程式 |
| `Tab` | 主介面 | 切換焦點面板（檔案列表 ↔ 操作面板） |
| `Esc` | 主介面 | 焦點在操作面板時返回檔案列表；焦點在檔案列表時離開 |
| | | |
| **檔案列表面板** | | |
| `↑` / `k` | 檔案列表 | 游標上移 |
| `↓` / `j` | 檔案列表 | 游標下移 |
| `Space` / `Enter` | 檔案列表 | 切換目前檔案的選取狀態（●/○） |
| `A` | 檔案列表 | 全選所有檔案 |
| `N` | 檔案列表 | 取消全選 |
| `滑鼠滾輪` | 檔案列表 | 捲動檔案列表 |
| `滑鼠點擊` | 檔案列表 | 焦點切換到檔案列表 |
| | | |
| **操作面板** | | |
| `0-9` / 字元輸入 | 操作面板 | 輸入帳號 ID（Scan 模式）或 AppID（AddAppid 模式） |
| `Backspace` | 操作面板 | 刪除最後輸入的字元 |
| `Enter` | 操作面板 | 執行目前操作（掃描寫入 / 注入 AppID） |
| | | |
| **設定面板** | | |
| `L` | 設定面板 | 開啟語言選擇介面 |
| `D` | 設定面板 | 開啟目錄輸入介面 |
| `Esc` / `Q` | 設定面板 | 返回主介面 |
| | | |
| **目錄輸入介面** | | |
| 字元輸入 | 目錄輸入 | 輸入目錄路徑 |
| `Backspace` | 目錄輸入 | 刪除最後輸入的字元 |
| `Enter` | 目錄輸入 | 確認目錄並返回主介面 |
| `Esc` | 目錄輸入 | 取消修改，返回設定面板 |
| | | |
| **語言選擇介面** | | |
| `↑` / `k` | 語言選擇 | 游標上移 |
| `↓` / `j` | 語言選擇 | 游標下移 |
| `Enter` | 語言選擇 | 確認選擇目前反白的語言 |
| `1` / `2` / `3` | 語言選擇 | 直接選擇 简体中文 / 繁體中文 / English |
| `Esc` / `Q` | 語言選擇 | 離開程式 |

#### 詳細操作流程

**1. 首次啟動**

直接執行 `faos-cli`，先選擇語言（↑↓ 移動，Enter 確認），進入主介面後自動載入上次保存的設定（帳號 ID、目錄）。如果尚未保存目錄，需要先設定目錄（`Ctrl+S` → `D` 輸入路徑）。

**2. 掃描 setStat（F1 模式）**

按 `F1` 切換到掃描模式。左側檔案列表只顯示**尚未包含 `setStat`** 的檔案（掃描前需已填入帳號 ID）。選取目標檔案（`Space` 切換 ●/○），按 `Tab` 切換到操作面板，確認帳號 ID 無誤後按 `Enter` 執行。寫入完成後重新掃描，已寫入的檔案會從列表消失。

**3. 新增 AppID（F2 模式）**

按 `F2` 切換到新增 AppID 模式。左側檔案列表顯示目錄中**所有**數字命名的 `.lua` 檔案。選取檔案後切換到操作面板，輸入 AppID（純數字），按 `Enter` 執行。工具會向每個選取檔案追加 `addappid(<AppID>)`。

**4. 變更目錄**

`Ctrl+S` 開啟設定，按 `D` 進入目錄輸入介面，輸入完整路徑（如 `D:\path\to\lua`），`Enter` 確認。路徑會自動規範化和持久化保存。

**5. 重新載入檔案**

任何時候按 `F5` 可重新掃描目錄。切換操作模式（`F1`/`F2`）也會自動更新檔案列表篩選條件。

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
    --cli                       Force CLI mode (no TUI)
```

Available subcommands:

```powershell
scan          Scan and inject missing setStat(...)
add-appid     Append addappid(...) to selected files
tui          Force start TUI interface (default)
```

### TUI Interactive Mode (Default)

When launched without any arguments, the program starts in TUI (Terminal User Interface) mode:

```powershell
faos-cli
```

#### Layout

The screen is organized top to bottom as:

- **Header** — program name, current language, Ctrl+S (Settings), Q (Quit)
- **Directory** — path to the folder containing Lua files
- **Content area** — two bordered panels side by side:

```
┌─ File List ───────────┐  ┌─ Operation Panel ──────┐
│ ○ 100.lua             │  │ F1:Scan setStat        │
│ ● 101.lua             │  │ F2:AddAppID            │
│ ○ 102.lua             │  │ ───────────────────────│
│ ...                   │  │ Account ID/AppID: input│
│                       │  │                        │
│                       │  │ → Enter to Execute     │
└───────────────────────┘  └────────────────────────┘
```

- **Command bar** — status message and keybinding hints at the bottom

#### Full Key Bindings

| Key | Scope | Action |
|-----|-------|--------|
| **Global** | | |
| `F1` | Anywhere | Switch to **Scan setStat** mode |
| `F2` | Anywhere | Switch to **Add AppID** mode |
| `F5` | Anywhere | Reload the Lua file list |
| `Ctrl+S` | Main screen | Open the Settings panel |
| `Ctrl+Q` / `Q` | Anywhere | Quit the program |
| `Tab` | Main screen | Toggle focus between File List and Operation Panel |
| `Esc` | Main screen | Focus on Operation Panel → back to File List; focus on File List → quit |
| | | |
| **File List Panel** | | |
| `↑` / `k` | File list | Move cursor up |
| `↓` / `j` | File list | Move cursor down |
| `Space` / `Enter` | File list | Toggle selection of the current file (●/○) |
| `A` | File list | Select all files |
| `N` | File list | Deselect all files |
| `Mouse scroll` | File list | Scroll through the file list |
| `Mouse click` | File list | Switch focus to file list |
| | | |
| **Operation Panel** | | |
| `0-9` / character input | Operation panel | Type account ID (Scan mode) or AppID (AddAppid mode) |
| `Backspace` | Operation panel | Delete the last character |
| `Enter` | Operation panel | Execute the current operation (scan & write / inject AppID) |
| | | |
| **Settings Panel** | | |
| `L` | Settings panel | Open language selection screen |
| `D` | Settings panel | Open directory input screen |
| `Esc` / `Q` | Settings panel | Return to main screen |
| | | |
| **Directory Input Screen** | | |
| Character input | Directory input | Type a directory path |
| `Backspace` | Directory input | Delete the last character |
| `Enter` | Directory input | Confirm the directory and return to main screen |
| `Esc` | Directory input | Cancel changes, return to Settings panel |
| | | |
| **Language Selection Screen** | | |
| `↑` / `k` | Language selection | Move cursor up |
| `↓` / `j` | Language selection | Move cursor down |
| `Enter` | Language selection | Confirm the highlighted language |
| `1` / `2` / `3` | Language selection | Directly pick 简体中文 / 繁體中文 / English |
| `Esc` / `Q` | Language selection | Quit the program |

#### Walkthrough

**1. First Launch**

Run `faos-cli`. Pick a language (↑↓ to move, Enter to confirm). The main screen loads. Saved settings (account ID, directory) are restored automatically. If no directory has been saved, open Settings (`Ctrl+S`), press `D`, and type a path.

**2. Scan setStat (F1 Mode)**

Press `F1` to enter Scan mode. The file list on the left shows only files that **do not already contain a `setStat`** call (requires an account ID first). Toggle selections with `Space` (● = selected). Press `Tab` to switch to the operation panel, verify the account ID, and press `Enter` to execute. After writing completes, the tool rescans automatically and the written files disappear from the list.

**3. Add AppID (F2 Mode)**

Press `F2` to enter AddAppid mode. The file list shows **all** numeric `.lua` files in the directory. Select the target files, switch to the operation panel, type the AppID (digits only), and press `Enter`. The tool appends `addappid(<AppID>)` to every selected file.

**4. Change Directory**

Press `Ctrl+S` to open Settings, then `D` to enter the directory input screen. Type the full path (e.g. `D:\path\to\lua`) and press `Enter`. The path is canonicalized and persisted automatically.

**5. Reload Files**

Press `F5` at any time to re-scan the directory. Switching operation modes (`F1`/`F2`) also refreshes the file list filter automatically.

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
