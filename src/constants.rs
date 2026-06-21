pub const CREDENTIAL_SERVICE_NAME: &str = "Opal";
pub const CREDENTIAL_USER_NAME: &str = "Opal_user";
pub const SPOTIFY_REDIRECT_URI: &str = "http://127.0.0.1:8888/callback";
pub const SPOTIFY_ACCESS_SCOPES: &str = "streaming,user-read-email,user-read-private,playlist-read-private,playlist-read-collaborative,playlist-modify-public,playlist-modify-private,user-follow-modify,user-follow-read,user-library-read,user-library-modify,user-top-read,user-read-recently-played,user-read-playback-state,user-read-currently-playing,user-modify-playback-state";

pub const LOGIN_OK_HTML: &str = r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><title>Opal</title><style>body{display:flex;flex-direction:column;justify-content:center;align-items:center;height:100vh;text-align:center;font-family:Arial,sans-serif;background:#121212;color:#fff}.m{font-size:20px}.c{margin-top:10px;font-size:14px;color:#aaa}</style><script>let t=5;onload=()=>{let e=document.getElementById('c');let i=setInterval(()=>{t--;e.textContent=`Closing in ${t}s...`;if(t<=0){clearInterval(i);window.close();}},1000);}</script></head><body><div class="m">Authentication successful!</div><div class="c" id="c">Closing in 5s...</div></body></html>"#;

pub const LOGIN_ERR_HTML: &str = r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><title>Opal</title><style>body{display:flex;justify-content:center;align-items:center;height:100vh;font-family:Arial;background:#121212;color:#fff}</style></head><body><div>Login error. Try again.</div></body></html>"#;
