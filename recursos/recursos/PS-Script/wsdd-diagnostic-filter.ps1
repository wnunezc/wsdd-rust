# Diagnostic text only. Never changes arguments, captured process data or settings.
function Initialize-WsddDiagnosticFilter {
  param([string]$EnvironmentRoot)
  if ($null -eq $script:WsddDiagnosticSecrets) { $script:WsddDiagnosticSecrets = [Collections.Generic.List[string]]::new() }
  if ($null -eq $script:WsddDiagnosticIdentifiers) { $script:WsddDiagnosticIdentifiers = [Collections.Generic.List[string]]::new() }
  $script:WsddDiagnosticReady = $false
  try {
    foreach ($name in @('wsdd-config.json', 'wsdd-secrets.json')) {
      $path = Join-Path $EnvironmentRoot $name
      if (-not (Test-Path -LiteralPath $path)) { continue }
      $settings = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -ErrorAction Stop
      $entries = @(
        @{ Value = $settings.prereq_credentials.mysql_user; Identifier = $true },
        @{ Value = $settings.prereq_credentials.mysql_password; Identifier = $false },
        @{ Value = $settings.prereq_credentials.mysql_root_password; Identifier = $false },
        @{ Value = $settings.ngrok_authtoken; Identifier = $false }
      )
      foreach ($entry in $settings.webmin_credentials) {
        $entries += @(
          @{ Value = $entry.username; Identifier = $true },
          @{ Value = $entry.password; Identifier = $false }
        )
      }
      foreach ($entry in $entries) {
        $value = $entry.Value
        if ([string]::IsNullOrEmpty([string]$value)) { continue }
        $value = [string]$value
        if ($entry.Identifier) { $target = $script:WsddDiagnosticIdentifiers }
        else { $target = $script:WsddDiagnosticSecrets }
        $json = ConvertTo-Json -InputObject $value -Compress
        $variants = @($value, $value.Replace("'", "''"), $value.Replace("'", "'\''"), $json.Substring(1, $json.Length - 2))
        foreach ($variant in $variants) {
          foreach ($part in @($variant) + @($variant -split '[\r\n]')) {
            if ($part -and $part -ne '[REDACTED]' -and -not $target.Contains($part)) {
              $target.Add($part)
            }
          }
        }
      }
    }
    $script:WsddDiagnosticReady = $true
  } catch {
    # Do not log the exception: it can contain the value that failed to parse.
    $script:WsddDiagnosticReady = $false
  }
}

function Protect-WsddDiagnosticText {
  param([string]$Text)
  if (-not $script:WsddDiagnosticReady) { return '[Diagnostic suppressed: credential filter unavailable]' }
  try {
    $clean = [regex]::Replace($Text, "`e\[[0-9;]*[A-Za-z]", '')
    $fields = @'
(?ix)(\b(?:WEBMIN_USER|WEBMIN_PASS|USERNAME|PASSWORD|HASHED_PASSWORD|MYSQL_ROOT_PASSWORD|MYSQL_PASSWORD|MYSQL_USER|PMA_PASSWORD|PMA_USER|NGROK_AUTHTOKEN|mysql_root_password|mysql_password|mysql_user|ngrok_authtoken)\b["']?\s*[:=]\s*)(\[REDACTED\]|"(?:\\.|[^"\\])*"|'(?:''|\\.|[^'\\])*'|[^\s,;&}\]]+)
'@
    $safe = [regex]::Replace($clean, $fields, [Text.RegularExpressions.MatchEvaluator]{
      param($match)
      $value = $match.Groups[2].Value
      $quote = ''
      if ($value.StartsWith('"') -and $value.EndsWith('"')) { $quote = '"' }
      elseif ($value.StartsWith("'") -and $value.EndsWith("'")) { $quote = "'" }
      $match.Groups[1].Value + $quote + '[REDACTED]' + $quote
    })
    $values = @($script:WsddDiagnosticSecrets | Sort-Object -Property Length -Descending)
    $prefix = @($values | Where-Object { $_.StartsWith('[REDACTED]') -and $_.Length -gt 10 } | ForEach-Object { [regex]::Escape($_) })
    $remaining = @($values | ForEach-Object { [regex]::Escape($_) })
    $pattern = (@($prefix) + @('\[REDACTED\]') + @($remaining)) -join '|'
    $safe = [regex]::Replace($safe, $pattern, '[REDACTED]')
    $identifiers = @($script:WsddDiagnosticIdentifiers | Sort-Object -Property Length -Descending | ForEach-Object { [regex]::Escape($_) })
    if ($identifiers.Count -gt 0) {
      $identifierPattern = '\[REDACTED\]|(?<![\p{L}\p{N}_])(?:' + ($identifiers -join '|') + ')(?![\p{L}\p{N}_])'
      $safe = [regex]::Replace($safe, $identifierPattern, '[REDACTED]')
    }
    $safe
  } catch {
    '[Diagnostic suppressed: credential filter unavailable]'
  }
}
