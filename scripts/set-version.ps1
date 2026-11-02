#requires -Version 5.1
<#
    Bump the version everywhere it is written down.

    Versioning is calendar based:

        YYYY.M.D          first release on that day,   e.g. 2026.11.2
        YYYY.M.D+vN       later release that day,      e.g. 2026.11.2+v2

    The month and day carry no leading zero because SemVer rejects them, and the
    sequence uses build metadata because a bare "v2" suffix is not a valid
    version. Everything that reports a version - the tag, Cargo.toml, the README
    and --diagnose - therefore shows the same string.

    The version lives in exactly two files: Cargo.toml (the source of truth for
    the binary) and the line under the title in README.md. Cargo.lock follows from
    cargo itself. The release workflow refuses to publish when the tag, Cargo.toml
    and the README disagree, so this script is the only supported way to change it.

    Usage:
        .\scripts\set-version.ps1        # derive today's next release
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
        $base = (Get-Date).ToString('yyyy.M.d')
        $escaped = [regex]::Escape($base)

        # The bare date is the first release of the day; anything after it is +vN.
        $highest = 0
        if (& git tag --list "v$base") { $highest = 1 }
        foreach ($tag in (& git tag --list "v$base+v*")) {
            if ($tag -match "^v$escaped\+v(\d+)$") {
                $n = [int]$Matches[1]
                if ($n -gt $highest) { $highest = $n }
            }
        }

        $Version = if ($highest -eq 0) { $base } else { "$base+v$($highest + 1)" }
    }

    if ($Version -notmatch '^\d{4}\.([1-9]|1[0-2])\.([1-9]|[12]\d|3[01])(\+v\d+)?$') {
        throw "Version must be YYYY.M.D or YYYY.M.D+vN with no leading zeros, e.g. 2026.11.2 or 2026.11.2+v2 - got '$Version'"
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
    $readmePattern = '\*\*v[0-9]+\.[0-9]+\.[0-9]+(\+v[0-9]+)?\*\*'
    if ($readme -notmatch $readmePattern) {
        throw 'README.md has no "**vX.Y.Z**" line to update'
    }
    Set-Text $readmePath ([regex]::Replace($readme, $readmePattern, "**v$Version**", 1))

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
