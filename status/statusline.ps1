# Disable progress bar to speed up web requests or execution if any
$ProgressPreference = 'SilentlyContinue'
$ErrorActionPreference = 'SilentlyContinue'
$global:LASTEXITCODE = 0

try {
    function script:Ensure-GitModule {
        if (-not (Get-Command -Name Get-GitStatus -ErrorAction SilentlyContinue)) {
            $modPath = Join-Path $PSScriptRoot 'Status.Git.psm1'
            if (Test-Path -LiteralPath $modPath) {
                Import-Module $modPath -Force -ErrorAction SilentlyContinue
            }
        }
    }

    function script:Ensure-PowerModule {
        if (-not (Get-Command -Name Get-PowerStatus -ErrorAction SilentlyContinue)) {
            $modPath = Join-Path $PSScriptRoot 'Status.Power.psm1'
            if (Test-Path -LiteralPath $modPath) {
                Import-Module $modPath -Force -ErrorAction SilentlyContinue
            }
        }
    }

    $SHOW_LEGEND = $args | Where-Object { $_ -in @('--legend', '-l', 'legend') }
    if ($SHOW_LEGEND) {
        Write-Output 'Antigravity Statusline Legend'
        Write-Output 'READY     Agent is idle'
        Write-Output 'THINKING  Agent is reasoning'
        Write-Output 'WORKING   Agent is executing'
        Write-Output 'TOOL      Tool call is active'
        Write-Output 'CTX       Context window usage'
        Write-Output '5H / 7D   Quota remaining and reset time'
        exit 0
    }

    # Set Output Encoding to UTF-8 to support nerd font icons on Windows
    try { [Console]::OutputEncoding = [System.Text.Encoding]::UTF8 } catch {}



    # Read JSON input from stdin or pipeline safely
    $inputJson = ""
    try {
        $pipelineStr = $input | Out-String
        if ($pipelineStr -and $pipelineStr.Trim().Length -gt 0) {
            $inputJson = $pipelineStr
        }
    } catch {}
    if (-not $inputJson -or $inputJson.Trim().Length -eq 0) {
        try {
            if ([Console]::IsInputRedirected) {
                $inputJson = [Console]::In.ReadToEnd()
            }
        } catch {}
    }

    # Parse JSON safely, create empty object fallback if input is null/empty
    $data = $null
    if ($inputJson -and $inputJson.Trim().Length -gt 0) {
        try {
            $data = ConvertFrom-Json $inputJson
        } catch {}
    }
    if ($data -eq $null) {
        $data = [PSCustomObject]@{
            agent_state = "idle"
            terminal_width = 100
        }
    }

function Safe-Double([object]$val, [double]$default = 0.0) {
    if ($val -eq $null) { return $default }
    $d = 0.0
    if ([double]::TryParse([string]$val, [System.Globalization.NumberStyles]::Any, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$d)) {
        if ([double]::IsNaN($d) -or [double]::IsInfinity($d)) { return $default }
        return $d
    }
    return $default
}

function Clamp-Double([double]$val, [double]$min, [double]$max) {
    if ([double]::IsNaN($val) -or [double]::IsInfinity($val)) { return $min }
    return [Math]::Max($min, [Math]::Min($max, $val))
}

function Safe-Bool([object]$val, [bool]$default = $false) {
    if ($val -is [bool]) { return [bool]$val }
    if ($val -eq $null) { return $default }
    $parsed = $false
    if ([bool]::TryParse(([string]$val).Trim(), [ref]$parsed)) { return $parsed }
    return $default
}

$script:ANSI_CSI_REGEX = [regex]::new('\x1b\[[0-?]*[ -/]*[@-~]', [System.Text.RegularExpressions.RegexOptions]::Compiled)
$script:CTRL_CHARS_REGEX = [regex]::new('[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]', [System.Text.RegularExpressions.RegexOptions]::Compiled)

function Clean-DisplayText([object]$val, [int]$maxLen = 0) {
    if ($val -eq $null) { return "" }
    $text = $script:ANSI_CSI_REGEX.Replace([string]$val, '')
    $text = $script:CTRL_CHARS_REGEX.Replace($text, '')
    if ($maxLen -gt 0 -and $text.Length -gt $maxLen) {
        if ($maxLen -le 3) { return $text.Substring(0, $maxLen) }
        return $text.Substring(0, $maxLen - 3) + '...'
    }
    return $text
}

function Safe-Quota([object]$quotaObj) {
    if ($quotaObj -eq $null -or $quotaObj.remaining_fraction -eq $null) { return -1 }
    $val = Safe-Double $quotaObj.remaining_fraction -999.0
    if ($val -eq -999.0 -or $val -lt 0) { return -1 }
    return [Math]::Round((Clamp-Double ($val * 100) 0 100), 1)
}

# Extract properties with fallbacks
$STATE = if ($data -and $data.agent_state) { Clean-DisplayText $data.agent_state 16 } else { "idle" }
$usedPctVal = 0.0
if ($data -and $data.context_window -and $data.context_window.used_percentage -ne $null) {
    $usedPctVal = $data.context_window.used_percentage
}
$USED_PCT = Clamp-Double (Safe-Double $usedPctVal 0.0) 0 100

$VCS_BRANCH = if ($data -and $data.vcs -and $data.vcs.branch) { Clean-DisplayText $data.vcs.branch 80 } else { "" }
$VCS_DIRTY = if ($data -and $data.vcs -and $data.vcs.dirty -ne $null) { Safe-Bool $data.vcs.dirty } else { $false }
$SANDBOX = if ($data -and $data.sandbox -and $data.sandbox.enabled -ne $null) { Safe-Bool $data.sandbox.enabled } else { $false }
$SANDBOX_NET = if ($data -and $data.sandbox -and $data.sandbox.allow_network -ne $null) { Safe-Bool $data.sandbox.allow_network } else { $false }
$ARTIFACTS = if ($data -and $data.artifact_count -ne $null) { [Math]::Max(0, [int](Safe-Double $data.artifact_count 0)) } else { 0 }
$SUBAGENTS = if ($data -and $data.subagents -and $data.subagents.GetType().IsArray) { $data.subagents.Length } else { 0 }
$BG_TASKS = if ($data -and $data.task_count -ne $null) { [Math]::Max(0, [int](Safe-Double $data.task_count 0)) } else { 0 }
$MODEL_ID = if ($data -and $data.model -and $data.model.id) { Clean-DisplayText $data.model.id 80 } else { "" }
$MODEL_NAME = if ($data -and $data.model -and $data.model.display_name) { Clean-DisplayText $data.model.display_name 80 } else { "" }
$COLS = if ($data -and $data.terminal_width -ne $null) { [int](Clamp-Double (Safe-Double $data.terminal_width 80) 20 400) } else { 80 }
$CWD = if ($data -and $data.cwd) { Clean-DisplayText $data.cwd 260 } else { "" }
$CONV_ID = if ($data -and $data.conversation_id) { Clean-DisplayText $data.conversation_id 80 } else { "" }
$CLI_VERSION = if ($data -and $data.version) { Clean-DisplayText $data.version 32 } else { "" }
$PLAN_TIER = if ($data -and $data.plan_tier) { Clean-DisplayText $data.plan_tier 32 } else { "" }
$USER_EMAIL = if ($data -and $data.email) { Clean-DisplayText $data.email 120 } else { "" }

$TURN_INPUT_TOKENS = 0
if ($data -and $data.context_window -and $data.context_window.current_usage -and $data.context_window.current_usage.input_tokens -ne $null) {
    $TURN_INPUT_TOKENS = [Math]::Max(0, [int](Safe-Double $data.context_window.current_usage.input_tokens 0))
}
$TURN_OUTPUT_TOKENS = 0
if ($data -and $data.context_window -and $data.context_window.current_usage -and $data.context_window.current_usage.output_tokens -ne $null) {
    $TURN_OUTPUT_TOKENS = [Math]::Max(0, [int](Safe-Double $data.context_window.current_usage.output_tokens 0))
}

$INPUT_TOKENS = 0
if ($data -and $data.context_window -and $data.context_window.total_input_tokens -ne $null) {
    $INPUT_TOKENS = [Math]::Max(0, [int](Safe-Double $data.context_window.total_input_tokens 0))
}
$OUTPUT_TOKENS = 0
if ($data -and $data.context_window -and $data.context_window.total_output_tokens -ne $null) {
    $OUTPUT_TOKENS = [Math]::Max(0, [int](Safe-Double $data.context_window.total_output_tokens 0))
}
$CTX_LIMIT = 0
if ($data -and $data.context_window -and $data.context_window.context_window_size -ne $null) {
    $CTX_LIMIT = [Math]::Max(0, [int](Safe-Double $data.context_window.context_window_size 0))
}
$CTX_USED = $INPUT_TOKENS + $OUTPUT_TOKENS

# Quotas
$GEMINI_5H = Safe-Quota $data.quota.'gemini-5h'
$GEMINI_WK = Safe-Quota $data.quota.'gemini-weekly'
$TP_5H = Safe-Quota $data.quota.'3p-5h'
$TP_WK = Safe-Quota $data.quota.'3p-weekly'

$GEMINI_5H_RESET = if ($data -and $data.quota -and $data.quota.'gemini-5h' -and $data.quota.'gemini-5h'.reset_in_seconds -ne $null) { [Math]::Max(0, [int](Safe-Double $data.quota.'gemini-5h'.reset_in_seconds 0)) } else { -1 }
$GEMINI_WK_RESET = if ($data -and $data.quota -and $data.quota.'gemini-weekly' -and $data.quota.'gemini-weekly'.reset_in_seconds -ne $null) { [Math]::Max(0, [int](Safe-Double $data.quota.'gemini-weekly'.reset_in_seconds 0)) } else { -1 }
$TP_5H_RESET = if ($data -and $data.quota -and $data.quota.'3p-5h' -and $data.quota.'3p-5h'.reset_in_seconds -ne $null) { [Math]::Max(0, [int](Safe-Double $data.quota.'3p-5h'.reset_in_seconds 0)) } else { -1 }
$TP_WK_RESET = if ($data -and $data.quota -and $data.quota.'3p-weekly' -and $data.quota.'3p-weekly'.reset_in_seconds -ne $null) { [Math]::Max(0, [int](Safe-Double $data.quota.'3p-weekly'.reset_in_seconds 0)) } else { -1 }

# ANSI Helpers & Morandi Color Palette
$ESC = [char]27
$R = "$ESC[0m"
$B = "$ESC[1m"
$D = "$ESC[2m"
$I = "$ESC[3m"

# Morandi Palette (TrueColor 24-bit ANSI: Low-saturation, soft grey-undertone)
$FG_GRAY = "$ESC[38;2;125;135;145m"          # Muted slate grey (Borders, Separators, v-version)
$FG_DIM_GRAY = "$ESC[38;2;75;85;95m"         # Deep muted slate (Unfilled progress bars)
$FG_WHITE = "$ESC[38;2;215;215;215m"         # Soft neutral grey-white
$FG_BRIGHT_WHITE = "$ESC[38;2;240;240;240m"  # Creamy white for highlighted numbers

$FG_SAGE = "$ESC[38;2;145;175;155m"          # 鼠尾草綠 (Ready, AC, Good Quota)
$FG_OAT = "$ESC[38;2;215;185;145m"           # 燕麥暖黃 (Thinking, Warning, Mid Context)
$FG_FOG_BLUE = "$ESC[38;2;140;170;195m"      # 霧霾藍 (Working, Host, Artifacts)
$FG_DUSTY_ROSE = "$ESC[38;2;195;155;170m"    # 煙燻粉藕 (Tool, Tasks, 7D Quota)
$FG_TERRACOTTA = "$ESC[38;2;205;130;120m"    # 陶土磚紅 (Dirty branch, High Context, Low Quota)
$FG_MUTED_CYAN = "$ESC[38;2;135;175;180m"    # 灰海青 / 天青 (CWD, Clean Git, Subagents, 5H Quota)
$FG_LAVENDER = "$ESC[38;2;175;160;190m"      # 薰衣草灰紫 (Model display, badges)

# ANSI Aliases mapped to Morandi Palette for compatibility
$FG_BLACK = "$ESC[38;2;40;44;52m"
$FG_RED = $FG_TERRACOTTA
$FG_GREEN = $FG_SAGE
$FG_YELLOW = $FG_OAT
$FG_BLUE = $FG_FOG_BLUE
$FG_MAGENTA = $FG_DUSTY_ROSE
$FG_CYAN = $FG_MUTED_CYAN

$FG_BRIGHT_RED = $FG_TERRACOTTA
$FG_BRIGHT_GREEN = $FG_SAGE
$FG_BRIGHT_YELLOW = $FG_OAT
$FG_BRIGHT_BLUE = $FG_FOG_BLUE
$FG_BRIGHT_MAGENTA = $FG_LAVENDER
$FG_BRIGHT_CYAN = $FG_MUTED_CYAN

$NUM_COLOR = "${FG_BRIGHT_WHITE}${B}"

# VCS: Only query git directly if JSON payload didn't provide branch info
if (-not $VCS_BRANCH) {
    Ensure-GitModule
    if (Get-Command -Name Get-GitStatus -ErrorAction SilentlyContinue) {
        $GIT_DIR = if ($CWD) { $CWD.TrimEnd('\', '/') } else { "." }
        $gitInfo = Get-GitStatus -Path $GIT_DIR -TimeoutMs 200
        if ($gitInfo.Available) {
            $VCS_BRANCH = $gitInfo.Branch
            $VCS_DIRTY = $gitInfo.Dirty
        }
    }
}

# Format percentages
$PCT_FMT = $USED_PCT.ToString("0.0", [System.Globalization.CultureInfo]::InvariantCulture)
$PCT_INT = [int][Math]::Floor($USED_PCT)

# Formatting helpers
function human_format($num) {
    if ($num -eq $null -or $num -eq 0) { return "0" }
    if ($num -ge 1000000) {
        $val = [Math]::Round($num / 1000000, 1)
        return $val.ToString("0.0", [System.Globalization.CultureInfo]::InvariantCulture) + "M"
    }
    if ($num -ge 1000) {
        $val = [Math]::Round($num / 1000, 1)
        return $val.ToString("0.0", [System.Globalization.CultureInfo]::InvariantCulture) + "K"
    }
    return $num.ToString()
}

$INPUT_TOK_FMT = human_format $INPUT_TOKENS
$OUTPUT_TOK_FMT = human_format $OUTPUT_TOKENS
$CTX_LIMIT_FMT = human_format $CTX_LIMIT
$CTX_USED_FMT = human_format $CTX_USED
$TURN_INPUT_FMT = human_format $TURN_INPUT_TOKENS
$TURN_OUTPUT_FMT = human_format $TURN_OUTPUT_TOKENS

function shorten_path($path, $max_len = 25) {
    if (-not $path) { return "" }
    try {
        $path = [string]$path
        $homeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
        if ($homeDir -and $path.StartsWith($homeDir)) {
            $path = "~" + $path.Substring($homeDir.Length)
        }
        if ($path.Length -gt $max_len) {
            $leaf = ""
            try {
                $leaf = Split-Path $path -Leaf -ErrorAction Stop
            } catch {
                $parts = $path.Split([char[]]@('\', '/'))
                $leaf = $parts[-1]
            }
            if ($leaf.Length -gt ($max_len - 3)) {
                $slice_len = $max_len - 3
                if ($slice_len -lt 1) { $slice_len = 1 }
                $startIdx = [Math]::Max(0, $leaf.Length - $slice_len)
                return "..." + (Safe-Substring $leaf $startIdx $slice_len)
            }
            return "..." + $leaf
        }
        return $path
    } catch {
        return ""
    }
}

function Safe-Substring([string]$str, [int]$startIndex, [int]$length) {
    if (-not $str) { return "" }
    if ($startIndex -lt 0) { $startIndex = 0 }
    if ($startIndex -ge $str.Length) { return "" }
    if ($length -le 0) { return "" }
    $actualLength = [Math]::Min($length, $str.Length - $startIndex)
    return $str.Substring($startIndex, $actualLength)
}

function Truncate-String($str, $maxLen) {
    if (-not $str) { return "" }
    $str = [string]$str
    if ($maxLen -le 0) { return "" }
    if ($str.Length -le $maxLen) { return $str }
    if ($maxLen -le 3) { return Safe-Substring $str 0 $maxLen }
    return (Safe-Substring $str 0 ($maxLen - 3)) + "..."
}

# Dynamic limits based on terminal width ($COLS)
$max_path_len = 25
$max_user_len = 35
$max_host_len = 30
$max_model_len = 35
$max_branch_len = 30

if ($COLS -lt 65) {
    $max_path_len = 10
    $max_user_len = 12
    $max_host_len = 10
    $max_model_len = 14
    $max_branch_len = 14
} elseif ($COLS -lt 85) {
    $max_path_len = 16
    $max_user_len = 18
    $max_host_len = 14
    $max_model_len = 20
    $max_branch_len = 20
} elseif ($COLS -lt 110) {
    $max_path_len = 20
    $max_user_len = 24
    $max_host_len = 18
    $max_model_len = 28
    $max_branch_len = 25
}

$CWD_SHORT = shorten_path $CWD $max_path_len

# ─── Parse CLI Arguments & Theme ─────────────────────────────────────────────
$USE_CLASSIC_ICONS = $false
foreach ($arg in $args) {
    if ($arg -eq "--classic" -or $arg -eq "--no-nerdfont" -or $arg -eq "--compatibility") {
        $USE_CLASSIC_ICONS = $true
    }
}
if ($data -and ($data.nerdfont -eq $false -or $data.classic -eq $true -or $data.theme -eq "classic")) {
    $USE_CLASSIC_ICONS = $true
}

if ($USE_CLASSIC_ICONS) {
    $DOT_L1 = "${FG_GRAY} ╱ ${R}"
    $DOT_L2 = "${FG_GRAY} · ${R}"
    $ICON_READY = "●"
    $ICON_THINKING = "◆"
    $ICON_WORKING = "⚙"
    $ICON_TOOL = "🔧"
    $ICON_STATE_UNKNOWN = "⏳"
    $ICON_VCS = "╱"
    $ICON_MODEL = ""
    $ICON_SANDBOX_NET = "ON (net)"
    $ICON_SANDBOX_NONET = "ON (no-net)"
    $ICON_SANDBOX_OFF = "OFF"
    $ICON_CONTEXT_BAR = "ctx"
    $ICON_ARTIFACTS = "artifacts"
    $ICON_SUBAGENTS = "subagents"
    $ICON_TASKS = "tasks"
    $ICON_DIR = "╱"
    $ICON_CONV = "╱"
    $ICON_TOK_SUM = ""
    $ICON_RESET = "~"
    $ICON_AC = "AC"
    $ICON_BAT = "BAT"
} else {
    $DOT_L1 = "${FG_GRAY} | ${R}"
    $DOT_L2 = "${FG_GRAY} | ${R}"
    $ICON_READY = ""
    $ICON_THINKING = "󰟷"
    $ICON_WORKING = ""
    $ICON_TOOL = ""
    $ICON_STATE_UNKNOWN = ""
    $ICON_VCS = ""
    $ICON_MODEL = ""
    $ICON_SANDBOX_NET = "󰒙"
    $ICON_SANDBOX_NONET = "󰴴"
    $ICON_SANDBOX_OFF = "󰦜"
    $ICON_CONTEXT_BAR = "󱍏"
    $ICON_ARTIFACTS = ""
    $ICON_SUBAGENTS = "󱙺"
    $ICON_TASKS = ""
    $ICON_DIR = ""
    $ICON_CONV = "󰍪"
    $ICON_TOK_SUM = ""
    $ICON_RESET = "󰔟"
    $ICON_AC = "󰚥"
    $ICON_BAT = "🔋"
}

$script:ANSI_REGEX = [regex]::new('\x1b\[[0-9;]*m', [System.Text.RegularExpressions.RegexOptions]::Compiled)
$script:STRIP_SEP_REGEX = [regex]::new('^\s*(\x1b\[[0-9;]*m\s*)*[\|╱·]\s*(\x1b\[[0-9;]*m\s*)*', [System.Text.RegularExpressions.RegexOptions]::Compiled)
$script:HAS_SEP_REGEX = [regex]::new('^\s*(\x1b\[[0-9;]*m\s*)*[\|╱·]', [System.Text.RegularExpressions.RegexOptions]::Compiled)

function visible_len($str) {
    if (-not $str) { return 0 }
    # Strips ESC sequences and counts visible length
    $stripped = $script:ANSI_REGEX.Replace($str, '')
    
    $len = 0
    for ($i = 0; $i -lt $stripped.Length; $i++) {
        $val = 0
        if ([char]::IsHighSurrogate($stripped[$i]) -and ($i + 1 -lt $stripped.Length) -and [char]::IsLowSurrogate($stripped[$i + 1])) {
            try {
                $val = [char]::ConvertToUtf32($stripped, $i)
                $i++ # Skip low surrogate
            } catch {
                $val = [int]$stripped[$i]
            }
        } else {
            $val = [int]$stripped[$i]
        }
        
        # Variation selectors (like \ufe0f) are 0-width
        if ($val -ge 0xfe00 -and $val -le 0xfe0f) {
            continue
        }
        
        # Double-width characters in standard rendering:
        # 1. CJK characters and fullwidth symbols
        # 2. Hourglass emoji (0x231b)
        # 3. Standard Plane 1 Emoji Ranges (0x1f000 to 0x1faff, 0x1f300 to 0x1f9ff)
        # Note: Nerd Fonts in Plane 15/16 PUA (starting at 0xf0000) are treated as single-width (1 column).
        if (($val -ge 0x4e00 -and $val -le 0x9fff) -or 
            ($val -ge 0x3000 -and $val -le 0x303f) -or 
            ($val -ge 0xff00 -and $val -le 0xffef) -or
            ($val -eq 0x231b) -or
            ($val -ge 0x1f000 -and $val -le 0x1faff) -or
            ($val -ge 0x1f300 -and $val -le 0x1f9ff)) {
            $len += 2
        } else {
            $len += 1
        }
    }
    return $len
}

$CLI_VER_FMT = ""
if ($CLI_VERSION) {
    $CLI_VER_FMT = "${DOT_L1}${FG_GRAY}v${CLI_VERSION}${R}"
}

$USER_FMT = ""
if ($PLAN_TIER -or $USER_EMAIL) {
    $userInfo = ""
    if ($PLAN_TIER -and $USER_EMAIL) {
        $userInfo = "${PLAN_TIER} (${USER_EMAIL})"
    } elseif ($PLAN_TIER) {
        $userInfo = $PLAN_TIER
    } else {
        $userInfo = $USER_EMAIL
    }
    # Truncate if too long
    $userInfo = Truncate-String $userInfo $max_user_len
    if ($USE_CLASSIC_ICONS) {
        $USER_FMT = "${DOT_L1}${FG_GRAY}${userInfo}${R}"
    } else {
        $USER_FMT = "${DOT_L1}${FG_GRAY}󰇮 ${userInfo}${R}"
    }
}

# Get hostname
$HOST_NAME = ""
try { $HOST_NAME = [System.Net.Dns]::GetHostName() } catch {}
$HOST_NAME = if ($env:COMPUTERNAME) { $env:COMPUTERNAME } else { $HOST_NAME }

$HOST_FMT = ""
if ($HOST_NAME) {
    $hostDetails = Truncate-String $HOST_NAME $max_host_len
    if ($USE_CLASSIC_ICONS) {
        $HOST_FMT = "${DOT_L1}${FG_BRIGHT_BLUE}${hostDetails}${R}"
    } else {
        $HOST_FMT = "${DOT_L1}${FG_BRIGHT_BLUE}󰒋 ${hostDetails}${R}"
    }
}

# Get Power Status
$POWER_FMT = ""
if ($env:ANTIGRAVITY_STATUS_NO_POWER -ne '1' -and ($env:OS -like '*Windows*' -or $env:COMPUTERNAME)) {
    Ensure-PowerModule
    if (Get-Command -Name Get-PowerStatus -ErrorAction SilentlyContinue) {
        $powerInfo = Get-PowerStatus
        if ($powerInfo) {
            $chargePct = $powerInfo.Percent
            $lineStatus = $powerInfo.LineStatus
            if ($lineStatus -eq "Offline" -or ($chargePct -lt 100 -and $chargePct -gt 0 -and $lineStatus -ne "Online")) {
                if ($USE_CLASSIC_ICONS) {
                    $POWER_FMT = "${DOT_L2}${FG_BRIGHT_YELLOW}${ICON_BAT}:${chargePct}%${R}"
                } else {
                    $POWER_FMT = "${DOT_L2}${FG_BRIGHT_YELLOW}${ICON_BAT} ${chargePct}%${R}"
                }
            } else {
                if ($USE_CLASSIC_ICONS) {
                    $POWER_FMT = "${DOT_L2}${FG_GREEN}${ICON_AC}${R}"
                } else {
                    $POWER_FMT = "${DOT_L2}${FG_GREEN}${ICON_AC} AC${R}"
                }
            }
        }
    }
}

# State Indicator
$S = ""
switch ($STATE) {
    "idle"     { $S = "${FG_BRIGHT_GREEN}${B}${ICON_READY} READY${R}" }
    "thinking" { $S = "${FG_BRIGHT_YELLOW}${B}${ICON_THINKING} THINKING${R}" }
    "working"  { $S = "${FG_BRIGHT_CYAN}${B}${ICON_WORKING} WORKING${R}" }
    "tool_use" { $S = "${FG_BRIGHT_MAGENTA}${B}${ICON_TOOL} TOOL${R}" }
    default    { $S = "${FG_WHITE}${B}${ICON_STATE_UNKNOWN} $($STATE.ToUpper())${R}" }
}

# VCS branch details
$V = ""
if ($VCS_BRANCH) {
    $vcsDisp = Truncate-String $VCS_BRANCH $max_branch_len
    if ($VCS_DIRTY -eq $true) {
        if ($USE_CLASSIC_ICONS) {
            $V = "${DOT_L1}${FG_BRIGHT_RED}${vcsDisp}${FG_BRIGHT_YELLOW}*${R}"
        } else {
            $V = "${DOT_L1}${R}${FG_BRIGHT_RED}${ICON_VCS} ${vcsDisp}${FG_BRIGHT_YELLOW}*${R}"
        }
    } else {
        if ($USE_CLASSIC_ICONS) {
            $V = "${DOT_L1}${FG_BRIGHT_BLUE}${vcsDisp}${R}"
        } else {
            $V = "${DOT_L1}${R}${FG_BRIGHT_BLUE}${ICON_VCS} ${vcsDisp}${R}"
        }
    }
}

# Model details
$disp = if ($MODEL_NAME) { $MODEL_NAME } else { $MODEL_ID }
$disp = Truncate-String $disp $max_model_len
$M = ""
if ($disp) {
    if ($USE_CLASSIC_ICONS) {
        $M = "${DOT_L1}${FG_BRIGHT_MAGENTA}${I}${disp}${R}"
    } else {
        $M = "${DOT_L1}${FG_BRIGHT_MAGENTA}${I}${ICON_MODEL} ${disp}${R}"
    }
}

# Sandbox Badge
$SB = ""
if ($SANDBOX -eq $true) {
    if ($SANDBOX_NET -eq $true) {
        if ($USE_CLASSIC_ICONS) { $SB = "${FG_GREEN}sandbox net${R}" }
        else { $SB = "${FG_GREEN}${ICON_SANDBOX_NET} net${R}" }
    } else {
        if ($USE_CLASSIC_ICONS) { $SB = "${FG_GREEN}sandbox on${R}" }
        else { $SB = "${FG_GREEN}${ICON_SANDBOX_NONET} on${R}" }
    }
} else {
    if ($USE_CLASSIC_ICONS) {
        $SB = "${FG_GRAY}sandbox off${R}"
    } else {
        $SB = "${FG_RED}${ICON_SANDBOX_OFF} off${R}"
    }
}

# Context bar (Compact 10 blocks)
$BAR_LEN = 10
$FILLED = [int][Math]::Floor(($PCT_INT * $BAR_LEN) / 100)
$REMAINDER = ($PCT_INT * $BAR_LEN) % 100

$FILL_COLOR = $FG_MUTED_CYAN
if ($PCT_INT -ge 90) { $FILL_COLOR = $FG_TERRACOTTA }
elseif ($PCT_INT -ge 60) { $FILL_COLOR = $FG_OAT }

if ($USE_CLASSIC_ICONS) {
    $BAR = ""
    for ($i = 0; $i -lt $BAR_LEN; $i++) {
        if ($i -lt $FILLED) {
            $BAR += "█"
        } elseif ($i -eq $FILLED) {
            if ($REMAINDER -ge 75) { $BAR += "▓" }
            elseif ($REMAINDER -ge 50) { $BAR += "▒" }
            elseif ($REMAINDER -ge 25) { $BAR += "░" }
            else { $BAR += "·" }
        } else {
            $BAR += "·"
        }
    }
    $CTX_BAR = "${FG_GRAY}ctx ${FILL_COLOR}${BAR} ${NUM_COLOR}${PCT_FMT}%${R}"
} else {
    $BAR = ""
    for ($i = 0; $i -lt $BAR_LEN; $i++) {
        if ($i -lt $FILLED) {
            $BAR += "${FILL_COLOR}█${R}"
        } elseif ($i -eq $FILLED) {
            if ($REMAINDER -ge 75) { $BAR += "${FILL_COLOR}▓${R}${FG_DIM_GRAY}" }
            elseif ($REMAINDER -ge 50) { $BAR += "${FILL_COLOR}▒${R}${FG_DIM_GRAY}" }
            else { $BAR += "${FILL_COLOR}░${R}${FG_DIM_GRAY}" }
        } else {
            $BAR += "${FG_DIM_GRAY}░${R}"
        }
    }
    $CTX_BAR = "${FG_OAT}${ICON_CONTEXT_BAR} ${R}${BAR} ${NUM_COLOR}${PCT_FMT}%${R}"
}

# Stats badges
if ($USE_CLASSIC_ICONS) {
    if ($COLS -lt 90) {
        $ART_FMT = "${FG_GRAY}art:${NUM_COLOR}${ARTIFACTS}${R}"
        $SUB_FMT = "${FG_GRAY}sub:${NUM_COLOR}${SUBAGENTS}${R}"
        $BG_FMT = "${FG_GRAY}task:${NUM_COLOR}${BG_TASKS}${R}"
    } else {
        $ART_FMT = "${FG_GRAY}artifacts ${NUM_COLOR}${ARTIFACTS}${R}"
        $SUB_FMT = "${FG_GRAY}subagents ${NUM_COLOR}${SUBAGENTS}${R}"
        $BG_FMT = "${FG_GRAY}tasks ${NUM_COLOR}${BG_TASKS}${R}"
    }
} else {
    $ART_FMT = "${FG_BLUE}${ICON_ARTIFACTS} ${NUM_COLOR}${ARTIFACTS}${R}"
    $SUB_FMT = "${FG_CYAN}${ICON_SUBAGENTS} ${NUM_COLOR}${SUBAGENTS}${R}"
    $BG_FMT = "${FG_MAGENTA}${ICON_TASKS} ${NUM_COLOR}${BG_TASKS}${R}"
}

$CONV_FMT = ""
if ($CONV_ID) {
    $short_conv = Safe-Substring $CONV_ID 0 8
    if ($USE_CLASSIC_ICONS) {
        $CONV_FMT = "${DOT_L1}${FG_GRAY}${short_conv}${R}"
    } else {
        $CONV_FMT = "${DOT_L1}${FG_GRAY}${ICON_CONV} ${short_conv}${R}"
    }
}

$TOK_DETAILS_WIDE = ""
if ($CTX_USED -gt 0 -or $CTX_LIMIT -gt 0) {
    if ($COLS -ge 95) {
        $turnStr = ""
        if ($TURN_INPUT_TOKENS -gt 0 -or $TURN_OUTPUT_TOKENS -gt 0) {
            $turnStr = " ${FG_GRAY}| turn:${R} ${FG_SAGE}+${TURN_INPUT_FMT}${R}/${FG_DUSTY_ROSE}${TURN_OUTPUT_FMT}${R}"
        }
        if ($USE_CLASSIC_ICONS) {
            $TOK_DETAILS_WIDE = "(${CTX_USED_FMT}/${CTX_LIMIT_FMT}) ${DOT_L2}total: ${NUM_COLOR}${INPUT_TOK_FMT}${R}/${NUM_COLOR}${OUTPUT_TOK_FMT}${R}${turnStr}"
        } else {
            $TOK_DETAILS_WIDE = "(${CTX_USED_FMT}/${CTX_LIMIT_FMT}) ${DOT_L2}${FG_YELLOW}${ICON_TOK_SUM}${R} total: ${NUM_COLOR}${INPUT_TOK_FMT}${R}/${NUM_COLOR}${OUTPUT_TOK_FMT}${R}${turnStr}"
        }
    } elseif ($COLS -ge 75) {
        if ($USE_CLASSIC_ICONS) {
            $TOK_DETAILS_WIDE = "(${CTX_USED_FMT}/${CTX_LIMIT_FMT})"
        } else {
            $TOK_DETAILS_WIDE = "${FG_YELLOW}${ICON_TOK_SUM}${R} (${CTX_USED_FMT}/${CTX_LIMIT_FMT})"
        }
    } else {
        $TOK_DETAILS_WIDE = ""
    }
}

# Quota bars
function format_reset_time($sec, [bool]$compact = $false) {
    if ($sec -eq $null -or $sec -le 0) { return "" }
    $days = [int][Math]::Floor($sec / 86400)
    $rem = $sec % 86400
    $hours = [int][Math]::Floor($rem / 3600)
    $rem = $rem % 3600
    $mins = [int][Math]::Floor($rem / 60)

    if ($compact) {
        if ($days -gt 0) {
            if ($hours -gt 0) { return "${days}d${hours}h" }
            return "${days}d"
        }
        if ($hours -gt 0) {
            if ($mins -gt 0) { return "${hours}h${mins}m" }
            return "${hours}h"
        }
        if ($mins -gt 0) { return "${mins}m" }
        return "<1m"
    }

    if ($days -gt 0) {
        if ($hours -gt 0) { return "${days}d ${hours}h" }
        return "${days}d"
    }
    if ($hours -gt 0) {
        if ($mins -gt 0) { return "${hours}h ${mins}m" }
        return "${hours}h"
    }
    if ($mins -gt 0) { return "${mins}m" }
    return "<1m"
}

function make_quota_bar($val, $label, $bar_color, $reset_sec, [int]$target_bar_len = 8, [bool]$compact_time = $false) {
    $reset_label = if ($USE_CLASSIC_ICONS) { " " } else { " ${ICON_RESET} " }
    $separator = if ($USE_CLASSIC_ICONS) { "${FG_GRAY} · ${R}" } else { "${FG_GRAY} | ${R}" }

    if ($val -eq $null -or $val -lt 0) {
        $bar = ""
        for ($i = 0; $i -lt $target_bar_len; $i++) {
            if ($USE_CLASSIC_ICONS) { $bar += "·" } else { $bar += "░" }
        }
        return "${separator}${FG_BRIGHT_WHITE}${B}${label}${R} ${FG_GRAY}${bar} N/A${R}"
    }

    $val_int = [int][Math]::Floor($val)
    $text_color = $FG_SAGE
    if ($val_int -lt 20) { $text_color = $FG_TERRACOTTA }
    elseif ($val_int -lt 50) { $text_color = $FG_OAT }

    $filled = [int][Math]::Floor(($val_int * $target_bar_len) / 100)
    $remainder = ($val_int * $target_bar_len) % 100

    $bar = ""
    for ($i = 0; $i -lt $target_bar_len; $i++) {
        if ($i -lt $filled) {
            if ($USE_CLASSIC_ICONS) {
                $bar += "█"
            } else {
                $bar += "${bar_color}█${R}"
            }
        } elseif ($i -eq $filled) {
            if ($USE_CLASSIC_ICONS) {
                if ($remainder -ge 75) { $bar += "▓" }
                elseif ($remainder -ge 50) { $bar += "▒" }
                elseif ($remainder -ge 25) { $bar += "░" }
                else { $bar += "·" }
            } else {
                if ($remainder -ge 75) { $bar += "${bar_color}▓${R}${FG_DIM_GRAY}" }
                elseif ($remainder -ge 50) { $bar += "${bar_color}▒${R}${FG_DIM_GRAY}" }
                elseif ($remainder -ge 25) { $bar += "${bar_color}░${R}${FG_DIM_GRAY}" }
                else { $bar += "${FG_DIM_GRAY}░${R}" }
            }
        } else {
            if ($USE_CLASSIC_ICONS) {
                $bar += "·"
            } else {
                $bar += "${FG_DIM_GRAY}░${R}"
            }
        }
    }

    $val_fmt = if ($compact_time) {
        $val_int.ToString()
    } else {
        $val.ToString("0.0", [System.Globalization.CultureInfo]::InvariantCulture)
    }

    $reset_str = ""
    $t = format_reset_time $reset_sec $compact_time
    if ($t) {
        $reset_icon = if ($USE_CLASSIC_ICONS) { "~" } elseif ($compact_time) { " ${ICON_RESET}" } else { " ${ICON_RESET} " }
        $reset_str = "${reset_icon}${t}"
    }

    if ($USE_CLASSIC_ICONS) {
        return "${separator}${FG_BRIGHT_WHITE}${B}${label}${R} ${bar_color}${bar}${R} ${text_color}${val_fmt}%${R}${reset_str}"
    } else {
        return "${separator}${FG_BRIGHT_WHITE}${B}${label}${R} ${bar} ${text_color}${val_fmt}%${R}${reset_str}"
    }
}

# Determine active quota based on active model and quota availability
$isGeminiModel = ($MODEL_ID -like "*gemini*" -or $MODEL_NAME -like "*gemini*")

$hasGeminiQuota = (($GEMINI_5H -ne $null -and $GEMINI_5H -ne -1) -or ($GEMINI_WK -ne $null -and $GEMINI_WK -ne -1))
$has3pQuota = (($TP_5H -ne $null -and $TP_5H -ne -1) -or ($TP_WK -ne $null -and $TP_WK -ne -1))

if ($isGeminiModel) {
    if ($hasGeminiQuota) {
        $Q_5H = $GEMINI_5H
        $Q_WK = $GEMINI_WK
        $Q_5H_R = $GEMINI_5H_RESET
        $Q_WK_R = $GEMINI_WK_RESET
    } elseif ($has3pQuota) {
        $Q_5H = $TP_5H
        $Q_WK = $TP_WK
        $Q_5H_R = $TP_5H_RESET
        $Q_WK_R = $TP_WK_RESET
    } else {
        $Q_5H = -1
        $Q_WK = -1
        $Q_5H_R = -1
        $Q_WK_R = -1
    }
} else {
    if ($has3pQuota) {
        $Q_5H = $TP_5H
        $Q_WK = $TP_WK
        $Q_5H_R = $TP_5H_RESET
        $Q_WK_R = $TP_WK_RESET
    } elseif ($hasGeminiQuota) {
        $Q_5H = $GEMINI_5H
        $Q_WK = $GEMINI_WK
        $Q_5H_R = $GEMINI_5H_RESET
        $Q_WK_R = $GEMINI_WK_RESET
    } else {
        $Q_5H = -1
        $Q_WK = -1
        $Q_5H_R = -1
        $Q_WK_R = -1
    }
}

$isCompactBars = ($COLS -lt 95)
$qBarLen = if ($COLS -lt 95) { 5 } else { 8 }

$Q_5H_FMT = if (($Q_5H -ne $null -and $Q_5H -ne -1)) { make_quota_bar $Q_5H "5H" $FG_BRIGHT_CYAN $Q_5H_R $qBarLen $isCompactBars } else { "" }
$Q_WK_FMT = if (($Q_WK -ne $null -and $Q_WK -ne -1)) {
    if ($COLS -lt 75 -and $Q_5H_FMT) { "" }
    else { make_quota_bar $Q_WK "7D" $FG_BRIGHT_MAGENTA $Q_WK_R $qBarLen $isCompactBars }
} else { "" }



# Output Assembly based on Column Width (Always 4 lines to prevent terminal jumping)
$width = [Math]::Max(20, [int]$COLS)

# Box drawing characters
$BOX_TOP_L = "╭─"
$BOX_TOP_R = "─╮"
$BOX_BOT_L = "╰─"
$BOX_BOT_R = "─╯"
$BOX_MID_L = "│ "
$BOX_MID_R = " │"

$title = if ($width -lt 40) { " Status " } elseif ($width -lt 70) { " Antigravity " } else { " Antigravity Dashboard " }
$top_border = "${FG_GRAY}${BOX_TOP_L}${R}${title}${FG_GRAY}$("─" * [Math]::Max(0, $width - 4 - (visible_len $title)))${BOX_TOP_R}${R}"
$bottom_border = "${FG_GRAY}${BOX_BOT_L}$("─" * [Math]::Max(0, $width - 4))${BOX_BOT_R}${R}"

function Strip-Separator($str) {
    if (-not $str) { return "" }
    return $script:STRIP_SEP_REGEX.Replace($str, '')
}

function Has-Separator($str) {
    if (-not $str) { return $false }
    return $script:HAS_SEP_REGEX.IsMatch($str)
}

function Format-FlexWrapLine($left_items, $right_items, $total_width) {
    $max_content = $total_width - 4
    if ($max_content -lt 1) { $max_content = 1 }
    
    $left_str = ""
    $is_first_left = $true
    foreach ($item in $left_items) {
        if ($item -and (visible_len $item) -gt 0) {
            if ($is_first_left) {
                $left_str += Strip-Separator $item
                $is_first_left = $false
            } else {
                $left_str += $item
            }
        }
    }
    $right_str = ""
    foreach ($item in $right_items) {
        if ($item -and (visible_len $item) -gt 0) { $right_str += $item }
    }
    
    $left_vis = visible_len $left_str
    $right_vis = visible_len $right_str
    if ($left_vis + $right_vis -le $max_content) {
        $pad = $max_content - $left_vis - $right_vis
        if ($pad -lt 0) { $pad = 0 }
        $spaces = " " * $pad
        return @("${FG_GRAY}│${R} ${left_str}${spaces}${right_str} ${FG_GRAY}│${R}")
    }
    
    $all_items = @()
    foreach ($item in $left_items) {
        if ($item -and (visible_len $item) -gt 0) { $all_items += $item }
    }
    $is_first_right = $true
    foreach ($item in $right_items) {
        if ($item -and (visible_len $item) -gt 0) {
            $processed = $item
            if ($is_first_right) {
                if (-not (Has-Separator $item)) {
                    $processed = "${DOT_L2}${item}"
                }
                $is_first_right = $false
            }
            $all_items += $processed
        }
    }
    
    $lines = @()
    $current_line_items = @()
    $current_line_vis = 0
    
    for ($i = 0; $i -lt $all_items.Length; $i++) {
        $item = $all_items[$i]
        $is_first_in_line = ($current_line_items.Length -eq 0)
        
        $processed_item = $item
        if ($is_first_in_line) {
            $processed_item = Strip-Separator $item
        }
        
        $item_vis = visible_len $processed_item
        if ($item_vis -gt $max_content) {
            $processed_item = $processed_item -replace '\x1b\[[0-9;]*m', ''
            if ($max_content -gt 3) {
                $processed_item = (Safe-Substring $processed_item 0 ($max_content - 3)) + "..."
            } else {
                $processed_item = Safe-Substring $processed_item 0 $max_content
            }
            $item_vis = visible_len $processed_item
        }
        
        if ($current_line_vis + $item_vis -le $max_content) {
            $current_line_items += $processed_item
            $current_line_vis += $item_vis
        } else {
            if ($current_line_items.Length -gt 0) {
                $pad = [Math]::Max(0, ($max_content - $current_line_vis))
                $spaces = " " * $pad
                $line_content = $current_line_items -join ""
                $lines += "${FG_GRAY}│${R} ${line_content}${spaces} ${FG_GRAY}│${R}"
            }
            $stripped_item = Strip-Separator $item
            $stripped_vis = visible_len $stripped_item
            if ($stripped_vis -gt $max_content) {
                $stripped_item = $stripped_item -replace '\x1b\[[0-9;]*m', ''
                if ($max_content -gt 3) {
                    $stripped_item = (Safe-Substring $stripped_item 0 ($max_content - 3)) + "..."
                } else {
                    $stripped_item = Safe-Substring $stripped_item 0 $max_content
                }
                $stripped_vis = visible_len $stripped_item
            }
            $current_line_items = @($stripped_item)
            $current_line_vis = $stripped_vis
        }
    }
    
    if ($current_line_items.Length -gt 0) {
        $pad = [Math]::Max(0, ($max_content - $current_line_vis))
        $spaces = " " * $pad
        $line_content = $current_line_items -join ""
        $lines += "${FG_GRAY}│${R} ${line_content}${spaces} ${FG_GRAY}│${R}"
    }
    
    return $lines
}

function Format-BoxLine($left, $right, $total_width) {
    $lines = Format-FlexWrapLine $left $right $total_width
    return $lines -join "`n"
}

$DIR_FMT = if ($CWD_SHORT -and $COLS -ge 65) {
    if ($USE_CLASSIC_ICONS) { "${FG_MUTED_CYAN}${CWD_SHORT}${R}" }
    else { "${FG_MUTED_CYAN}${ICON_DIR} ${CWD_SHORT}${R}" }
} else { "" }

# 3-Row Clean Dashboard Layout
# Row 1: Core Agent Identity, Model, Git branch (Left) | Working Directory (Right)
$L1_LEFT = @($S, $M, $V)
$L1_RIGHT = @($DIR_FMT)

# Row 2: Context Window usage bar (Left) | Token Usage Metrics (Right)
$L2_LEFT = @($CTX_BAR)
$L2_RIGHT = @($TOK_DETAILS_WIDE)

# Row 3: Quotas & Power (Left) | Sandbox, Artifacts, Subagents, Tasks (Right)
$L3_LEFT = @(
    $(if ($Q_5H_FMT) { $Q_5H_FMT } else { "" }),
    $(if ($Q_WK_FMT) { $Q_WK_FMT } else { "" }),
    $(if ($POWER_FMT) { $POWER_FMT } else { "" })
)
$itemSep = if ($COLS -lt 90) { " " } else { $DOT_L2 }
$L3_RIGHT = @(
    $SB,
    $(if ($ARTIFACTS -gt 0) { "${itemSep}${ART_FMT}" } else { "" }),
    $(if ($SUBAGENTS -gt 0) { "${itemSep}${SUB_FMT}" } else { "" }),
    $(if ($BG_TASKS -gt 0) { "${itemSep}${BG_FMT}" } else { "" })
)

$out1 = $top_border
$out2 = Format-BoxLine $L1_LEFT $L1_RIGHT $width
$out3 = Format-BoxLine $L2_LEFT $L2_RIGHT $width
$out4 = Format-BoxLine $L3_LEFT $L3_RIGHT $width
$out5 = $bottom_border
Write-Output "${out1}`n${out2}`n${out3}`n${out4}`n${out5}"
$global:LASTEXITCODE = 0
exit 0
} catch {
    $global:LASTEXITCODE = 0
    exit 0
}
