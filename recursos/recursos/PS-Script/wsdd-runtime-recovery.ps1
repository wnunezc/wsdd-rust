param(
  [ValidateSet('Diagnose', 'Recover')]
  [string]$Mode = 'Recover',
  [string]$EnvironmentRoot = 'C:\WSDD-Environment',
  [int]$DockerReadyTimeoutSeconds = 180,
  [ValidateRange(1, 10)]
  [int]$MaxRecoveryFailures = 3,
  [switch]$ResetCircuit
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
try {
  . (Join-Path $PSScriptRoot 'wsdd-diagnostic-filter.ps1')
  Initialize-WsddDiagnosticFilter -EnvironmentRoot $EnvironmentRoot
} catch {
  function Protect-WsddDiagnosticText { param([string]$Text) '[Diagnostic suppressed: credential filter unavailable]' }
}
$script:LogPath = Join-Path $EnvironmentRoot 'logs\wsl-runtime-recovery.log'
$script:StatePath = Join-Path $EnvironmentRoot 'state\wsl-runtime-recovery.json'
$script:Mutex = $null

function Rotate-RecoveryLog {
  $directory = Split-Path -Parent $script:LogPath
  New-Item -ItemType Directory -Path $directory -Force | Out-Null
  if (-not (Test-Path -LiteralPath $script:LogPath)) { return }
  if ((Get-Item -LiteralPath $script:LogPath).Length -lt 2MB) { return }

  for ($index = 3; $index -ge 1; $index--) {
    $source = if ($index -eq 1) { $script:LogPath } else { "$script:LogPath.$($index - 1)" }
    $target = "$script:LogPath.$index"
    if (Test-Path -LiteralPath $source) {
      Move-Item -LiteralPath $source -Destination $target -Force
    }
  }
}

function Write-RecoveryLog {
  param(
    [ValidateSet('INFO', 'WARN', 'ERROR')]
    [string]$Level,
    [string]$Message
  )
  $safe = Protect-WsddDiagnosticText -Text $Message
  $line = '{0} [{1}] {2}' -f (Get-Date -Format 'yyyy-MM-dd HH:mm:ss.fff'), $Level, $safe
  Add-Content -LiteralPath $script:LogPath -Value $line -Encoding UTF8
}

function Read-RecoveryState {
  if (-not (Test-Path -LiteralPath $script:StatePath)) {
    return @{ Version = 1; Runtimes = @{} }
  }
  try {
    $state = Get-Content -LiteralPath $script:StatePath -Raw | ConvertFrom-Json -AsHashtable
    if ($null -eq $state.Runtimes) { $state.Runtimes = @{} }
    return $state
  } catch {
    Write-RecoveryLog WARN "Ignoring invalid recovery state: $($_.Exception.Message)"
    return @{ Version = 1; Runtimes = @{} }
  }
}

function Save-RecoveryState {
  param([hashtable]$State)
  $directory = Split-Path -Parent $script:StatePath
  New-Item -ItemType Directory -Path $directory -Force | Out-Null
  $temporary = "$($script:StatePath).tmp"
  $State | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $temporary -Encoding UTF8
  Move-Item -LiteralPath $temporary -Destination $script:StatePath -Force
}

function Get-RecoveryFingerprint {
  param(
    [string]$ContainerName,
    [object[]]$Targets
  )
  $parts = @($ContainerName, 'BrokenWslRelay')
  foreach ($target in ($Targets | Sort-Object VolumeName)) {
    $parts += @(
      [string]$target.VolumeName,
      [string]$target.Distro,
      [string]$target.LinuxPath,
      [string]$target.WindowsUnc
    )
  }
  $bytes = [Text.Encoding]::UTF8.GetBytes(($parts -join '|'))
  [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes))
}

function Clear-RecoveryFailure {
  param(
    [hashtable]$State,
    [string]$ContainerName
  )
  if ($State.Runtimes.ContainsKey($ContainerName)) {
    [void]$State.Runtimes.Remove($ContainerName)
    Save-RecoveryState $State
  }
}

