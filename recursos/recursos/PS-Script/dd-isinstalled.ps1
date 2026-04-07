function CheckDockerInstalled {
    $DockerDesktopExe = (Get-ChildItem -Path "C:\Program Files\Docker\Docker\Docker Desktop.exe","C:\Program Files (x86)\Docker\Docker\Docker Desktop.exe" -ErrorAction SilentlyContinue)

    if ($null -eq $DockerDesktopExe) {
        Write-Output ""
        return;
    }

    $FileVersion = (Get-Item -Path "$($DockerDesktopExe.FullName)" -ErrorAction SilentlyContinue).VersionInfo.FileVersion

    if ($null -eq $FileVersion) {
        Write-Output ""
        return;
    }

    Write-Output "Installed"
    return;
}

CheckDockerInstalled