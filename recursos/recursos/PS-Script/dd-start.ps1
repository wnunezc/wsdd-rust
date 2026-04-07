function Start-Docker {
    param()
    $ErrorActionPreference = 'Continue'

    Start-Service -Name "com.docker.service" -ErrorAction SilentlyContinue

    $serviceTimeout = [datetime]::Now.AddSeconds(120)
    while ((Get-Service -Name "com.docker.service").Status -ne "Running") {
        if ([datetime]::Now -ge $serviceTimeout) {
            Write-Warning "Error: service-timeout — com.docker.service no alcanzó estado Running en 120s"
            return
        }
        Start-Sleep -Seconds 1
    }

    if((Get-Service -Name "com.docker.service").Status -eq "Running"){
        $status = (Get-Service -Name "com.docker.service").Status
        Write-Output "Service: $status"
    }

    $dockerDesktopFilePath = $env:ProgramFiles | Join-Path -ChildPath 'Docker\Docker\Docker Desktop.exe'; Start-Process -FilePath $dockerDesktopFilePath

    $ipcTimeout = New-TimeSpan -Seconds 120
    $waitUntil = [datetime]::Now.Add($ipcTimeout)
    $pipeOpen = $false

    Write-Output 'Validando pipe line'

    do {
        Start-Sleep -Milliseconds 500
        $pipeOpen = Test-Path -LiteralPath \\.\pipe\docker_engine
    } until ($pipeOpen -or ($waitUntil -le [datetime]::Now))

    if (-not $pipeOpen) {
        Write-Warning "Error: pipe-line"
        return
    } else {
        Write-Output "Working: pipe-line"
    }

    $responseTimeout = New-TimeSpan -Seconds 120
    $waitUntil = [datetime]::Now.Add($responseTimeout)
    $dockerInfoSuccess = $false

    Write-Output 'Validando docker access'

    do {
        Start-Sleep -Milliseconds 500
        docker info 2>&1 | Out-Null
        $dockerInfoSuccess = ($LASTEXITCODE -eq 0)
    } until ($dockerInfoSuccess -or ($waitUntil -le [datetime]::Now))

    if (-not $dockerInfoSuccess) {
        Write-Warning "Error: docker access"
        return
    } else {
        Write-Output "Working: docker access"
    }

    Write-Output 'Continue'
}

Start-Docker
