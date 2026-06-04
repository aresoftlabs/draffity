//! `WhisperServer`: gestiona el binario `whisper-server` (modelo caliente) y
//! transcribe vía HTTP `POST /inference`. Ver spec §5.4.

use std::path::Path;

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
}
