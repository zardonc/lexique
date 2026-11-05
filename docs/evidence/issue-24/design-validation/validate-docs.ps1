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
. (Join-Path $RepositoryRoot 'scripts/check-docs.ps1') -RepositoryRoot $RepositoryRoot
$taskDocumentation = Test-Documentation -Root $RepositoryRoot -DesignRoot $DesignRoot -Files $taskFiles
$taskProblems = [System.Collections.Generic.List[string]]::new()
$taskProblems.AddRange([string[]]$taskDocumentation.problems)
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
  local_links = $taskDocumentation.local_links
  repository_fallback_links = $taskDocumentation.repository_fallback_links
  skipped_local_links = $taskDocumentation.skipped_local_links
  github_coverage = $taskCoverageCounts
  validation_scope = 'UTF-8, whitespace, headings, fences, tracked links, IDs, confirmation status, GitHub coverage, internal contract wording'
  product_tests = 'not run'
  problems = @($taskProblems)
}
$taskResult | ConvertTo-Json -Depth 3
if ($taskProblems.Count -gt 0) { exit 1 }
