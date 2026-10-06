# One-time setup for the diablo1_rs release zip (run through "Setup.bat").
#
# 1. Finds your Diablo folder (the one with DIABDAT.MPQ from your own copy of the game) and
#    remembers it in data-dir.txt next to this script.
# 2. Gets devilutionx.mpq (DevilutionX's own fonts and interface pieces, not Blizzard data) from
#    the official DevilutionX 1.5.3 release if you do not have it yet, and checks it is the right
#    file.
#
# Nothing from the game is downloaded or included: you need your own copy of Diablo (GOG sells
# it as "Diablo + Hellfire").

param(
    # Use this folder instead of searching (for scripts and tests).
    [string]$DataDir,
    # A local copy of devilutionx-windows-x86_64.zip to take devilutionx.mpq from (for tests).
    [string]$DevilutionXZip,
    # No message boxes or prompts.
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$zipUrl = 'https://github.com/diasurgical/DevilutionX/releases/download/1.5.3/devilutionx-windows-x86_64.zip'
$mpqSha256 = 'BED79D378D08B30FD1D3B4A48F609CAD887D3A00F6546AE24BA1A71FE4332052'

Add-Type -AssemblyName System.Windows.Forms

function Say([string]$text, [string]$icon = 'Information') {
    Write-Host $text
    if (-not $Quiet) {
        [void][System.Windows.Forms.MessageBox]::Show($text, 'Diablo (diablo1_rs) setup', 'OK', $icon)
    }
}

function Has-GameData([string]$dir) {
    return $dir -and (Test-Path -LiteralPath (Join-Path $dir 'DIABDAT.MPQ'))
}

# --- 1. The Diablo folder -------------------------------------------------------------------

$found = $null
if ($DataDir) {
    if (Has-GameData $DataDir) { $found = $DataDir }
} else {
    $candidates = New-Object System.Collections.Generic.List[string]
    $candidates.Add($here)
    # GOG installs (GOG Galaxy and the offline installers) register their folder
    foreach ($key in 'HKLM:\SOFTWARE\WOW6432Node\GOG.com\Games', 'HKLM:\SOFTWARE\GOG.com\Games') {
        if (Test-Path $key) {
            Get-ChildItem $key -ErrorAction SilentlyContinue | ForEach-Object {
                $p = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
                if ($p -and $p.gameName -like 'Diablo*' -and $p.path) { $candidates.Add($p.path) }
            }
        }
    }
    foreach ($root in ${env:ProgramFiles(x86)}, $env:ProgramFiles) {
        if ($root) {
            $candidates.Add((Join-Path $root 'GOG Galaxy\Games\Diablo'))
            $candidates.Add((Join-Path $root 'GOG Games\Diablo'))
        }
    }
    foreach ($drive in Get-PSDrive -PSProvider FileSystem -ErrorAction SilentlyContinue) {
        $candidates.Add((Join-Path $drive.Root 'GOG Games\Diablo'))
        $candidates.Add((Join-Path $drive.Root 'GOG Galaxy\Games\Diablo'))
        $candidates.Add((Join-Path $drive.Root 'Games\Diablo'))
    }
    foreach ($c in $candidates) {
        if (Has-GameData $c) { $found = $c; break }
    }
    if (-not $found -and -not $Quiet) {
        [void][System.Windows.Forms.MessageBox]::Show(
            "Could not find your Diablo folder automatically.`n`nIn the next window, pick the folder of your Diablo installation: the one that contains DIABDAT.MPQ.",
            'Diablo (diablo1_rs) setup', 'OK', 'Information')
        $dialog = New-Object System.Windows.Forms.FolderBrowserDialog
        $dialog.Description = 'Pick your Diablo folder (the one with DIABDAT.MPQ)'
        if ($dialog.ShowDialog() -eq 'OK' -and (Has-GameData $dialog.SelectedPath)) {
            $found = $dialog.SelectedPath
        }
    }
}

if (-not $found) {
    Say ("DIABDAT.MPQ was not found.`n`nThis game needs your own copy of Diablo (GOG sells it as 'Diablo + Hellfire'). " +
        "Install it, then run Setup.bat again, or copy DIABDAT.MPQ into this folder:`n$here") 'Warning'
    exit 1
}
$found = (Resolve-Path -LiteralPath $found).Path
Write-Host "Diablo folder: $found"

# --- 2. devilutionx.mpq ---------------------------------------------------------------------

function Is-Good([string]$file) {
    return (Test-Path -LiteralPath $file) -and ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -eq $mpqSha256)
}

$mpqHere = Join-Path $here 'devilutionx.mpq'
if (-not (Is-Good $mpqHere)) {
    $mpqThere = Join-Path $found 'devilutionx.mpq'
    if (Is-Good $mpqThere) {
        Copy-Item -LiteralPath $mpqThere -Destination $mpqHere -Force
    } else {
        $zip = $DevilutionXZip
        $tmp = $null
        if (-not $zip) {
            Write-Host "Downloading devilutionx.mpq from the DevilutionX 1.5.3 release..."
            $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("devilutionx-1.5.3-" + [guid]::NewGuid() + '.zip')
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
            try {
                (New-Object System.Net.WebClient).DownloadFile($zipUrl, $tmp)
            } catch {
                Say ("Could not download devilutionx.mpq ($($_.Exception.Message)).`n`n" +
                    "Download devilutionx-windows-x86_64.zip yourself from`n$zipUrl`n" +
                    "and copy devilutionx.mpq from it into this folder:`n$here") 'Warning'
                exit 1
            }
            $zip = $tmp
        }
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $archive = [System.IO.Compression.ZipFile]::OpenRead($zip)
        try {
            $entry = $archive.Entries | Where-Object { $_.Name -ieq 'devilutionx.mpq' } | Select-Object -First 1
            if (-not $entry) { throw 'devilutionx.mpq is not in the zip' }
            [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $mpqHere, $true)
        } finally {
            $archive.Dispose()
            if ($tmp) { Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue }
        }
        if (-not (Is-Good $mpqHere)) {
            Remove-Item -LiteralPath $mpqHere -Force -ErrorAction SilentlyContinue
            Say 'The downloaded devilutionx.mpq is not the expected DevilutionX 1.5.3 file. Setup stopped.' 'Error'
            exit 1
        }
    }
}

# remembered only once everything is in place (the play scripts run the setup until then)
[System.IO.File]::WriteAllText((Join-Path $here 'data-dir.txt'), $found, [System.Text.Encoding]::Default)

Say "Setup is done.`n`nDiablo folder: $found`n`nStart the game with 'Play Diablo.bat'. Free movement (and the first-person view on X) is in Settings > Gameplay."
exit 0
