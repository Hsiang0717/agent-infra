# ANSI Helpers (Standard 16-color palette only)
$ESC = [char]27
$R = "$ESC[0m"         # Reset
$B = "$ESC[1m"         # Bold
$D = "$ESC[2m"         # Dim
$I = "$ESC[3m"         # Italic

# Foreground accents (Standard 16 colors)
$FG_BLACK = "$ESC[30m"
$FG_RED = "$ESC[31m"
$FG_GREEN = "$ESC[32m"
$FG_YELLOW = "$ESC[33m"
$FG_BLUE = "$ESC[34m"
$FG_MAGENTA = "$ESC[35m"
$FG_CYAN = "$ESC[36m"
$FG_WHITE = "$ESC[37m"

$FG_GRAY = "$ESC[90m"
$FG_BRIGHT_RED = "$ESC[91m"
$FG_BRIGHT_GREEN = "$ESC[92m"
$FG_BRIGHT_YELLOW = "$ESC[93m"
$FG_BRIGHT_BLUE = "$ESC[94m"
$FG_BRIGHT_MAGENTA = "$ESC[95m"
$FG_BRIGHT_CYAN = "$ESC[96m"
$FG_BRIGHT_WHITE = "$ESC[97m"

# Number Highlight Color
$NUM_COLOR = "${FG_BRIGHT_WHITE}${B}"

# Parse JSON from stdin
$stdin = $input | Out-String

$state = "idle"
$used_pct = 0.0
$vcs_branch = ""
$vcs_dirty = $false
$sandbox = $false
$artifacts = 0
$subagents = 0
$bg_tasks = 0
$model = ""
$cols = 80

if ($stdin) {
    try {
        $data = ConvertFrom-Json $stdin
        if ($data) {
            if ($data.agent_state) { $state = $data.agent_state }
            if ($data.context_window -and $data.context_window.used_percentage -ne $null) { $used_pct = [double]$data.context_window.used_percentage }
            if ($data.vcs) {
                if ($data.vcs.branch) { $vcs_branch = $data.vcs.branch }
                if ($data.vcs.dirty -eq $true -or $data.vcs.dirty -eq "true") { $vcs_dirty = $true }
            }
            if ($data.sandbox -and ($data.sandbox.enabled -eq $true -or $data.sandbox.enabled -eq "true")) { $sandbox = $true }
            if ($data.artifact_count -ne $null) { $artifacts = [int]$data.artifact_count }
            if ($data.subagents) {
                if ($data.subagents -is [array]) {
                    $subagents = $data.subagents.Count
                } elseif ($data.subagents -is [int]) {
                    $subagents = $data.subagents
                }
            }
            if ($data.task_count -ne $null) { $bg_tasks = [int]$data.task_count }
            if ($data.model -and $data.model.display_name) { $model = $data.model.display_name }
            if ($data.terminal_width -ne $null) { $cols = [int]$data.terminal_width }
        }
    } catch {
        # ignore parse errors, use defaults
    }
}

# Computed Values
$pct_fmt = $used_pct.ToString("F1", [System.Globalization.CultureInfo]::InvariantCulture)
$pct_int = [int][math]::Floor($used_pct)

# State Indicator
switch ($state) {
    "idle"     { $s = "${FG_BRIGHT_GREEN}${B}● READY${R}" }
    "thinking" { $s = "${FG_BRIGHT_YELLOW}${B}◆ THINKING${R}" }
    "working"  { $s = "${FG_BRIGHT_CYAN}${B}⚙ WORKING${R}" }
    "tool_use" { $s = "${FG_BRIGHT_MAGENTA}${B}🔧 TOOL${R}" }
    Default    { $s = "${FG_WHITE}${B}⏳ $($state.ToUpper())${R}" }
}

# VCS Branch
$v = ""
if ($vcs_branch) {
    if ($vcs_dirty) {
        $v = "${FG_GRAY} ╱ ${FG_BRIGHT_RED}${vcs_branch}${FG_BRIGHT_YELLOW}*${R}"
    } else {
        $v = "${FG_GRAY} ╱ ${FG_BRIGHT_BLUE}${vcs_branch}${R}"
    }
}

# Model
$m = ""
if ($model) {
    $m = "${FG_GRAY} ╱ ${FG_BRIGHT_MAGENTA}${I}${model}${R}"
}

# Sandbox Badge
if ($sandbox) {
    $sb = "${FG_GRAY}sandbox ${FG_BRIGHT_GREEN}${B}ON${R}"
} else {
    $sb = "${FG_GRAY}sandbox off${R}"
}

# Context Bar (15 segments)
$bar_len = 15
$filled = [int][math]::Floor(($pct_int * $bar_len) / 100)
$remainder = ($pct_int * $bar_len) % 100

if ($pct_int -ge 90) {
    $bar_color = $FG_BRIGHT_RED
} elseif ($pct_int -ge 60) {
    $bar_color = $FG_BRIGHT_YELLOW
} else {
    $bar_color = $FG_BRIGHT_WHITE
}

$bar = ""
for ($i = 0; $i -lt $bar_len; $i++) {
    if ($i -lt $filled) {
        $bar += "█"
    } elseif ($i -eq $filled) {
        if ($remainder -ge 75) { $bar += "▓" }
        elseif ($remainder -ge 50) { $bar += "▒" }
        elseif ($remainder -ge 25) { $bar += "░" }
        else { $bar += "·" }
    } else {
        $bar += "·"
    }
}

# Stats
$ctx = "${FG_GRAY}ctx ${bar_color}${bar} ${NUM_COLOR}${pct_fmt}%${R}"
$art_fmt = "${FG_GRAY}artifacts ${NUM_COLOR}${artifacts}${R}"
$sub_fmt = "${FG_GRAY}subagents ${NUM_COLOR}${subagents}${R}"
$bg_fmt = "${FG_GRAY}tasks ${NUM_COLOR}${bg_tasks}${R}"

# Separators
$dot = "${FG_GRAY} · ${R}"

# Output
$line1 = "${s}${m}${v}"
$line2 = " ${ctx}${dot}${art_fmt}${dot}${sub_fmt}${dot}${bg_fmt}${dot}${sb}"

if ($cols -ge 120) {
    Write-Output "${line1}${FG_GRAY}  │  ${R}${line2}"
} elseif ($cols -ge 80) {
    Write-Output "${FG_GRAY}╭─${R} ${line1}"
    Write-Output "${FG_GRAY}╰─${R}${line2}"
} else {
    Write-Output "${s}${m}"
    Write-Output "${ctx}${dot}${bg_fmt}"
}
