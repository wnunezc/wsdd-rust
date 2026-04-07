$ErrorActionPreference = 'Continue'

$logDir  = "C:\WSDD-Environment\logs"
$logFile = "$logDir\dd-setting.log"
if (-not (Test-Path $logDir)) { New-Item -ItemType Directory -Path $logDir -Force | Out-Null }
"$(Get-Date -Format 'HH:mm:ss') START dd-setting.ps1" | Set-Content $logFile

function Log { param($msg) "$(Get-Date -Format 'HH:mm:ss') $msg" | Add-Content $logFile; Write-Output $msg }

Log "STEP 1: Iniciado"

$settingsToUpdate = @{
  exposeDockerAPIOnTCP2375 = $true
  updateHostsFile          = $true
  licenseTermsVersion      = 2
  noWindowsContainers      = $false
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
      if ($target = $settingsObject.psobject.Properties.Match($update.Key)) {
        if ($target.Value -ne $update.Value) {
          Add-Member -InputObject $settingsObject -MemberType NoteProperty -Name $update.Key -Value $update.Value -Force
          $trackUpdates++
        }
      }
    }

    if ($trackUpdates -gt 0) {
      $settingsObject | ConvertTo-Json | Set-Content $settingsPath
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
  Stop-Service com.docker.backend -Force -ErrorAction SilentlyContinue
  Stop-Service com.docker.service -Force -ErrorAction SilentlyContinue

  Log "STEP 5d: Apagando WSL"
  wsl --shutdown

  Log "STEP 5e: Configurando startup type"
  Set-Service -Name "com.docker.service" -StartupType Automatic -ErrorAction SilentlyContinue
} else {
  Log "STEP 5: Sin cambios en settings — omitiendo reinicio"
}

Log "STEP 6: Iniciando servicio com.docker.service"
Start-Service -Name "com.docker.service" -ErrorAction SilentlyContinue

$serviceTimeout = [datetime]::Now.AddSeconds(120)
while ((Get-Service -Name "com.docker.service").Status -ne "Running") {
  if ([datetime]::Now -ge $serviceTimeout) {
    Log "ERROR: service-timeout — com.docker.service no alcanzo Running en 120s"
    Write-Warning "Error: service-timeout"
    return
  }
  Start-Sleep -Seconds 1
}

$status = (Get-Service -Name "com.docker.service").Status
Log "STEP 7: Service $status — lanzando Docker Desktop"

$dockerDesktopFilePath = Join-Path $env:ProgramFiles 'Docker\Docker\Docker Desktop.exe'
Start-Process -FilePath $dockerDesktopFilePath

$ipcTimeout = New-TimeSpan -Seconds 120
$waitUntil  = [datetime]::Now.Add($ipcTimeout)
$pipeOpen   = $false

Log "STEP 8: Validando pipe"
do {
  Start-Sleep -Milliseconds 500
  $pipeOpen = Test-Path -LiteralPath \\.\pipe\docker_engine
} until ($pipeOpen -or ($waitUntil -le [datetime]::Now))

if (-not $pipeOpen) {
  Log "ERROR: pipe no disponible tras 120s"
  Write-Warning "Error: pipe-timeout"
  return
}
Log "STEP 9: pipe OK"

$responseTimeout = New-TimeSpan -Seconds 120
$waitUntil       = [datetime]::Now.Add($responseTimeout)
$dockerInfoOK    = $false

Log "STEP 10: Validando docker access"
do {
  Start-Sleep -Milliseconds 500
  docker info 2>&1 | Out-Null
  $dockerInfoOK = ($LASTEXITCODE -eq 0)
} until ($dockerInfoOK -or ($waitUntil -le [datetime]::Now))

if (-not $dockerInfoOK) {
  Log "ERROR: docker access no disponible tras 120s"
  Write-Warning "Error: docker-access-timeout"
  return
}
Log "STEP 11: docker access OK"

Log "DONE: Continue"
Write-Output 'Continue'
