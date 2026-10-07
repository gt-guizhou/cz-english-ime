# Generate placeholder icon for CZ-English IME: brand blue #2563EB rounded square + white "CZ".
# Outputs: apps/windows/tsf/resources/czime.ico (256px PNG-in-ICO) + assets/icon/logo.png
# NOTE (ascii-only comments): PS5 parses args like New-Object Type(a, $x - 1, b) as array math,
# so all coordinates are pre-computed into plain variables below.
# Also: keep this file ASCII-only or save with UTF-8 BOM; PS5 reads no-BOM files as ANSI/GBK
# and a trailing multi-byte comment char can swallow the newline after it.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

$size = 256
$edge = $size - 1
$r = 56

$bmp = New-Object System.Drawing.Bitmap($size, $size)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAlias

$blue = [System.Drawing.Color]::FromArgb(255, 37, 99, 235)
$path = New-Object System.Drawing.Drawing2D.GraphicsPath

$x0 = 0
$y0 = 0
$xr = $edge - $r
$yr = $edge - $r

$path.AddArc($x0, $y0, $r, $r, 180, 90)
$path.AddArc($xr, $y0, $r, $r, 270, 90)
$path.AddArc($xr, $yr, $r, $r, 0, 90)
$path.AddArc($x0, $yr, $r, $r, 90, 90)
$path.CloseFigure()

$brush = New-Object System.Drawing.SolidBrush($blue)
$g.FillPath($brush, $path)

$font = New-Object System.Drawing.Font("Segoe UI", 96, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
$fmt = New-Object System.Drawing.StringFormat
$fmt.Alignment = [System.Drawing.StringAlignment]::Center
$fmt.LineAlignment = [System.Drawing.StringAlignment]::Center
$layout = New-Object System.Drawing.RectangleF(0, 0, $size, $size)
$g.DrawString("CZ", $font, [System.Drawing.Brushes]::White, $layout, $fmt)
$g.Dispose()

$root = Split-Path -Parent $PSScriptRoot
$pngPath = Join-Path $root "assets\icon\logo.png"
$bmp.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)

$ms = New-Object System.IO.MemoryStream
$bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
$png = $ms.ToArray()
$bmp.Dispose()

$ico = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($ico)
$bw.Write([UInt16]0); $bw.Write([UInt16]1); $bw.Write([UInt16]1)
$bw.Write([Byte]0); $bw.Write([Byte]0)
$bw.Write([Byte]0); $bw.Write([Byte]0)
$bw.Write([UInt16]1); $bw.Write([UInt16]32)
$bw.Write([UInt32]$png.Length); $bw.Write([UInt32]22)
$bw.Write($png)
$bw.Flush()

$icoPath = Join-Path $root "apps\windows\tsf\resources\czime.ico"
[System.IO.File]::WriteAllBytes($icoPath, $ico.ToArray())
Write-Host "OK: $pngPath / $icoPath ($($png.Length) bytes png)"
