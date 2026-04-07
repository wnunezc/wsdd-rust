<?php
date_default_timezone_set('UTC');
?>
<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PHP Information - Development Environment</title>
    
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
        
        .phpinfo-content {
            max-height: 70vh;
            overflow-y: auto;
            border: 1px solid #dee2e6;
            border-radius: 0.5rem;
            background: #f8f9fa;
        }
        
        .phpinfo-content iframe {
            width: 100%;
            height: 100%;
            border: none;
            min-height: 600px;
        }
        
        .back-btn {
            transition: all 0.3s ease;
        }
        
        .back-btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
        }
        
        /* Forzar alineación a la izquierda en todas las tablas de phpinfo */
        .phpinfo-content table td,
        .phpinfo-content table th {
            text-align: left !important;
            vertical-align: top;
            padding: 0.75rem;
        }
        
        /* Mantener solo los headers principales centrados */
        .phpinfo-content table th.text-center {
            text-align: center !important;
        }
        
        /* Mejorar legibilidad de valores largos */
        .phpinfo-content table td.text-break {
            word-break: break-word;
            white-space: pre-wrap;
        }
    </style>
</head>
<body>
    <div class="container-fluid py-4">
        <div class="row justify-content-center">
            <div class="col-12">
                <div class="card info-container rounded-4">
                    <div class="card-header bg-transparent border-0 p-4">
                        <div class="d-flex justify-content-between align-items-center">
                            <div>
                                <h1 class="display-6 fw-bold gradient-text mb-2">
                                    <i class="bi bi-info-circle-fill"></i> PHP Information
                                </h1>
                                <p class="text-muted mb-0">Complete configuration of the PHP environment</p>
                            </div>
                            <a href="index.php" class="btn btn-outline-primary back-btn rounded-3">
                                <i class="bi bi-arrow-left me-2"></i>
                                Return to Home
                            </a>
                        </div>
                    </div>
                    
                    <div class="card-body p-4">
                        <div class="phpinfo-content">
                            <?php
                            // Capturar la salida de phpinfo() y mostrarla
                            ob_start();
                            phpinfo();
                            $phpinfo = ob_get_contents();
                            ob_end_clean();
                            
                            // Limpiar el HTML de phpinfo para que se vea mejor con Bootstrap
                            $phpinfo = preg_replace('%^.*<body>(.*)</body>.*$%ms', '$1', $phpinfo);
                            
                            // Aplicar clases de Bootstrap a las tablas
                            $phpinfo = str_replace('<table', '<table class="table table-striped table-hover table-bordered"', $phpinfo);
                            
                            // Mantener centrado solo para headers principales de sección
                            $phpinfo = str_replace('class="center"', 'class="text-center table-primary"', $phpinfo);
                            
                            // Mejorar presentación de valores
                            $phpinfo = str_replace('class="v"', 'class="text-break"', $phpinfo);
                            
                            // Mejorar headers de columnas
                            $phpinfo = str_replace('<th>', '<th class="table-secondary">', $phpinfo);
                            
                            echo $phpinfo;
                            ?>
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
    
    <!-- Bootstrap 5.3.6 JavaScript Bundle -->
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.7/dist/js/bootstrap.bundle.min.js" integrity="sha384-ndDqU0Gzau9qJ1lfW4pNLlhNTkCfHzAVBReH9diLvGRem5+R9g2FzA8ZGN954O5Q" crossorigin="anonymous"></script>
</body>
</html>