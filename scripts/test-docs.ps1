$ErrorActionPreference = 'Stop'
$taskRepository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskOwner = Join-Path $taskRepository 'target/ci-guardrails'
New-Item -ItemType Directory -Path $taskOwner -Force | Out-Null
$taskFixture = Join-Path $taskOwner ('docs-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $taskFixture -ErrorAction Stop | Out-Null
$taskResults = [System.Collections.Generic.List[string]]::new()
$taskUtf8 = [Text.UTF8Encoding]::new($false)

function Write-Fixture {
  param([string]$Path, [string]$Text)
  $taskDestination = Join-Path $taskFixture $Path
  New-Item -ItemType Directory -Path (Split-Path $taskDestination -Parent) -Force | Out-Null
  [IO.File]::WriteAllText($taskDestination, $Text, $taskUtf8)
}

function Invoke-Check {
  param([string]$Name, [int]$ExpectedExit, [string]$Problem)
  $taskOutput = @(& pwsh -NoProfile -NonInteractive -File (Join-Path $PSScriptRoot 'check-docs.ps1') -RepositoryRoot $taskFixture)
  if ($LASTEXITCODE -ne $ExpectedExit) { throw "$Name returned $LASTEXITCODE instead of ${ExpectedExit}: $($taskOutput -join ' ')" }
  $taskReport = ($taskOutput -join "`n") | ConvertFrom-Json
  if ($Problem -and ($taskReport.problems -join "`n") -notmatch $Problem) { throw "$Name did not report ${Problem}: $($taskOutput -join ' ')" }
  if ($ExpectedExit -eq 0 -and $taskReport.status -ne 'passed') { throw "$Name did not pass" }
  $taskResults.Add($Name)
  $taskReport
}

try {
  git init --quiet $taskFixture
  if ($LASTEXITCODE -ne 0) { throw 'Cannot initialize documentation fixture' }
  $taskValid = "# Documentation`n`n[guide](docs/guide.md#section)`n[folder](docs/)`n[accent](docs/%C3%A9cole%20guide.md)`n[external](https://example.com)`n"
  Write-Fixture 'README.md' $taskValid
  Write-Fixture 'docs/guide.md' "# Guide`n`nText.`n"
  Write-Fixture 'docs/école guide.md' "# École`n`nText.`n"
  Write-Fixture 'AGENTS.md' "# Guidance`n`n[local environment](LOCAL-DEV-ENV.md)`n"
  Write-Fixture '.gitignore' "local.md`nLOCAL-DEV-ENV.md`ntarget/`n.tooling/`n"
  Write-Fixture '.gitattributes' "* text=auto eol=lf`noriginal.md -text`n"
  Write-Fixture 'original.md' "# Upstream`nOriginal text  `n`n"
  git -C $taskFixture add -- .
  if ($LASTEXITCODE -ne 0) { throw 'Cannot track documentation fixture' }
  $taskReport = Invoke-Check 'tracked files, directories, encoded paths and original sources' 0
  if ($taskReport.documents -ne 4 -or $taskReport.skipped_local_links -ne 1) { throw 'Fixture selection or optional environment link is incorrect' }

  Write-Fixture 'local.md' "# Local`n"
  Write-Fixture 'README.md' "# Documentation`n`n[local](local.md)`n"
  $null = Invoke-Check 'ignored file exists but cannot satisfy a published link' 1 'Missing tracked link target'
  Write-Fixture 'LOCAL-DEV-ENV.md' "# Local`n"
  Write-Fixture 'README.md' "# Documentation`n`n[local](LOCAL-DEV-ENV.md)`n"
  $null = Invoke-Check 'optional environment exception is limited to AGENTS.md' 1 'Missing tracked link target'
  Write-Fixture 'README.md' "# Documentation`n`n[missing](missing.md)`n"
  $null = Invoke-Check 'missing link target' 1 'Missing tracked link target'
  Write-Fixture 'README.md' "# Documentation`n`n[case](docs/Guide.md)`n"
  $null = Invoke-Check 'published path casing' 1 'Missing tracked link target'
  Write-Fixture 'README.md' "# Documentation`n`n[outside](../outside.md)`n"
  $null = Invoke-Check 'link outside repository' 1 'Link leaves repository'
  Write-Fixture 'README.md' "# Documentation`n`nText  `n"
  $null = Invoke-Check 'trailing whitespace' 1 'Trailing whitespace'
  Write-Fixture 'README.md' ($taskValid + "`n")
  $null = Invoke-Check 'blank line at EOF' 1 'New blank line at EOF'
  Write-Fixture 'README.md' "# Documentation`nText.`n"
  $null = Invoke-Check 'heading spacing' 1 'Heading needs blank line'
  Write-Fixture 'README.md' "# Documentation`n`n~~~markdown`n[example](missing.md)`n"
  $null = Invoke-Check 'unbalanced fence' 1 'Unbalanced code fence'
  [IO.File]::WriteAllBytes((Join-Path $taskFixture 'README.md'), [byte[]]@(255))
  $null = Invoke-Check 'invalid UTF-8' 1 'Cannot read UTF-8 documentation'
  Write-Fixture 'README.md' (@('# Documentation', '', '````markdown',
    '[example](missing.md)', '```', '````', '') -join "`n")
  $null = Invoke-Check 'nested fenced examples are not published links' 0
  Write-Fixture 'README.md' "# Documentation`n`n[log](target/retained.log)`n[source](.tooling/source.log)`n"
  $taskReport = Invoke-Check 'local evidence is reported separately on a fresh checkout' 0
  if ($taskReport.skipped_local_links -ne 3) { throw 'Local evidence links were not reported separately' }
  [ordered]@{ status = 'passed'; checks = $taskResults.Count; cases = @($taskResults) } | ConvertTo-Json -Depth 3
} finally {
  $taskResolvedFixture = [IO.Path]::GetFullPath($taskFixture)
  if ([IO.Path]::GetDirectoryName($taskResolvedFixture) -ne [IO.Path]::GetFullPath($taskOwner)) { throw 'Fixture cleanup leaves its owner directory' }
  Remove-Item -LiteralPath $taskResolvedFixture -Recurse -Force
}
