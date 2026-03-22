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
  Invoke-Step "Upstream intake tests" { node --test tests/upstream-intake/upstream-intake.test.mjs }
  Invoke-Step "Upstream intake dry run" { node scripts/intake-upstreams.mjs --dry-run }
  Invoke-Step "Git workflow tooling tests" { node --test tests/git-workflow/git-workflow.test.mjs }
  Invoke-Step "Git worktree manager tests" { node --test tests/git-workflow/git-worktree-manager.test.mjs }
  Invoke-Step "Fixture ledger tests" { node --test tests/fixture-ledger/fixture-ledger.test.mjs }
  Invoke-Step "Fixture snapshot tests" { node --test tests/fixture-ledger/fixture-snapshot.test.mjs }
  Invoke-Step "Browser preview tests" { node --test tests/web-preview/browser-preview.test.mjs }
  Invoke-Step "Fixture ledger check" { node scripts/fixture-ledger.mjs --check }
  Invoke-Step "Fixture snapshot check" { node scripts/check-fixture-snapshot.mjs }
  Invoke-Step "Git review snapshot dry run" {
    $reviewPath = Join-Path ([System.IO.Path]::GetTempPath()) "osm-git-review-smoke.md"
    node scripts/git-review-snapshot.mjs --item TOOL-01 --phase review --note "verify smoke test" --command "node --test tests/git-workflow/git-workflow.test.mjs" --output $reviewPath
  }
  Invoke-Step "Git checkpoint dry run" {
    node scripts/git-tdd-checkpoint.mjs --item TOOL-01 --phase verify --note "verify smoke test" --command "node --test tests/git-workflow/git-workflow.test.mjs" --dry-run
  }
  Invoke-Step "Rust tests" { & $cargo test }
  Invoke-Step "Web unit tests" { npm --prefix web run test }
  Invoke-Step "Web build" { npm --prefix web run build }
  Invoke-Step "Desktop debug build" { node scripts/run-tauri.mjs build --debug }
  Invoke-Step "Web end-to-end tests" { npm --prefix web run e2e }
} finally {
  Pop-Location
}
