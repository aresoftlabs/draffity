//! `WhisperServer`: gestiona el binario `whisper-server` (modelo caliente) y
//! transcribe vía HTTP `POST /inference`. Ver spec §5.4.

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::services::DraffityHome;

use crate::services::asr::Transcript;

/// Respuesta JSON de `/inference` con `response_format=json`. whisper-server
/// devuelve al menos `{ "text": "..." }`.
#[derive(serde::Deserialize)]
struct InferenceResponse {
    text: String,
}

/// Parsea la respuesta de `/inference` a `Transcript` (texto recortado).
pub fn parse_inference_response(body: &str) -> Option<Transcript> {
    let r: InferenceResponse = serde_json::from_str(body).ok()?;
    Some(Transcript::plain(r.text.trim().to_string()))
}

/// Argumentos para spawnear `whisper-server` (sin el path del binario).
pub fn build_server_args(model: &Path, vad_model: &Path, port: u16) -> Vec<String> {
    vec![
        "--model".into(),
        model.to_string_lossy().into_owned(),
        "--vad".into(),
        "--vad-model".into(),
        vad_model.to_string_lossy().into_owned(),
        "-l".into(),
        "auto".into(),
        "--host".into(),
        "127.0.0.1".into(),
        "--port".into(),
        port.to_string(),
    ]
}

use crate::error::{AppError, AppResult};

/// POST del WAV a `<base>/inference` y parseo. Separado del proceso para testear contra un stub.
pub fn transcribe_at(base_url: &str, wav: &[u8], language: Option<&str>) -> AppResult<Transcript> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| AppError::Unexpected(format!("cliente http: {e}")))?;
    let part = reqwest::blocking::multipart::Part::bytes(wav.to_vec())
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| AppError::Unexpected(format!("multipart: {e}")))?;
    let mut form = reqwest::blocking::multipart::Form::new()
        .part("file", part)
        .text("response_format", "json");
    if let Some(l) = language {
        if l != "auto" {
            form = form.text("language", l.to_string());
        }
    }
    let resp = client
        .post(format!("{base_url}/inference"))
        .multipart(form)
        .send()
        .map_err(|e| AppError::Unexpected(format!("post /inference: {e}")))?;
    let body = resp
        .text()
        .map_err(|e| AppError::Unexpected(format!("lectura respuesta: {e}")))?;
    parse_inference_response(&body)
        .ok_or_else(|| AppError::Unexpected("respuesta /inference inválida".into()))
}

/// Poll hasta que el server responda en `port`.
pub fn wait_ready(port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let client = match reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(400))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    while Instant::now() < deadline {
        if client
            .get(format!("http://127.0.0.1:{port}/"))
            .send()
            .is_ok()
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    false
}

/// Elige un puerto efímero libre.
pub fn pick_port() -> Option<u16> {
    std::net::TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|l| l.local_addr().ok())
        .map(|a| a.port())
}

/// Proceso `whisper-server` vivo, con su puerto.
pub struct WhisperServer {
    child: Child,
    port: u16,
}

impl WhisperServer {
    pub fn start(bin: &Path, model: &Path, vad_model: &Path) -> AppResult<Self> {
        if !bin.exists() {
            return Err(AppError::Unsupported("whisper-server no instalado".into()));
        }
        let port = pick_port().ok_or_else(|| AppError::Unexpected("sin puerto libre".into()))?;
        let mut cmd = Command::new(bin);
        super::proc::no_window(&mut cmd);
        let child = cmd
            .args(build_server_args(model, vad_model, port))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| AppError::Unexpected(format!("spawn whisper-server: {e}")))?;
        let mut server = WhisperServer { child, port };
        if !wait_ready(port, Duration::from_secs(30)) {
            let _ = server.child.kill();
            return Err(AppError::Unexpected("whisper-server no quedó listo".into()));
        }
        Ok(server)
    }

    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn transcribe(&self, wav: &[u8], language: Option<&str>) -> AppResult<Transcript> {
        transcribe_at(&format!("http://127.0.0.1:{}", self.port), wav, language)
    }

    pub fn shutdown(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for WhisperServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Arranca/reusa el `whisper-server` de forma perezosa y lo apaga al salir.
pub struct WhisperServerManager {
    home: DraffityHome,
    inner: Mutex<Option<WhisperServer>>,
}

impl WhisperServerManager {
    pub fn new(home: &DraffityHome) -> Self {
        Self {
            home: DraffityHome::with_root(home.root().to_path_buf()),
            inner: Mutex::new(None),
        }
    }

    /// Transcribe con el server caliente. Lo arranca si hace falta y lo
    /// reinicia si el proceso murió. `Err` si no hay binario/modelo (el caller
    /// cae al CLI).
    pub fn transcribe(
        &self,
        model: &Path,
        wav: &[u8],
        language: Option<&str>,
    ) -> AppResult<Transcript> {
        let bin = self.home.whisper_server_bin_path();
        let vad = self.home.vad_model_path();
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| AppError::Unexpected("lock server".into()))?;
        if let Some(s) = guard.as_mut() {
            if !s.is_alive() {
                *guard = None;
            }
        }
        if guard.is_none() {
            *guard = Some(WhisperServer::start(&bin, model, &vad)?);
        }
        guard.as_ref().unwrap().transcribe(wav, language)
    }

    pub fn shutdown(&self) {
        if let Ok(mut g) = self.inner.lock() {
            if let Some(mut s) = g.take() {
                s.shutdown();
            }
        }
    }

    /// ¿Hay binario de server + VAD instalados? (decidir server vs CLI sin spawnear).
    pub fn available(&self) -> bool {
        self.home.whisper_server_bin_path().exists() && self.home.vad_model_path().exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inference_json_text() {
        let json = r#"{"text":"  hola mundo  "}"#;
        let t = parse_inference_response(json).unwrap();
        assert_eq!(t.text, "hola mundo");
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse_inference_response("not json").is_none());
    }

    #[test]
    fn builds_server_args_with_model_vad_and_port() {
        let args = build_server_args(
            std::path::Path::new("/m/model.bin"),
            std::path::Path::new("/m/vad.bin"),
            54123,
        );
        let joined = args.join(" ");
        assert!(joined.contains("--model /m/model.bin"));
        assert!(joined.contains("--vad"));
        assert!(joined.contains("--vad-model /m/vad.bin"));
        assert!(joined.contains("--host 127.0.0.1"));
        assert!(joined.contains("--port 54123"));
    }

    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn spawn_stub() -> (u16, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let h = std::thread::spawn(move || {
            for stream in listener.incoming().take(2) {
                let mut s = stream.unwrap();
                let mut buf = [0u8; 1024];
                let _ = s.read(&mut buf);
                let body = r#"{"text":"hola desde el stub"}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = s.write_all(resp.as_bytes());
            }
        });
        (port, h)
    }

    #[test]
    fn transcribe_at_posts_and_parses() {
        let (port, _h) = spawn_stub();
        let base = format!("http://127.0.0.1:{port}");
        let wav = vec![0u8; 64];
        let t = transcribe_at(&base, &wav, None).expect("transcribe ok");
        assert_eq!(t.text, "hola desde el stub");
    }

    #[test]
    fn wait_ready_succeeds_against_live_port() {
        let (port, _h) = spawn_stub();
        assert!(wait_ready(port, std::time::Duration::from_secs(2)));
    }
}