function Test-RecoveryBlocked {
  param(
    [hashtable]$State,
    [string]$ContainerName,
    [string]$Fingerprint
  )
  if (-not $State.Runtimes.ContainsKey($ContainerName)) { return $false }
  $entry = $State.Runtimes[$ContainerName]
  if ([string]$entry.Fingerprint -ne $Fingerprint) {
    [void]$State.Runtimes.Remove($ContainerName)
    Save-RecoveryState $State
    return $false
  }
  [bool]$entry.RecoveryBlocked
}

function Register-RecoveryFailure {
  param(
    [hashtable]$State,
    [string]$ContainerName,
    [string]$Fingerprint
  )
  $count = 1
  if ($State.Runtimes.ContainsKey($ContainerName)) {
    $entry = $State.Runtimes[$ContainerName]
    if ([string]$entry.Fingerprint -eq $Fingerprint) {
      $count = [int]$entry.ConsecutiveFailures + 1
    }
  }
  $blocked = $count -ge $MaxRecoveryFailures
  $State.Runtimes[$ContainerName] = @{
    Fingerprint = $Fingerprint
    ConsecutiveFailures = $count
    RecoveryBlocked = $blocked
    UpdatedAt = (Get-Date).ToString('o')
  }
  Save-RecoveryState $State
  if ($blocked) {
    Write-RecoveryLog ERROR "RecoveryBlocked for '$ContainerName' after $count consecutive failures; automatic reconstruction is disabled until the runtime is healthy or its configuration changes"
  } else {
    Write-RecoveryLog WARN "Recovery failure $count/$MaxRecoveryFailures for '$ContainerName'"
  }
}

function Enter-RecoveryMutex {
  $created = $false
  $script:Mutex = [Threading.Mutex]::new($true, 'Global\WSDD-WslRuntimeRecovery', [ref]$created)
  if ($created) { return $true }
  try {
    return $script:Mutex.WaitOne(0)
  } catch [Threading.AbandonedMutexException] {
    return $true
  }
}

function Invoke-Captured {
  param(
    [string]$FilePath,
    [string[]]$Arguments
  )
  $output = @(& $FilePath @Arguments 2>&1)
  [pscustomobject]@{
    ExitCode = $LASTEXITCODE
    Output = ($output | ForEach-Object { $_.ToString() }) -join "`n"
  }
}

function Read-WsddSettings {
  $path = Join-Path $EnvironmentRoot 'wsdd-config.json'
  if (-not (Test-Path -LiteralPath $path)) { return $null }
  Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
}

function Read-WsddProjects {
  $directory = Join-Path $EnvironmentRoot 'Docker-Structure\projects'
  if (-not (Test-Path -LiteralPath $directory)) { return @() }
  @(
    Get-ChildItem -LiteralPath $directory -Filter '*.json' -File |
      ForEach-Object {
        try {
          Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
        } catch {
          Write-RecoveryLog WARN "Ignoring invalid project metadata '$($_.FullName)': $($_.Exception.Message)"
        }
      }
  )
}

function Get-PhpContract {
  param([string]$PhpVersion)
  $suffix = switch ($PhpVersion) {
    'Php56' { '56' }
    'Php72' { '72' }
    'Php74' { '74' }
    'Php81' { '81' }
    'Php82' { '82' }
    'Php83' { '83' }
    'Php84' { '84' }
    default { return $null }
  }
  [pscustomobject]@{
    ComposeTag = "php$suffix"
    Directory = 'php{0}.{1}' -f $suffix[0], $suffix[1]
    RuntimeTag = 'PHP{0}.{1}' -f $suffix[0], $suffix[1]
    Service = "webserver$suffix"
  }
}

function Convert-WslUnc {
  param([string]$Path)
  if ($Path -notmatch '^\\\\wsl(?:\.localhost|\$)\\([^\\]+)\\(.+)$') { return $null }
  $distro = $Matches[1]
  $tail = $Matches[2]
  [pscustomobject]@{
    Distro = $distro
    LinuxPath = '/' + ($tail -replace '\\', '/')
    WindowsUnc = "\\wsl.localhost\$distro\$tail"
  }
}

