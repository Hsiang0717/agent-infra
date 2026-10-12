@{
    # 模組根指令碼檔
    RootModule = 'KeyBindings.psm1'

    # 模組版本號
    ModuleVersion = '1.0.0'

    # 唯一標識 GUID
    GUID = '3c8e4720-6d1a-4f51-b8ef-572cb0a1e3b8'

    # 作者與公司資訊
    Author = 'Agent-Infra'
    CompanyName = 'Community'
    Copyright = '(c) 2026. All rights reserved.'

    # 模組描述
    Description = 'PowerShell Interactive UX & PSReadLine Tuning (Context-aware RightArrow, prefix history search, inline tab completion)'

    # PowerShell 引擎相容版本 (7.0+)
    PowerShellVersion = '7.0'

    # 所需的相依模組 (確保 PSReadLine 已載入)
    RequiredModules = @('PSReadLine')

    # 不匯出一般函式（純環境調校模組，於 import 時自動套用）
    FunctionsToExport = @()
    CmdletsToExport = @()
    VariablesToExport = @()
    AliasesToExport = @()

    # 私有資料
    PrivateData = @{
        PSData = @{
            Tags = @('PSReadLine', 'KeyBindings', 'UX', 'Terminal')
            ProjectUri = 'https://github.com/your-org/agent-infra'
        }
    }
}
