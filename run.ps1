#Requires -Version 5
<#
.SYNOPSIS
  Run the armssim Arms-warrior gear optimizer (Rust).
.DESCRIPTION
  Builds the Rust armssim.exe with cargo if needed, then optimizes the given
  character with equipped gear and bag/bank items from a single JSON file.
  Every DPS number comes from the bundled WoWSims TBC engine (wowsimcli).
.EXAMPLE
  .\run.ps1
  Runs against the saved sample export (testdata).
.EXAMPLE
  .\run.ps1 -Character .\me.json
  Runs against your own character export (with bagItems included).
#>
[CmdletBinding()]
param(
    [string]$Character,
    [int]$Iterations = 1500,
    [int]$FinalIterations = 20000,
    [ValidateRange(0.0, 1.0)]
    [double]$AoeFraction = 0.3,
    [ValidateSet('all', 'st', 'blend', 'aoe')]
    [string]$Only = 'all'
)

$ErrorActionPreference = 'Stop'

# $PSScriptRoot is not reliably populated while binding parameter defaults,
# so the sample export is resolved here instead.
if (-not $Character) {
    $Character = Join-Path $PSScriptRoot 'testdata\sample-arms.json'
}
$crate = Join-Path $PSScriptRoot 'armssim-rs'
$engine = Join-Path $PSScriptRoot 'engine'
$exe = Join-Path $crate 'target\release\armssim.exe'

# Ensure cargo (and the rest of the user/machine PATH) is available.
$env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' +
            [Environment]::GetEnvironmentVariable('Path', 'User')

if (-not (Test-Path $exe)) {
    Write-Host 'Building armssim.exe (cargo build --release) ...' -ForegroundColor Cyan
    Push-Location $crate
    try { & cargo build --release } finally { Pop-Location }
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit $LASTEXITCODE)" }
}

if (-not (Test-Path $Character)) { throw "Character export not found: $Character" }

$aoeArg = $AoeFraction.ToString([System.Globalization.CultureInfo]::InvariantCulture)

& $exe `
    --iterations $Iterations `
    --final-iterations $FinalIterations `
    --aoe-fraction $aoeArg `
    --only $Only `
    --engine $engine `
    $Character
