@{
    # 模組根指令碼檔
    RootModule = 'AgentHelpers.psm1'

    # 模組版本號
    ModuleVersion = '1.0.0'

    # 唯一標識 GUID
    GUID = '9d4f40f0-826a-4d3e-90cb-4672e87c0e64'

    # 作者與公司資訊
    Author = 'Agent-Infra'
    CompanyName = 'Community'
    Copyright = '(c) 2026. All rights reserved.'

    # 模組描述
    Description = 'Agent-Native High-Density CLI Helpers: Low-token, structured CLI diagnostics for AI agents.'

    # PowerShell 引擎相容版本 (7.0+)
    PowerShellVersion = '7.0'

    # 匯出的函式清單 (明確宣告以利 PowerShell Auto-loading 索引)
    FunctionsToExport = @(
        'outline',
        'lsj',
        'topfiles',
        'pingj',
        'portj',
        'psj',
        'killp',
        'start-gui'
    )

    # 匯出的 Cmdlet、別名、變數
    CmdletsToExport = @()
    VariablesToExport = @()
    AliasesToExport = @()

    # 私有資料
    PrivateData = @{
        PSData = @{
            Tags = @('Agent', 'CLI', 'Toolchain', 'Diagnostics')
            ProjectUri = 'https://github.com/your-org/agent-infra'
        }
    }
}
