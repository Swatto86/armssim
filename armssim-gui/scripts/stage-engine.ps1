#Requires -Version 5
<#
.SYNOPSIS
  Stage the WoWSims engine binary + item database into src-tauri's bundle
  resources directory, so `tauri dev`/`tauri build` can bundle them.
.DESCRIPTION
  Copies from the repo's local engine\ clone (see README "Getting the
  engine"). Run automatically via npm's predev/prebuild hooks.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$source = Join-Path $repoRoot 'engine'
$dest = Join-Path $PSScriptRoot '..\src-tauri\engine-resources'

if (-not (Test-Path (Join-Path $source 'wowsimcli.exe'))) {
    throw "engine\wowsimcli.exe not found at $source -- see README ""Getting the engine"" to build it locally first."
}
if (-not (Test-Path (Join-Path $source 'assets\database\db.json'))) {
    throw "engine\assets\database\db.json not found at $source -- see README ""Getting the engine""."
}

New-Item -ItemType Directory -Force -Path (Join-Path $dest 'assets\database') | Out-Null
Copy-Item (Join-Path $source 'wowsimcli.exe') (Join-Path $dest 'wowsimcli.exe') -Force
Copy-Item (Join-Path $source 'assets\database\db.json') (Join-Path $dest 'assets\database\db.json') -Force

Write-Host "Staged engine resources into $dest" -ForegroundColor Cyan