function Resolve-WslRuntimeTargets {
  param(
    [object[]]$Projects,
    [object]$Settings
  )
  $targets = @()
  foreach ($project in $Projects) {
    $mode = [string]$project.storage_mode
    $resolved = $null

    if ($mode -eq 'managed_native_linux_storage') {
      if ($null -eq $project.managed_storage) {
        Write-RecoveryLog WARN "Managed project '$($project.name)' has no managed_storage metadata"
        continue
      }
      $resolved = [pscustomobject]@{
        Distro = [string]$project.managed_storage.distro
        LinuxPath = [string]$project.managed_storage.linux_path
        WindowsUnc = [string]$project.managed_storage.windows_unc
      }
    } elseif ($mode -eq 'native_linux_project') {
      $workPath = [string]$project.work_path
      $resolved = Convert-WslUnc $workPath
      if (($null -eq $resolved) -and $workPath.StartsWith('/')) {
        $defaultDistro = [string]$Settings.wsl_distro
        if (-not [string]::IsNullOrWhiteSpace($defaultDistro)) {
          $tail = $workPath.TrimStart('/') -replace '/', '\'
          $resolved = [pscustomobject]@{
            Distro = $defaultDistro
            LinuxPath = $workPath
            WindowsUnc = "\\wsl.localhost\$defaultDistro\$tail"
          }
        } else {
          Write-RecoveryLog WARN "Native project '$($project.name)' uses an absolute Linux path without a configured WSL distro"
        }
      }
    }

    if ($null -eq $resolved) { continue }
    $contract = Get-PhpContract ([string]$project.php_version)
    if ($null -eq $contract) {
      Write-RecoveryLog WARN "Project '$($project.name)' uses unsupported PHP metadata '$($project.php_version)'"
      continue
    }

    $targets += [pscustomobject]@{
      ProjectName = [string]$project.name
      Domain = [string]$project.domain
      PhpVersion = [string]$project.php_version
      VolumeName = "$($contract.ComposeTag)-$($project.domain)"
      ContainerName = "WSDD-Web-Server-$($contract.RuntimeTag)"
      Distro = [string]$resolved.Distro
      LinuxPath = [string]$resolved.LinuxPath
      WindowsUnc = [string]$resolved.WindowsUnc
      Contract = $contract
    }
  }
  @($targets)
}

function Wait-DockerReady {
  # Equivalent to docker info; kept as direct arguments for safe process invocation.
  $deadline = (Get-Date).AddSeconds($DockerReadyTimeoutSeconds)
  do {
    $result = Invoke-Captured docker @('info', '--format', '{{.ServerVersion}}')
    if ($result.ExitCode -eq 0) { return $true }
    Start-Sleep -Seconds 5
  } while ((Get-Date) -lt $deadline)
  $false
}

function Test-TargetSource {
  param([object]$Target)
  $wsl = Invoke-Captured wsl.exe @('-d', $Target.Distro, '--', 'test', '-d', $Target.LinuxPath)
  if ($wsl.ExitCode -ne 0) {
    Write-RecoveryLog ERROR "Linux source missing for '$($Target.ProjectName)': $($Target.Distro):$($Target.LinuxPath)"
    return $false
  }
  if (-not (Test-Path -LiteralPath $Target.WindowsUnc)) {
    Write-RecoveryLog ERROR "UNC source is not reachable for '$($Target.ProjectName)': $($Target.WindowsUnc)"
    return $false
  }
  $true
}

function Get-ContainerState {
  param([string]$ContainerName)
  $result = Invoke-Captured docker @(
    'inspect', $ContainerName,
    '--format', '{{.State.Status}}|{{.State.Error}}'
  )
  if ($result.ExitCode -ne 0) { return $null }
  $parts = $result.Output.Trim() -split '\|', 2
  [pscustomobject]@{
    Status = $parts[0]
    Error = if ($parts.Count -gt 1) { $parts[1] } else { '' }
  }
}

function Test-ContainerTargets {
  param(
    [string]$ContainerName,
    [object[]]$Targets
  )
  foreach ($target in $Targets) {
    $containerPath = "/var/www/html/$($target.Domain)"
    $result = Invoke-Captured docker @('exec', $ContainerName, 'test', '-d', $containerPath)
    if ($result.ExitCode -ne 0) {
      Write-RecoveryLog WARN "Container '$ContainerName' cannot read '$containerPath'"
      return $false
    }
  }
  $apache = Invoke-Captured docker @('exec', $ContainerName, 'sh', '-lc', 'pgrep -x apache2 >/dev/null')
  $apache.ExitCode -eq 0
}

