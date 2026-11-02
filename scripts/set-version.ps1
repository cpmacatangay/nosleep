#requires -Version 5.1
<#
    Bump the version everywhere it is written down.

    Versioning is calendar based: YYYY.MM.N, where N counts releases within the
    month starting at 1. Examples: 2026.11.1, 2026.11.2, 2026.12.1.

    The version lives in exactly two files: Cargo.toml (the source of truth for
    the binary) and the line under the title in README.md. Cargo.lock follows from
    cargo itself. The release workflow refuses to publish when the tag, Cargo.toml
    and the README disagree, so this script is the only supported way to change it.

    Usage:
        .\scripts\set-version.ps1        # next release for the current month
        .\scripts\set-version.ps1 2026.12.1
#>
param(
    [Parameter(Position = 0)]
    [string]$Version = ''
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

Push-Location $root
try {
    if ([string]::IsNullOrWhiteSpace($Version)) {
        $month = (Get-Date).ToString('yyyy.MM')
        $existing = & git tag --list "v$month.*" 2>$null
        $highest = 0
        foreach ($tag in $existing) {
            if ($tag -match "^v$([regex]::Escape($month))\.(\d+)$") {
                $n = [int]$Matches[1]
                if ($n -gt $highest) { $highest = $n }
            }
        }
        $Version = "$month.$($highest + 1)"
    }

    if ($Version -notmatch '^\d{4}\.(0[1-9]|1[0-2])\.\d+$') {
        throw "Version must be calendar form YYYY.MM.N, e.g. 2026.11.1 - got '$Version'"
    }

    function Set-Text([string]$path, [string]$text) {
        # WriteAllText emits UTF-8 without a BOM, which Set-Content does not do on
        # Windows PowerShell.
        [System.IO.File]::WriteAllText($path, $text, (New-Object System.Text.UTF8Encoding($false)))
    }

    $cargoPath = Join-Path $root 'Cargo.toml'
    $cargo = [System.IO.File]::ReadAllText($cargoPath)
    $updated = [regex]::Replace($cargo, '(?m)^version = "[^"]+"', "version = `"$Version`"", 1)
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
