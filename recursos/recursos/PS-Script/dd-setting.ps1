$ErrorActionPreference = 'Stop'

$logDir  = "C:\WSDD-Environment\logs"
$logFile = "$logDir\dd-setting.log"
if (-not (Test-Path $logDir)) { New-Item -ItemType Directory -Path $logDir -Force | Out-Null }
"$(Get-Date -Format 'HH:mm:ss') START dd-setting.ps1" | Set-Content $logFile

function Log { param($msg) "$(Get-Date -Format 'HH:mm:ss') $msg" | Add-Content $logFile; Write-Output $msg }

Log "STEP 1: Iniciado"

$settingsToUpdate = @{
  exposeDockerAPIOnTCP2375 = $true
  updateHostsFile          = $true
  runWinServiceInWslMode   = $true
  useResourceSaver         = $false
  openUIOnStartupDisabled  = $true
}

# Detectar ruta del archivo de settings (settings-store.json en versiones recientes)
$settingsPath = "$env:APPDATA\Docker\settings-store.json"
if (-not (Test-Path $settingsPath)) {
    $settingsPath = "$env:APPDATA\Docker\settings.json"
}
Log "STEP 2: Settings path = $settingsPath"

$trackUpdates = 0

if (Test-Path $settingsPath) {
  Log "STEP 3: Leyendo y verificando settings"
  try {
    $settingsContent = Get-Content $settingsPath -Raw
    $settingsObject  = $settingsContent | ConvertFrom-Json

    foreach ($update in $settingsToUpdate.GetEnumerator()) {
      $target = $settingsObject.PSObject.Properties[$update.Key]
      if ($null -eq $target -or $target.Value -ne $update.Value) {
        Add-Member -InputObject $settingsObject -MemberType NoteProperty -Name $update.Key -Value $update.Value -Force
        $trackUpdates++
      }
    }

    if ($trackUpdates -gt 0) {
      $settingsObject | ConvertTo-Json -Depth 10 | Set-Content $settingsPath
      Log "STEP 4: Settings actualizados ($trackUpdates cambios) — se requiere reinicio"
    } else {
      Log "STEP 4: Settings ya correctos — no se requiere reinicio de Docker"
    }
  } catch {
    Log "ERROR leyendo settings: $_ — se asume reinicio necesario"
    $trackUpdates = 1
  }
} else {
  Log "STEP 3: Archivo de settings no encontrado — Docker Desktop lo creara al iniciar"
  $trackUpdates = 0
}

if ($trackUpdates -gt 0) {
  Log "STEP 5: Reiniciando entorno Docker para aplicar configuracion"

  Log "STEP 5a: Cerrando procesos UI Docker"
  Stop-Process -Name "Docker Desktop" -Force -ErrorAction SilentlyContinue
  Stop-Process -Name "DockerDesktop"  -Force -ErrorAction SilentlyContinue

  Log "STEP 5b: Cerrando todos los procesos docker"
  Get-Process *docker* -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

  Log "STEP 5c: Deteniendo servicios"
  Stop-Service com.docker.service -Force -ErrorAction SilentlyContinue

  Log "STEP 5d: Apagando WSL"
  wsl --shutdown

  Log "STEP 5e: Configurando startup type"
  Set-Service -Name "com.docker.service" -StartupType Automatic -ErrorAction SilentlyContinue
} else {
  Log "STEP 5: Sin cambios en settings — omitiendo reinicio"
}

function Get-DockerDesktopPath {
  $candidates = @(
    "$env:ProgramFiles\Docker\Docker\Docker Desktop.exe",
    "${env:ProgramFiles(x86)}\Docker\Docker\Docker Desktop.exe"
  )

  foreach ($candidate in $candidates) {
    if (Test-Path $candidate) {
      return $candidate
    }
  }

  return $null
}

function Wait-ServiceRunning {
  param([string] $Name, [int] $TimeoutSeconds = 120)

  $waitUntil = [datetime]::Now.AddSeconds($TimeoutSeconds)
  do {
    $service = Get-Service -Name $Name -ErrorAction SilentlyContinue
    if ($null -ne $service -and $service.Status -eq 'Running') {
      return $true
    }
    Start-Sleep -Seconds 1
  } until ([datetime]::Now -ge $waitUntil)

  return $false
}

function Wait-DockerReady {
  param([int] $TimeoutSeconds = 120)

  $waitUntil = [datetime]::Now.AddSeconds($TimeoutSeconds)
  do {
    $pipeOpen = Test-Path -LiteralPath \\.\pipe\docker_engine
    docker info 2>&1 | Out-Null
    if ($pipeOpen -and $LASTEXITCODE -eq 0) {
      return $true
    }
    Start-Sleep -Milliseconds 500
  } until ([datetime]::Now -ge $waitUntil)

  return $false
}

$dockerDesktopFilePath = Get-DockerDesktopPath
if ([string]::IsNullOrWhiteSpace($dockerDesktopFilePath)) {
  Log "ERROR: docker-desktop-not-found"
  Write-Warning "Error: docker-desktop-not-found"
  exit 1
}

Log "STEP 6: Iniciando servicio com.docker.service"
Start-Service -Name "com.docker.service" -ErrorAction SilentlyContinue

if (-not (Wait-ServiceRunning -Name 'com.docker.service')) {
  Log "ERROR: service-timeout"
  Write-Warning "Error: service-timeout"
  exit 1
}

Log "STEP 7: Lanzando Docker Desktop"
Start-Process -FilePath $dockerDesktopFilePath

Log "STEP 8: Validando Docker daemon"
if (-not (Wait-DockerReady)) {
  Log "ERROR: docker-timeout"
  Write-Warning "Error: docker-timeout"
  exit 1
}

Log "STEP 9: Docker daemon OK"

Log "DONE: Continue"
Write-Output 'Continue'
