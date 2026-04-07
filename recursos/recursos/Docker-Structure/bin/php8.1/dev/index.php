<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>PHP Development Environment - Navigation</title>
    
    <!-- Bootstrap 5.3.7 CSS -->
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.7/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-LN+7fdVzj6u52u30Kp6M/trliBMCMKTyK833zpbD+pXdCLuTusPj697FH4R/5mcr" crossorigin="anonymous">
    
    <!-- Bootstrap Icons -->
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.13.1/font/bootstrap-icons.min.css">
    
    <style>
        body {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        
        .main-container {
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        
        .nav-card {
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border: none;
            box-shadow: 0 15px 35px rgba(0, 0, 0, 0.1);
        }
        
        .nav-btn {
            transition: all 0.3s ease;
            font-weight: 600;
            border: none;
            padding: 1rem 2rem;
        }
        
        .nav-btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
        }
        
        .php-version-badge {
            font-family: 'Courier New', monospace;
            background: linear-gradient(135deg, #4299e1 0%, #3182ce 100%);
        }
        
        .gradient-text {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }
    </style>
</head>
<body>
    <div class="main-container">
        <div class="container">
            <div class="row justify-content-center">
                <div class="col-lg-6 col-md-8 col-sm-10">
                    <div class="card nav-card rounded-4">
                        <div class="card-body p-5">
                            <!-- Header -->
                            <div class="text-center mb-4">
                                <h1 class="display-5 fw-bold gradient-text mb-3">
                                    <i class="bi bi-code-slash"></i> PHP Development Environment
                                </h1>
                                <p class="lead text-muted">Select an option to continue</p>
                            </div>
                            
                            <!-- Navigation Buttons -->
                            <div class="d-grid gap-3 mb-4">
                                <a href="info.php" class="btn btn-success btn-lg nav-btn rounded-3">
                                    <i class="bi bi-info-circle-fill me-2"></i>
                                    View PHP Information
                                </a>
                                <a href="os.php" class="btn btn-warning btn-lg nav-btn rounded-3">
                                    <i class="bi bi-pc-display me-2"></i>
                                    View System Information
                                </a>
                            </div>
                            
                            <!-- Footer -->
                            <div class="border-top pt-4">
                                <div class="text-center">
                                    <p class="text-muted mb-3">PHP development environment</p>
                                    <span class="badge php-version-badge text-white px-3 py-2 rounded-pill">
                                        <i class="bi bi-gear-fill me-1"></i>
                                        PHP Version: <?php echo phpversion(); ?>
                                    </span>
                                </div>
                            </div>
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