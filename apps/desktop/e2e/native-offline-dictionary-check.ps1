param(
  [Parameter(Mandatory = $true)]
  [string]$EvidenceRoot
)

$ErrorActionPreference = 'Stop'
$taskWorkspace = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$taskEvidence = [IO.Path]::GetFullPath($EvidenceRoot)
New-Item -ItemType Directory -Path $taskEvidence -ErrorAction Stop | Out-Null
$taskLog = Join-Path $taskEvidence 'native-bundle.log'
$taskResult = [ordered]@{
  status = 'failed'
  physical_adapters_up_before = @(Get-NetAdapter -Physical | Where-Object Status -EQ 'Up').Count
}
$taskExit = 1
$taskClient = $null

try {
  $taskDeadline = [DateTime]::UtcNow.AddSeconds(180)
  while (@(Get-NetAdapter -Physical | Where-Object Status -EQ 'Up').Count -gt 0) {
    if ([DateTime]::UtcNow -ge $taskDeadline) {
      throw 'No whole-machine offline interval was observed within 180 seconds.'
    }
    Start-Sleep -Seconds 1
  }
  $taskResult.offline_started_at = [DateTime]::UtcNow.ToString('o')
  $taskClient = [Net.Http.HttpClient]::new()
  $taskClient.Timeout = [TimeSpan]::FromSeconds(3)
  $taskOnline = $false
  try {
    $taskResponse = $taskClient.GetAsync('https://example.com/').GetAwaiter().GetResult()
    $taskResponse.Dispose()
    $taskOnline = $true
  } catch {
    $taskResult.internet_probe_error = $_.Exception.Message
  }
  if ($taskOnline) { throw 'The Internet probe succeeded; this is not an offline test.' }

  Push-Location (Join-Path $taskWorkspace 'apps/desktop')
  try {
    node e2e/native-dictionary-check.mjs bundle `
      (Join-Path $taskWorkspace 'target/d2-research/mdx/native-resources') `
      (Join-Path $taskWorkspace 'target/release/lexique-desktop.exe') `
      (Join-Path $taskWorkspace 'target/release/conjugation') `
      (Join-Path $taskWorkspace 'target/release/frequency') `
      (Join-Path $taskEvidence 'native') *> $taskLog
    $taskResult.native_exit_code = $LASTEXITCODE
  } finally { Pop-Location }

  $taskResult.offline_finished_at = [DateTime]::UtcNow.ToString('o')
  $taskResult.physical_adapters_up_after = @(Get-NetAdapter -Physical | Where-Object Status -EQ 'Up').Count
  if ($taskResult.physical_adapters_up_after -ne 0) {
    throw 'A physical network adapter reconnected before native verification finished.'
  }
  if ($taskResult.native_exit_code -ne 0) { throw 'Native offline verification failed; see native-bundle.log.' }
  $taskResult.status = 'passed'
  $taskExit = 0
} catch {
  $taskResult.error = $_.Exception.Message
} finally {
  if ($null -ne $taskClient) { $taskClient.Dispose() }
  $taskResult | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $taskEvidence 'offline-result.json') -Encoding utf8
}

$taskResult | ConvertTo-Json -Depth 4
exit $taskExit