function Get-RuntimeHealth {
  param(
    [string]$ContainerName,
    [object[]]$Targets
  )
  $state = Get-ContainerState $ContainerName
  if ($null -eq $state) { return 'NotProvisioned' }
  if ($state.Status -eq 'running') {
    if (Test-ContainerTargets $ContainerName $Targets) { return 'Healthy' }
    return 'UnhealthyNonRelay'
  }
  if ($state.Error -match 'docker-desktop-bind-mounts.+no such file or directory') {
    return 'BrokenWslRelay'
  }
  'StoppedStartable'
}

function Invoke-RuntimeStart {
  param(
    [string]$ContainerName,
    [object[]]$Targets
  )
  $start = Invoke-Captured docker @('start', $ContainerName)
  if (($start.ExitCode -eq 0) -and (Test-ContainerTargets $ContainerName $Targets)) {
    Write-RecoveryLog INFO "Runtime '$ContainerName' started without recreation"
    return 'Healthy'
  }
  $health = Get-RuntimeHealth $ContainerName $Targets
  if ($health -eq 'BrokenWslRelay') { return $health }
  Write-RecoveryLog ERROR "Runtime '$ContainerName' could not start, but no broken WSL relay was confirmed: $($start.Output)"
  'UnhealthyNonRelay'
}

function Invoke-ComposeUp {
  param([object]$Target)
  $phpDirectory = Join-Path $EnvironmentRoot "Docker-Structure\bin\$($Target.Contract.Directory)"
  $webserver = Join-Path $phpDirectory "webserver.$($Target.Contract.ComposeTag).yml"
  $options = Join-Path $phpDirectory "options.$($Target.Contract.ComposeTag).yml"
  if (-not (Test-Path -LiteralPath $webserver) -or -not (Test-Path -LiteralPath $options)) {
    throw "Compose files are missing for $($Target.PhpVersion)"
  }

  Push-Location $phpDirectory
  try {
    $legacy = Get-Command docker-compose -ErrorAction SilentlyContinue
    if ($null -ne $legacy) {
      return Invoke-Captured docker-compose @(
        '-p', 'wsdd-projects', '-f', $webserver, '-f', $options,
        'up', '-d', '--no-build', '--force-recreate'
      )
    }
    Invoke-Captured docker @(
      'compose', '-p', 'wsdd-projects', '-f', $webserver, '-f', $options,
      'up', '-d', '--no-build', '--force-recreate'
    )
  } finally {
    Pop-Location
  }
}

function Invoke-RuntimeRecovery {
  param(
    [string]$ContainerName,
    [object[]]$Targets
  )
  foreach ($target in $Targets) {
    if (-not (Test-TargetSource $target)) { return $false }
  }

  if ($Mode -eq 'Diagnose') {
    Write-RecoveryLog WARN "Runtime '$ContainerName' requires recovery; Diagnose mode made no changes"
    return $true
  }

  Write-RecoveryLog WARN "Rehydrating WSL volumes for '$ContainerName'"
  if ($null -ne (Get-ContainerState $ContainerName)) {
    [void](Invoke-Captured docker @('stop', $ContainerName))
    $removeContainer = Invoke-Captured docker @('rm', $ContainerName)
    if ($removeContainer.ExitCode -ne 0) {
      Write-RecoveryLog ERROR "Could not remove '$ContainerName': $($removeContainer.Output)"
      return $false
    }
  }

  foreach ($target in $Targets) {
    [void](Invoke-Captured docker @('volume', 'rm', $target.VolumeName))
    $create = Invoke-Captured docker @(
      'volume', 'create', '--driver', 'local',
      '--opt', 'type=none',
      '--opt', "device=$($target.WindowsUnc)",
      '--opt', 'o=bind',
      $target.VolumeName
    )
    if ($create.ExitCode -ne 0) {
      Write-RecoveryLog ERROR "Could not recreate volume '$($target.VolumeName)': $($create.Output)"
      return $false
    }
  }

  $compose = Invoke-ComposeUp $Targets[0]
  if ($compose.ExitCode -ne 0) {
    Write-RecoveryLog ERROR "Compose failed for '$ContainerName': $($compose.Output)"
    return $false
  }

  $deadline = (Get-Date).AddSeconds(120)
  do {
    $current = Get-ContainerState $ContainerName
    if (($null -ne $current) -and ($current.Status -eq 'running') -and
        (Test-ContainerTargets $ContainerName $Targets)) {
      Write-RecoveryLog INFO "Recovered runtime '$ContainerName'"
      return $true
    }
    Start-Sleep -Seconds 3
  } while ((Get-Date) -lt $deadline)

  Write-RecoveryLog ERROR "Runtime '$ContainerName' did not reach healthy postconditions"
  $false
}

