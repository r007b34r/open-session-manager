$ErrorActionPreference = "Stop"

function Resolve-CargoCommand {
  $candidates = @(
    (Get-Command cargo -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source -ErrorAction SilentlyContinue),
    "$env:USERPROFILE\.cargo\bin\cargo.exe",
    "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe"
  ) | Where-Object { $_ }

  foreach ($candidate in $candidates) {
    if (Test-Path $candidate) {
      return $candidate
    }
  }

  throw "cargo executable was not found in PATH or common local Rust locations."
}

function Invoke-Step {
  param(
    [string]$Name,
    [scriptblock]$Script
  )

  Write-Host "==> $Name"
  & $Script
  if ($LASTEXITCODE -ne 0) {
    throw "$Name failed with exit code $LASTEXITCODE"
  }
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$cargo = Resolve-CargoCommand

Push-Location $repoRoot

try {
  Invoke-Step "Rust tests" { & $cargo test }
  Invoke-Step "Web unit tests" { npm --prefix web run test }
  Invoke-Step "Web build" { npm --prefix web run build }
  Invoke-Step "Desktop debug build" { node scripts/run-tauri.mjs build --debug }
  Invoke-Step "Web end-to-end tests" { npm --prefix web run e2e }
} finally {
  Pop-Location
}
