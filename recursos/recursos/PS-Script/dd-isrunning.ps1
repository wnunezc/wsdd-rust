function CheckDockerRunning {
    # Si el pipe no existe, Docker definitivamente no está corriendo
    # Evita colgar esperando conexión cuando el daemon está detenido
    if (-not (Test-Path -LiteralPath \\.\pipe\docker_engine)) {
        Write-Output ""
        return
    }
    docker ps 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Output "Running"
        return
    } else {
        Write-Output ""
        return
    }
}

CheckDockerRunning