try {
  Rotate-RecoveryLog
  if (-not (Enter-RecoveryMutex)) {
    Write-RecoveryLog INFO 'Another recovery instance is active; exiting'
    exit 0
  }

  Write-RecoveryLog INFO "Starting WSL runtime recovery in $Mode mode"
  $recoveryState = Read-RecoveryState
  if ($ResetCircuit) {
    $recoveryState = @{ Version = 1; Runtimes = @{} }
    Save-RecoveryState $recoveryState
    Write-RecoveryLog INFO 'Recovery circuit state reset explicitly'
  }
  $settings = Read-WsddSettings
  $projects = Read-WsddProjects
  $targets = Resolve-WslRuntimeTargets $projects $settings
  if ($targets.Count -eq 0) {
    Write-RecoveryLog INFO 'No WSL runtime targets'
    exit 0
  }

  foreach ($distro in ($targets.Distro | Sort-Object -Unique)) {
    [void](Invoke-Captured wsl.exe @('-d', $distro, '--', 'true'))
  }

  if (-not (Wait-DockerReady)) {
    Write-RecoveryLog INFO 'Docker Desktop is not ready; exiting without error'
    exit 0
  }

  $failed = $false
  foreach ($group in ($targets | Group-Object PhpVersion)) {
    $runtimeTargets = @($group.Group)
    $containerName = [string]$runtimeTargets[0].ContainerName
    $health = Get-RuntimeHealth $containerName $runtimeTargets
    Write-RecoveryLog INFO "Runtime '$containerName' health: $health"

    if ($Mode -eq 'Diagnose') {
      if (($health -ne 'Healthy') -and ($health -ne 'NotProvisioned')) {
        [void](Invoke-RuntimeRecovery $containerName $runtimeTargets)
      }
      continue
    }

    switch ($health) {
      'Healthy' {
        Clear-RecoveryFailure $recoveryState $containerName
        continue
      }
      'NotProvisioned' {
        Clear-RecoveryFailure $recoveryState $containerName
        Write-RecoveryLog INFO "Runtime '$containerName' is not provisioned; no action required"
        continue
      }
      'StoppedStartable' {
        $health = Invoke-RuntimeStart $containerName $runtimeTargets
        if ($health -eq 'Healthy') {
          Clear-RecoveryFailure $recoveryState $containerName
          continue
        }
        if ($health -ne 'BrokenWslRelay') {
          $failed = $true
          continue
        }
      }
      'UnhealthyNonRelay' {
        Write-RecoveryLog ERROR "Runtime '$containerName' is unhealthy, but no broken WSL relay was confirmed; automatic reconstruction skipped"
        $failed = $true
        continue
      }
    }

    if ($health -eq 'BrokenWslRelay') {
      $fingerprint = Get-RecoveryFingerprint $containerName $runtimeTargets
      if (Test-RecoveryBlocked $recoveryState $containerName $fingerprint) {
        Write-RecoveryLog ERROR "RecoveryBlocked for '$containerName'; automatic reconstruction skipped"
        $failed = $true
        continue
      }
      if (Invoke-RuntimeRecovery $containerName $runtimeTargets) {
        Clear-RecoveryFailure $recoveryState $containerName
      } else {
        Register-RecoveryFailure $recoveryState $containerName $fingerprint
        $failed = $true
      }
    } else {
      switch ($health) {
        'Healthy' { continue }
        'NotProvisioned' { continue }
        default {
          Write-RecoveryLog ERROR "Runtime '$containerName' reached unsupported health state '$health'"
          $failed = $true
        }
      }
    }
  }

  if ($failed) { exit 1 }
  exit 0
} catch {
  try { Write-RecoveryLog ERROR $_.Exception.Message } catch {}
  exit 1
} finally {
  if ($null -ne $script:Mutex) {
    try { $script:Mutex.ReleaseMutex() } catch {}
    $script:Mutex.Dispose()
  }
}
