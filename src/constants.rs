pub const UNIFIED_BIND_GROUP_LAYOUT_ENTRIES: &[wgpu::BindGroupLayoutEntry] = &[
    // Component uniform (now includes frosted glass parameters)
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    },
    // Texture
    wgpu::BindGroupLayoutEntry {
        binding: 1,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    },
    // Sampler
    wgpu::BindGroupLayoutEntry {
        binding: 2,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    },
];
pub const CREDENTIAL_SERVICE_NAME: &str = "Frostify";
pub const CREDENTIAL_USER_NAME: &str = "Frostify_user";
pub const WINDOW_RESIZE_BORDER_WIDTH: f64 = 2.0;
pub const BACKGROUND_FPS: u32 = 10;
pub const SPOTIFY_CLIENT_ID: &str = "f6f1788623fa400ebab54272bb3f515c";
pub const SPOTIFY_REDIRECT_URI: &str = "http://127.0.0.1:8888/callback";
pub const SPOTIFY_ACCESS_SCOPES: &str = "streaming,user-read-email,user-read-private,playlist-read-private,playlist-read-collaborative,playlist-modify-public,playlist-modify-private,user-follow-modify,user-follow-read,user-library-read,user-library-modify,user-top-read,user-read-recently-played";
pub const FROSTIFY_LOGIN_SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Frostify Login Success</title>
    <style>
        body {
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            height: 100vh;
            text-align: center;
            font-family: Arial, sans-serif;
            background-color: #121212;
            color: white;
        }
        img {
            width: 150px;
            height: auto;
            margin-bottom: 20px;
        }
        .message {
            font-size: 20px;
        }
        .countdown {
            margin-top: 10px;
            font-size: 16px;
            color: #aaa;
        }
    </style>
    <script>
        // Set countdown timer to close window
        let timeLeft = 5;
        window.onload = function() {
            const countdownElement = document.getElementById('countdown');
            
            // Update countdown every second
            const timer = setInterval(function() {
                timeLeft--;
                countdownElement.textContent = `This window will close in ${timeLeft} seconds...`;
                
                if (timeLeft <= 0) {
                    clearInterval(timer);
                    window.close();
                }
            }, 1000);
        }
    </script>
</head>
<body>
    <img src="data:image/png;base64,LOGO_BASE64" alt="Frostify Logo">
    <div class="message">Authentication successful!</div>
    <div class="countdown" id="countdown">This window will close in 5 seconds...</div>
</body>
</html>"#;

pub const FROSTIFY_LOGIN_ERROR_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Frostify Login Error</title>
    <style>
        body {
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            height: 100vh;
            text-align: center;
            font-family: Arial, sans-serif;
            background-color: #121212;
            color: white;
        }
        img {
            width: 150px;
            height: auto;
            margin-bottom: 20px;
        }
        .message {
            font-size: 20px;
        }
    </style>
</head>
<body>
    <img src="data:image/png;base64,LOGO_BASE64" alt="Frostify Logo">
    <div class="message">An error occurred during the login process. Please try again.</div>
</body>
</html>"#;
