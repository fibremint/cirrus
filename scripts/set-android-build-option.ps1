param (
    [string]$type = ''
)

if (!$type) {
    Write-Host "build type is not set"
    Write-Host "Please specify the build type with --type TYPE"
    Exit
}

$projectRoot = Split-Path -Path $PSScriptRoot
$androidAppPath = "$projectRoot\cirrus-app\src-tauri\gen\android\app\app"
$buildConfigLink = "$androidAppPath\build.gradle.kts"

$buildConfigTarget = switch ( $type ) {
    "debug" { "$buildConfigLink.debug" }
    "release" { "$buildConfigLink.aarch64-release" }
    default { "unknown" }
}

if ($buildConfigTarget -eq "unknown") {
    Write-Host "Invalid build type is given: $type"
    Exit
}

if (Test-Path $buildConfigLink) {
    Remove-Item $buildConfigLink
}

New-Item -ItemType SymbolicLink -path $buildConfigLink -target $buildConfigTarget

# Write-Host "Created build config symlink"
