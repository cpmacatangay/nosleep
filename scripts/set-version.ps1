#requires -Version 5.1
<#
    Bump the version everywhere it is written down.

    The version lives in exactly two files: Cargo.toml (the source of truth for
    the binary) and the line under the title in README.md. Cargo.lock follows from
    cargo itself. The release workflow refuses to publish when the tag, Cargo.toml
    and the README disagree, so this script is the only supported way to change it.

    Usage:
        .\scripts\set-version.ps1 0.1.2
#>
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Version
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    throw "Version must look like 1.2.3 (no leading 'v'), got '$Version'"
}

function Set-Text([string]$path, [string]$text) {
    # WriteAllText emits UTF-8 without a BOM, which Set-Content does not do on
    # Windows PowerShell.
    [System.IO.File]::WriteAllText($path, $text, (New-Object System.Text.UTF8Encoding($false)))
}

Push-Location $root
try {
    $cargoPath = Join-Path $root 'Cargo.toml'
    $cargo = [System.IO.File]::ReadAllText($cargoPath)
    $updated = [regex]::Replace(
        $cargo,
        '(?m)^version = "[^"]+"',
        "version = `"$Version`"",
        1
    )
    if ($updated -eq $cargo -and $cargo -notmatch "(?m)^version = `"$([regex]::Escape($Version))`"") {
        throw 'Could not find the package version line in Cargo.toml'
    }
    Set-Text $cargoPath $updated

    $readmePath = Join-Path $root 'README.md'
    $readme = [System.IO.File]::ReadAllText($readmePath)
    if ($readme -notmatch '\*\*v\d+\.\d+\.\d+\*\*') {
        throw 'README.md has no "**vX.Y.Z**" line to update'
    }
    Set-Text $readmePath ([regex]::Replace($readme, '\*\*v\d+\.\d+\.\d+\*\*', "**v$Version**", 1))

    # Refresh Cargo.lock so the lockfile records the new package version.
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    & cargo check --quiet
    if ($LASTEXITCODE -ne 0) { throw 'cargo check failed after the version bump' }

    Write-Host "Version is now $Version" -ForegroundColor Green
    Write-Host ''
    Write-Host 'Next:'
    Write-Host "  git commit -am `"chore: release v$Version`""
    Write-Host "  git tag v$Version"
    Write-Host '  git push origin master --tags'
}
finally {
    Pop-Location
}
