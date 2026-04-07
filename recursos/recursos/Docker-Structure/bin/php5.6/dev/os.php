<?php
// Configuración de timezone para mostrar fechas correctas
date_default_timezone_set('UTC');
?>
<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>System Information - Development Environment</title>

    <!-- Bootstrap 5.3.7 CSS -->
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.7/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-LN+7fdVzj6u52u30Kp6M/trliBMCMKTyK833zpbD+pXdCLuTusPj697FH4R/5mcr" crossorigin="anonymous">

    <!-- Bootstrap Icons -->
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.13.1/font/bootstrap-icons.min.css">

    <style>
        body {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        
        .info-container {
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border: none;
            box-shadow: 0 15px 35px rgba(0, 0, 0, 0.1);
        }
        
        .gradient-text {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
        
        .system-info-content {
            /* Permitir que el contenido fluya naturalmente */
            overflow: visible;
        }
        
        .back-btn {
            transition: all 0.3s ease;
        }
        
        .back-btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
        }
        
        .info-badge {
            font-family: 'Courier New', monospace;
        }
        
        .os-icon {
            font-size: 1.2rem;
        }
        
        /* Prevenir overflow horizontal */
        .info-badge {
            word-wrap: break-word;
            word-break: break-word;
            overflow-wrap: break-word;
            hyphens: auto;
            max-width: 100%;
        }
        
        /* Mejorar responsive en tarjetas */
        .card-body {
            overflow: hidden;
        }
        
        /* Asegurar que las columnas no se desborden */
        .col-xl-6, .col-lg-12, .col-md-6, .col-sm-12,
        .col-xl-4, .col-lg-6 {
            overflow: hidden;
        }
        
        /* Mejorar espaciado en dispositivos pequeños */
        @media (max-width: 576px) {
            .card-body {
                padding: 1rem !important;
            }
            
            .display-6 {
                font-size: 2rem !important;
            }
        }
    </style>
</head>
<body>
    <?php
    /**
     * Función para obtener información del sistema operativo
     * Detecta automáticamente el SO y extrae información relevante
     * 
     * @return array Información del sistema operativo
     */
    function getSystemInfo()
    {
        $os = strtolower(PHP_OS); // Detecta sistema operativo base
        $info = array();

        if (strpos($os, 'linux') !== false) {
            // Obtener datos desde archivos release en sistemas Linux
            $files = glob('/etc/*-release');

            foreach ($files as $file) {
                $lines = array_filter(array_map(function($line) {
                    $parts = explode('=', trim($line));
                    if (count($parts) !== 2) return false;
                    $parts[1] = str_replace(['"', "'"], '', $parts[1]);
                    return $parts;
                }, file($file)));

                foreach ($lines as $line) {
                    $info[$line[0]] = $line[1];
                }
            }

            // También agregar uname
            $info['UNAME'] = php_uname();

        } elseif (strpos($os, 'win') !== false) {
            // Para sistemas Windows
            $info['OS'] = php_uname('s');        // Windows NT
            $info['Host'] = php_uname('n');      // Nombre del host
            $info['Release'] = php_uname('r');   // Versión
            $info['Version'] = php_uname('v');   // Compilación
            $info['Machine'] = php_uname('m');   // Arquitectura

            // Extra: Ejecutar comando systeminfo para más detalles (requiere permisos)
            @exec('systeminfo', $output);
            foreach ($output as $line) {
                if (stripos($line, ':') !== false) {
                    list($key, $val) = explode(':', $line, 2);
                    $info[trim($key)] = trim($val);
                }
            }

        } elseif (strpos($os, 'darwin') !== false) {
            // Para macOS (basado en Darwin)
            $info['OS'] = 'macOS (Darwin)';
            $info['UNAME'] = php_uname();

            // Ejecutar sw_vers para obtener detalles del sistema
            @exec('sw_vers', $output);
            foreach ($output as $line) {
                if (strpos($line, ':') !== false) {
                    list($key, $val) = explode(':', $line, 2);
                    $info[trim($key)] = trim($val);
                }
            }
        } else {
            $info['OS'] = 'Desconocido';
            $info['UNAME'] = php_uname();
        }

        return $info;
    }

    /**
     * Función para determinar el icono del sistema operativo
     * 
     * @param string $os Sistema operativo detectado
     * @return string Clase del icono Bootstrap
     */
    function getOSIcon($os) {
        $os = strtolower($os);
        if (strpos($os, 'win') !== false) {
            return 'bi-windows';
        } elseif (strpos($os, 'linux') !== false) {
            return 'bi-ubuntu';
        } elseif (strpos($os, 'darwin') !== false || strpos($os, 'mac') !== false) {
            return 'bi-apple';
        } else {
            return 'bi-pc-display';
        }
    }

    // Obtener información del sistema
    $systemInfo = getSystemInfo();
    $osIcon = getOSIcon(PHP_OS);
    ?>
    
    <div class="container-fluid py-4">
        <div class="row justify-content-center">
            <div class="col-12">
                <div class="card info-container rounded-4">
                    <div class="card-header bg-transparent border-0 p-4">
                        <div class="d-flex justify-content-between align-items-center">
                            <div>
                                <h1 class="display-6 fw-bold gradient-text mb-2">
                                    <i class="bi bi-pc-display"></i> System Information
                                </h1>
                                <p class="text-muted mb-0">Complete details of the server environment</p>
                            </div>
                            <a href="index.php" class="btn btn-outline-primary back-btn rounded-3">
                                <i class="bi bi-arrow-left me-2"></i>
                                Return to Home
                            </a>
                        </div>
                    </div>
                    
                    <div class="card-body p-4">
                        <div class="system-info-content">
                            <!-- Main Operating System -->
                            <div class="row mb-4">
                                <div class="col-12">
                                    <div class="alert alert-primary d-flex align-items-center" role="alert">
                                        <i class="bi <?php echo $osIcon; ?> os-icon me-3"></i>
                                        <div>
                                            <h5 class="alert-heading mb-1">Detected Operating System</h5>
                                            <strong><?php echo PHP_OS; ?></strong>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            
                            <!-- Detailed Information -->
                            <div class="row">
                                <?php if (!empty($systemInfo)): ?>
                                    <?php foreach ($systemInfo as $key => $value): ?>
                                        <div class="col-xl-6 col-lg-12 col-md-6 col-sm-12 mb-3">
                                            <div class="card h-100 border-0 bg-light">
                                                <div class="card-body p-3">
                                                    <h6 class="card-title text-primary mb-2">
                                                        <i class="bi bi-gear-fill me-2"></i>
                                                        <?php echo htmlspecialchars($key); ?>
                                                    </h6>
                                                    <p class="card-text info-badge text-break">
                                                        <?php echo htmlspecialchars($value); ?>
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    <?php endforeach; ?>
                                <?php else: ?>
                                    <div class="col-12">
                                        <div class="alert alert-warning" role="alert">
                                            <i class="bi bi-exclamation-triangle me-2"></i>
                                            Could not obtain detailed system information.
                                        </div>
                                    </div>
                                <?php endif; ?>
                            </div>
                            
                            <!-- Additional PHP Information -->
                            <div class="row mt-4">
                                <div class="col-12">
                                    <h5 class="mb-3">
                                        <i class="bi bi-code-slash me-2"></i>
                                        Additional PHP Information
                                    </h5>
                                </div>
                                <div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 mb-3">
                                    <div class="card border-0 bg-info text-white">
                                        <div class="card-body text-center">
                                            <i class="bi bi-speedometer2 display-6 mb-2"></i>
                                            <h6>PHP Version</h6>
                                            <p class="mb-0"><?php echo phpversion(); ?></p>
                                        </div>
                                    </div>
                                </div>
                                <div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 mb-3">
                                    <div class="card border-0 bg-success text-white">
                                        <div class="card-body text-center">
                                            <i class="bi bi-server display-6 mb-2"></i>
                                            <h6>SAPI</h6>
                                            <p class="mb-0"><?php echo php_sapi_name(); ?></p>
                                        </div>
                                    </div>
                                </div>
                                <div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 mb-3">
                                    <div class="card border-0 bg-warning text-dark">
                                        <div class="card-body text-center">
                                            <i class="bi bi-memory display-6 mb-2"></i>
                                            <h6>Memory Limit</h6>
                                            <p class="mb-0"><?php echo ini_get('memory_limit'); ?></p>
                                        </div>
                                    </div>
                                </div>
                                <div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 mb-3">
                                    <div class="card border-0 bg-primary text-white">
                                        <div class="card-body text-center">
                                            <i class="bi bi-clock-history display-6 mb-2"></i>
                                            <h6>Timezone</h6>
                                            <p class="mb-0"><?php echo date_default_timezone_get(); ?></p>
                                        </div>
                                    </div>
                                </div>
                                <div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 mb-3">
                                    <div class="card border-0 bg-secondary text-white">
                                        <div class="card-body text-center">
                                            <i class="bi bi-calendar-date display-6 mb-2"></i>
                                            <h6>Server Date</h6>
                                            <p class="mb-0"><?php echo date('d/m/Y'); ?></p>
                                        </div>
                                    </div>
                                </div>
                                <div class="col-xl-4 col-lg-6 col-md-6 col-sm-12 mb-3">
                                    <div class="card border-0 bg-dark text-white">
                                        <div class="card-body text-center">
                                            <i class="bi bi-stopwatch display-6 mb-2"></i>
                                            <h6>Execution Time</h6>
                                            <p class="mb-0"><?php echo ini_get('max_execution_time'); ?>s</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    <div class="card-footer bg-transparent border-0 p-4">
                        <div class="text-center">
                            <small class="text-muted">
                                <i class="bi bi-clock me-1"></i>
                                Generated on <?php echo date('d/m/Y H:i:s'); ?>
                            </small>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Bootstrap 5.3.7 JavaScript Bundle -->
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.7/dist/js/bootstrap.bundle.min.js" integrity="sha384-ndDqU0Gzau9qJ1lfW4pNLlhNTkCfHzAVBReH9diLvGRem5+R9g2FzA8ZGN954O5Q" crossorigin="anonymous"></script>
</body>
</html>