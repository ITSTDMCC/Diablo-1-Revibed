# Plays a password-protected TCP game between the original DevilutionX 1.5.3 and the port, both
# on this machine over 127.0.0.1, and saves screenshots of both sides (game data: do not commit).
#
#   powershell -File tools\interop_original.ps1 -Original <devilutionx.exe> -OrigData <dir with
#     DIABDAT.MPQ and the original devilutionx.mpq> -PortData <dir with DIABDAT.MPQ> -Out <dir>
#     [-PortHosts]
#
# Default: the original hosts and the port joins. -PortHosts swaps the roles. The original is
# driven by key messages posted to its own window (tools\winkeys.ps1), never global keystrokes,
# and runs windowed and muted from its own settings folder under -Out; its install folder is not
# written to. Build the port first (cargo build --release).
param([string]$Original, [string]$OrigData, [string]$PortData, [string]$Out, [switch]$PortHosts)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$port = Join-Path $root 'target\release\diablo1_rs.exe'
$keys = Join-Path $PSScriptRoot 'winkeys.ps1'
$scripts = Join-Path $PSScriptRoot 'input_scripts'
New-Item -ItemType Directory -Force $Out | Out-Null

function Home([string]$name, [string]$extra) {
    $h = Join-Path $Out $name
    if (Test-Path $h) { Remove-Item -Recurse -Force $h }
    New-Item -ItemType Directory -Force $h | Out-Null
    Set-Content -Encoding ascii (Join-Path $h 'diablo.ini') ("[Network]`r`nBind Address=127.0.0.1`r`n" + $extra)
    return $h
}

function StartPort([string]$script, [string]$frames, [int]$maxFrames) {
    $h = Home 'port' ''
    $env:DIABLO_HEADLESS = '1'; $env:DIABLO_FIXED_STEP = 'pace'; $env:DIABLO_NO_AUDIO = '1'
    $env:DIABLO_TIME = '1700000000'; $env:DIABLO_MAX_FRAMES = "$maxFrames"
    $env:DIABLO_INPUT_SCRIPT = (Join-Path $scripts $script)
    $env:DIABLO_SCREENSHOT_FRAMES = $frames; $env:DIABLO_SCREENSHOT_DIR = (Join-Path $Out 'port_shots')
    return Start-Process -PassThru -NoNewWindow -FilePath $port -ArgumentList @('-n', '--data-dir', "`"$PortData`"", '--save-dir', "`"$h`"", '--config-dir', "`"$h`"") `
        -RedirectStandardError (Join-Path $Out 'port.log')
}

function StartOriginal() {
    $h = Home 'original' "[Graphics]`r`nFullscreen=0`r`n[Audio]`r`nSound Volume=-1600`r`nMusic Volume=-1600`r`n"
    $p = Start-Process -PassThru -FilePath $Original -ArgumentList @('--diablo', '-n', '--data-dir', "`"$OrigData`"", '--save-dir', "`"$h`"", '--config-dir', "`"$h`"")
    Start-Sleep -Seconds 8
    return $p
}

$shots = Join-Path $Out 'original_shots'
New-Item -ItemType Directory -Force $shots | Out-Null
# main menu: Multi Player; connection: Client-Server (TCP) is second; new hero: Warrior "Orig"
$toHeroName = 'xvk:28;wait:300;vk:0D;wait:2500;xvk:28;wait:300;vk:0D;wait:2500;vk:0D;wait:2000;text:Orig;wait:300;vk:0D;wait:2500'

if ($PortHosts) {
    $p = StartPort 'tcp_host_password.txt' '4000,5000,6000,7000' 9000
    Start-Sleep -Seconds 35
    $o = StartOriginal
    # Join Game, address, password
    & $keys -ProcessId $o.Id -ShotDir $shots -Steps "$toHeroName;xvk:28;wait:200;xvk:28;wait:200;vk:0D;wait:2000;text:127.0.0.1;vk:0D;wait:2000;text:secret;vk:0D;wait:12000;shot:joined.png;wait:10000;shot:later.png"
    $p.WaitForExit()
    Stop-Process -Id $o.Id
} else {
    $o = StartOriginal
    # Create Game, difficulty, speed, password
    & $keys -ProcessId $o.Id -ShotDir $shots -Steps "$toHeroName;vk:0D;wait:1500;vk:0D;wait:1500;vk:0D;wait:1500;text:secret;vk:0D;wait:6000;shot:hosting.png"
    $p = StartPort 'tcp_guest_password.txt' '2100,2500,2990' 3600
    & $keys -ProcessId $o.Id -ShotDir $shots -Steps "wait:30000;shot:join.png;wait:8000;shot:walk1.png;wait:8000;shot:walk2.png;wait:10000;shot:leave.png"
    $p.WaitForExit()
    Stop-Process -Id $o.Id
}
Write-Output "screenshots in $Out"
