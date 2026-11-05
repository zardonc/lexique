param(
  [string]$RepositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
)
$ErrorActionPreference = 'Stop'

function Get-ProjectMarkdown {
  param([string]$Root)
  $taskFiles = @(git -c core.quotepath=false -C $Root ls-files -- '*.md' ':!vendor/**')
  if ($LASTEXITCODE -ne 0) { throw 'Cannot list tracked Markdown files' }
  if ($taskFiles.Count -eq 0) { return }
  $taskAttributes = @(git -c core.quotepath=false -C $Root check-attr text -- $taskFiles)
  if ($LASTEXITCODE -ne 0) { throw 'Cannot identify original source documents' }
  $taskOriginals = @($taskAttributes | Where-Object { $_.EndsWith(': text: unset') } |
    ForEach-Object { $_.Substring(0, $_.Length - ': text: unset'.Length) })
  $taskFiles | Where-Object { $_ -notin $taskOriginals }
}

function Test-Documentation {
  param(
    [string]$Root,
    [string]$DesignRoot = $Root,
    [string[]]$Files
  )
  $taskTrackedFiles = @(git -c core.quotepath=false -C $Root ls-files)
  if ($LASTEXITCODE -ne 0) { throw 'Cannot list repository link targets' }
  $taskPaths = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
  $null = $taskPaths.Add('.')
  foreach ($taskTrackedFile in $taskTrackedFiles) {
    $null = $taskPaths.Add($taskTrackedFile)
    $taskParent = [IO.Path]::GetDirectoryName($taskTrackedFile)
    while ($taskParent) {
      $null = $taskPaths.Add($taskParent.Replace('\', '/'))
      $taskParent = [IO.Path]::GetDirectoryName($taskParent)
    }
  }
  $taskProblems = [System.Collections.Generic.List[string]]::new()
  $taskLocalLinks = 0
  $taskRepositoryLinks = 0
  $taskSkippedLinks = 0
  $taskUtf8 = [System.Text.UTF8Encoding]::new($false, $true)
  foreach ($taskFile in $Files) {
    $taskDocPath = Join-Path $DesignRoot $taskFile
    try { $taskText = $taskUtf8.GetString([IO.File]::ReadAllBytes($taskDocPath)) }
    catch { $taskProblems.Add("Cannot read UTF-8 documentation: $taskFile ($($_.Exception.Message))"); continue }
    $taskLines = $taskText -split '\r?\n'
    $taskProse = [System.Collections.Generic.List[string]]::new()
    $taskFence = ''
    for ($taskLineNumber = 0; $taskLineNumber -lt $taskLines.Length; $taskLineNumber++) {
      $taskLine = $taskLines[$taskLineNumber]
      if ($taskLine -match '[\t ]+$') {
        $taskProblems.Add("Trailing whitespace: $taskFile line $($taskLineNumber + 1)")
      }
      if ($taskFence) {
        $taskClosingFence = '^ {0,3}' + [regex]::Escape($taskFence[0].ToString()) + '{' + $taskFence.Length + ',}[ \t]*$'
        if ($taskLine -match $taskClosingFence) { $taskFence = '' }
        continue
      }
      if ($taskLine -match '^ {0,3}(`{3,}|~{3,})') { $taskFence = $Matches[1]; continue }
      if ($taskLine -match '^#{1,6} ' -and $taskLineNumber + 1 -lt $taskLines.Length -and $taskLines[$taskLineNumber + 1] -ne '') {
        $taskProblems.Add("Heading needs blank line: $taskFile line $($taskLineNumber + 1)")
      }
      $taskProse.Add($taskLine)
    }
    if ($taskFence) { $taskProblems.Add("Unbalanced code fence: $taskFile") }
    if ($taskText -match '\n[\t \r\n]+\z') { $taskProblems.Add("New blank line at EOF: $taskFile") }
    foreach ($taskMatch in [regex]::Matches(($taskProse -join "`n"), '\[[^\]]*\]\(([^)]+)\)')) {
      $taskTarget = $taskMatch.Groups[1].Value.Trim('<', '>')
      if ($taskTarget -match '^(?:[a-zA-Z][a-zA-Z0-9+.-]*:|#|//)') { continue }
      $taskTarget = [Uri]::UnescapeDataString(($taskTarget -split '#', 2)[0])
      $taskPublishedPath = [IO.Path]::GetFullPath((Join-Path (Split-Path (Join-Path $Root $taskFile) -Parent) $taskTarget))
      $taskRelativeTarget = [IO.Path]::GetRelativePath($Root, $taskPublishedPath).Replace('\', '/').TrimEnd('/')
      if ($taskRelativeTarget -eq '..' -or $taskRelativeTarget.StartsWith('../')) {
        $taskProblems.Add("Link leaves repository: $taskFile -> $taskTarget")
      } elseif ($taskRelativeTarget.StartsWith('target/') -or $taskRelativeTarget.StartsWith('.tooling/') -or
        ($taskFile -eq 'AGENTS.md' -and $taskRelativeTarget -eq 'LOCAL-DEV-ENV.md')) {
        $taskSkippedLinks++
      } elseif (-not $taskPaths.Contains($taskRelativeTarget) -or -not (Test-Path -LiteralPath $taskPublishedPath)) {
        $taskProblems.Add("Missing tracked link target: $taskFile -> $taskTarget")
      } elseif (Test-Path -LiteralPath (Join-Path (Split-Path $taskDocPath -Parent) $taskTarget)) {
        $taskLocalLinks++
      } else {
        $taskRepositoryLinks++
      }
    }
  }
  [ordered]@{
    status = $(if ($taskProblems.Count -eq 0) { 'passed' } else { 'failed' })
    documents = $Files.Count
    local_links = $taskLocalLinks
    repository_fallback_links = $taskRepositoryLinks
    skipped_local_links = $taskSkippedLinks
    problems = @($taskProblems)
  }
}

if ($MyInvocation.InvocationName -ne '.') {
  $taskResult = Test-Documentation -Root $RepositoryRoot -Files @(Get-ProjectMarkdown -Root $RepositoryRoot)
  $taskResult | ConvertTo-Json -Depth 3
  if ($taskResult.problems.Count -gt 0) { exit 1 }
}
