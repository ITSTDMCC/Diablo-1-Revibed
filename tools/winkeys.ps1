# winkeys.ps1 -ProcessId N -Steps "vk:0x28;wait:500;vk:0x0D;text:Orig;shot:name.png"
# Posts key messages to one process's main window only (no global SendKeys), and can capture
# that window to a PNG (PrintWindow).
param([int]$ProcessId, [string]$Steps, [string]$ShotDir)

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class W {
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint MapVirtualKey(uint uCode, uint uMapType);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hWnd, out RECT r);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
}
"@
Add-Type -AssemblyName System.Drawing

$p = Get-Process -Id $ProcessId
$h = $p.MainWindowHandle
if ($h -eq [IntPtr]::Zero) { Write-Output "no window"; exit 1 }

function Key([int]$vk, [int]$ext) {
    $sc = [W]::MapVirtualKey($vk, 0)
    $down = [IntPtr](1 -bor ($sc -shl 16) -bor ($ext -shl 24))
    $up = [IntPtr]([int64](1 -bor ($sc -shl 16) -bor ($ext -shl 24) -bor (3 -shl 30)))
    [W]::PostMessage($h, 0x0100, [IntPtr]$vk, $down) | Out-Null
    Start-Sleep -Milliseconds 60
    [W]::PostMessage($h, 0x0101, [IntPtr]$vk, $up) | Out-Null
    Start-Sleep -Milliseconds 60
}

function Shot([string]$name) {
    $r = New-Object W+RECT
    [W]::GetClientRect($h, [ref]$r) | Out-Null
    $bmp = New-Object System.Drawing.Bitmap ([Math]::Max(1, $r.R - $r.L)), ([Math]::Max(1, $r.B - $r.T))
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $hdc = $g.GetHdc()
    [W]::PrintWindow($h, $hdc, 3) | Out-Null
    $g.ReleaseHdc($hdc)
    $g.Dispose()
    $bmp.Save((Join-Path $ShotDir $name), [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

foreach ($step in $Steps.Split(';')) {
    $kv = $step.Split(':', 2)
    switch ($kv[0]) {
        'vk' { Key ([Convert]::ToInt32($kv[1], 16)) 0 }
        'xvk' { Key ([Convert]::ToInt32($kv[1], 16)) 1 }
        'wait' { Start-Sleep -Milliseconds ([int]$kv[1]) }
        'text' { foreach ($c in $kv[1].ToCharArray()) { [W]::PostMessage($h, 0x0102, [IntPtr][int]$c, [IntPtr]1) | Out-Null; Start-Sleep -Milliseconds 60 } }
        'shot' { Shot $kv[1] }
    }
}
Write-Output "done"
