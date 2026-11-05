param(
  [string]$RepositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../../..')),
  [string]$DesignRoot = $RepositoryRoot
)
$ErrorActionPreference = 'Stop'
$taskFiles = @(
  'README.md', 'GLOSSARY.md', 'docs/issue-24-design.md',
  'docs/issue-24-optimization-plan.md', 'docs/issue-24-test-plan.md',
  'docs/adr/0007-windowed-audio-availability.md',
  'docs/adr/0008-selected-word-audio-content-versions.md',
  'docs/evidence/issue-24/README.md',
  'docs/evidence/issue-24/github-design-review.md',
  'docs/evidence/issue-24/design-validation/results.md'
)
$taskProblems = [System.Collections.Generic.List[string]]::new()
$taskLocalLinks = 0
$taskRepositoryLinks = 0
$taskUtf8 = [System.Text.UTF8Encoding]::new($false, $true)
foreach ($taskRelativeFile in $taskFiles) {
  $taskDocPath = Join-Path $DesignRoot $taskRelativeFile
  if (-not (Test-Path -LiteralPath $taskDocPath -PathType Leaf)) {
    $taskProblems.Add("Missing document: $taskRelativeFile")
    continue
  }
  $taskText = $taskUtf8.GetString([System.IO.File]::ReadAllBytes($taskDocPath))
  $taskLines = $taskText -split '\r?\n'
  for ($taskLineNumber = 0; $taskLineNumber -lt $taskLines.Length; $taskLineNumber++) {
    if ($taskLines[$taskLineNumber] -match '[\t ]+$') {
      $taskProblems.Add("Trailing whitespace: $taskRelativeFile line $($taskLineNumber + 1)")
    }
    if ($taskLines[$taskLineNumber] -match '^#{1,6} ' -and $taskLineNumber + 1 -lt $taskLines.Length -and $taskLines[$taskLineNumber + 1] -ne '') {
      $taskProblems.Add("Heading needs blank line: $taskRelativeFile line $($taskLineNumber + 1)")
    }
  }
  $taskFenceCount = @($taskLines | Where-Object { $_ -match '^\x60{3}' }).Count
  if ($taskFenceCount % 2 -ne 0) { $taskProblems.Add("Unbalanced code fence: $taskRelativeFile") }
  foreach ($taskMatch in [regex]::Matches($taskText, '\[[^\]]+\]\(([^)]+)\)')) {
    $taskTarget = $taskMatch.Groups[1].Value.Trim('<', '>')
    if ($taskTarget -match '^https?://' -or $taskTarget.StartsWith('#')) { continue }
    $taskTarget = [System.Uri]::UnescapeDataString(($taskTarget -split '#', 2)[0])
    $taskActualPath = [System.IO.Path]::GetFullPath((Join-Path (Split-Path $taskDocPath -Parent) $taskTarget))
    if (Test-Path -LiteralPath $taskActualPath -PathType Leaf) {
      $taskLocalLinks++
      continue
    }
    $taskFuturePath = Join-Path $RepositoryRoot $taskRelativeFile
    $taskResolvedPath = [System.IO.Path]::GetFullPath((Join-Path (Split-Path $taskFuturePath -Parent) $taskTarget))
    if (-not $taskResolvedPath.StartsWith([System.IO.Path]::GetFullPath($RepositoryRoot), [System.StringComparison]::OrdinalIgnoreCase)) {
      $taskProblems.Add("Link leaves intended repository: $taskRelativeFile -> $taskTarget")
    } elseif (Test-Path -LiteralPath $taskResolvedPath -PathType Leaf) {
      $taskRepositoryLinks++
    } else {
      $taskProblems.Add("Broken planned-repository link: $taskRelativeFile -> $taskTarget")
    }
  }
}
$taskTestText = Get-Content -Raw -LiteralPath (Join-Path $DesignRoot 'docs/issue-24-test-plan.md')
$taskCaseRows = @($taskTestText -split '\r?\n' | Where-Object {
  $_ -match '^\| T24-[A-Z]\d{2} \|' -and ($_.Split('|').Count -eq 6)
})
$taskCaseIds = @($taskCaseRows | ForEach-Object { [regex]::Match($_, 'T24-[A-Z]\d{2}').Value })
if ($taskCaseIds.Count -ne 50) { $taskProblems.Add("Expected 50 cases; found $($taskCaseIds.Count)") }
if (@($taskCaseIds | Select-Object -Unique).Count -ne 50) { $taskProblems.Add('Duplicate case IDs') }
$taskRedStart = $taskTestText.IndexOf('## 必须先实现的 12 个红回归')
$taskRedEnd = $taskTestText.IndexOf('## 规则追踪', $taskRedStart)
$taskRedIds = @([regex]::Matches($taskTestText.Substring($taskRedStart, $taskRedEnd - $taskRedStart), '^\| (T24-[A-Z]\d{2}) \|', 'Multiline') | ForEach-Object { $_.Groups[1].Value })
if ($taskRedIds.Count -ne 12) { $taskProblems.Add('Expected 12 priority regressions') }
foreach ($taskRedId in $taskRedIds) {
  if ($taskRedId -notin $taskCaseIds) { $taskProblems.Add("Unmapped priority regression: $taskRedId") }
}
$taskQuestionIds = @([regex]::Matches($taskTestText, '^\| (Q[1-9]) ', 'Multiline') | ForEach-Object { $_.Groups[1].Value })
if (@($taskQuestionIds | Select-Object -Unique).Count -ne 9) { $taskProblems.Add('Q1-Q9 mapping incomplete') }
$taskPlanText = Get-Content -Raw -LiteralPath (Join-Path $DesignRoot 'docs/issue-24-optimization-plan.md')
if ($taskPlanText -match '至少 30 次' -or $taskTestText -match 'FNV\(card_id\)') { $taskProblems.Add('Stale sampling or FNV wording') }
if ($taskPlanText -match 'UNIQUE\(key,hash\)') { $taskProblems.Add('Unsafe reused physical version identity constraint') }
$taskReviewText = Get-Content -Raw -LiteralPath (Join-Path $DesignRoot 'docs/evidence/issue-24/github-design-review.md')
$taskCoverageCounts = [ordered]@{}
foreach ($taskCoverageRule in @(
  @{ prefix = 'R'; count = 10; key = 'github_requirements' },
  @{ prefix = 'V'; count = 6; key = 'pre_implementation_gates' },
  @{ prefix = 'T'; count = 12; key = 'upstream_scenarios' },
  @{ prefix = 'F'; count = 5; key = 'future_integration_gates' }
)) {
  $taskCoverageIds = @([regex]::Matches($taskReviewText, "^\| ($($taskCoverageRule.prefix)\d{2}) \|", 'Multiline') | ForEach-Object { $_.Groups[1].Value })
  $taskCoverageCounts[$taskCoverageRule.key] = $taskCoverageIds.Count
  $taskExpectedIds = @(1..$taskCoverageRule.count | ForEach-Object { '{0}{1:D2}' -f $taskCoverageRule.prefix, $_ })
  if ($taskCoverageIds.Count -ne $taskExpectedIds.Count -or @($taskCoverageIds | Select-Object -Unique).Count -ne $taskExpectedIds.Count -or @($taskExpectedIds | Where-Object { $_ -notin $taskCoverageIds }).Count -gt 0) {
    $taskProblems.Add("Incomplete or duplicate GitHub coverage IDs: $($taskCoverageRule.prefix)")
  }
}
foreach ($taskConfirmedFile in @('docs/issue-24-design.md', 'docs/issue-24-optimization-plan.md', 'docs/issue-24-test-plan.md')) {
  $taskConfirmedText = Get-Content -Raw -LiteralPath (Join-Path $DesignRoot $taskConfirmedFile)
  if ($taskConfirmedText -notmatch 'Q10\s*已共同确认') { $taskProblems.Add("Missing Q10 confirmation: $taskConfirmedFile") }
}
foreach ($taskAcceptedAdr in @('docs/adr/0007-windowed-audio-availability.md', 'docs/adr/0008-selected-word-audio-content-versions.md')) {
  $taskAdrText = Get-Content -Raw -LiteralPath (Join-Path $DesignRoot $taskAcceptedAdr)
  if ($taskAdrText -notmatch '(?m)^status: accepted\s*$') { $taskProblems.Add("ADR is not accepted: $taskAcceptedAdr") }
}

$taskResult = [ordered]@{
  status = $(if ($taskProblems.Count -eq 0) { 'passed' } else { 'failed' })
  documents = $taskFiles.Count
  cases = $taskCaseIds.Count
  priority_regressions = $taskRedIds.Count
  questions = $taskQuestionIds.Count
  local_links = $taskLocalLinks
  repository_fallback_links = $taskRepositoryLinks
  github_coverage = $taskCoverageCounts
  validation_scope = 'UTF-8, whitespace, headings, fences, local links, IDs, confirmation status, GitHub coverage, internal contract wording'
  product_tests = 'not run'
  problems = @($taskProblems)
}
$taskResult | ConvertTo-Json -Depth 3
if ($taskProblems.Count -gt 0) { exit 1 }
