function UpdateDockerSettings {
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

    if (-not (Test-Path $settingsPath)) {
        Write-Output ""
        return
    }

    try {
        $settingsContent = Get-Content $settingsPath -Raw
        $settingsObject  = $settingsContent | ConvertFrom-Json
    } catch {
        Write-Output ""
        return
    }

    $trackUpdates = 0
    foreach ($update in $settingsToUpdate.GetEnumerator()) {
        if ($target = $settingsObject.psobject.Properties.Match($update.Key)) {
            if ($target.Value -ne $update.Value) {
                $trackUpdates++
            }
        }
    }

    if ($trackUpdates -eq 0) {
        Write-Output "Updated"
        return
    } else {
        Write-Output ""
        return
    }
}

UpdateDockerSettings
