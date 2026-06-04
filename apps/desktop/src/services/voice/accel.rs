//! Detección del backend de aceleración para whisper.cpp.
//! Metal en Apple Silicon; Vulkan si hay GPU en Windows/Linux; si no, CPU.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Metal,
    Vulkan,
    Cpu,
}

impl Backend {
    pub fn as_str(self) -> &'static str {
        match self {
            Backend::Metal => "metal",
            Backend::Vulkan => "vulkan",
            Backend::Cpu => "cpu",
        }
    }
}

/// Decisión pura (testeable) dada plataforma + disponibilidad de Vulkan.
/// `os`/`arch` usan los valores de `std::env::consts`.
pub fn decide_backend(os: &str, arch: &str, vulkan_available: bool) -> Backend {
    match (os, arch) {
        ("macos", "aarch64") => Backend::Metal,
        ("windows" | "linux", _) if vulkan_available => Backend::Vulkan,
        _ => Backend::Cpu,
    }
}

/// Heurística de disponibilidad de Vulkan: el loader del sistema presente.
/// Deliberadamente laxa — si el binario Vulkan no inicializa GPU, el arranque
/// del server (Fase 2) cae a CPU. En macOS no aplica (usa Metal).
fn vulkan_loader_present() -> bool {
    #[cfg(target_os = "windows")]
    {
        let sys = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        std::path::Path::new(&sys)
            .join("System32")
            .join("vulkan-1.dll")
            .exists()
    }
    #[cfg(target_os = "linux")]
    {
        [
            "/usr/lib",
            "/usr/lib/x86_64-linux-gnu",
            "/lib/x86_64-linux-gnu",
        ]
        .iter()
        .any(|d| std::path::Path::new(d).join("libvulkan.so.1").exists())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

/// Elige el id de modelo whisper según el backend, respetando un override del
/// usuario (si está instalado). GPU favorece el turbo; CPU favorece `small`
/// por latencia. Cae a cualquiera instalado, o `None`.
pub fn pick_model(
    backend: Backend,
    installed: &[&str],
    user_override: Option<&str>,
) -> Option<String> {
    if let Some(ov) = user_override {
        if !ov.is_empty() && installed.contains(&ov) {
            return Some(ov.to_string());
        }
    }
    let pref: &[&str] = match backend {
        Backend::Metal | Backend::Vulkan => &["large-v3-turbo-q5_0", "small", "base", "tiny"],
        Backend::Cpu => &["small", "base", "tiny", "large-v3-turbo-q5_0"],
    };
    for id in pref {
        if installed.contains(id) {
            return Some((*id).to_string());
        }
    }
    installed.first().map(|s| s.to_string())
}

/// Backend para esta máquina (consts de compilación + probe Vulkan en runtime).
pub fn detect_backend() -> Backend {
    decide_backend(
        std::env::consts::OS,
        std::env::consts::ARCH,
        vulkan_loader_present(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decides_metal_on_apple_silicon() {
        assert_eq!(decide_backend("macos", "aarch64", false), Backend::Metal);
        assert_eq!(decide_backend("macos", "x86_64", false), Backend::Cpu);
    }

    #[test]
    fn decides_vulkan_when_gpu_present_on_win_linux() {
        assert_eq!(decide_backend("windows", "x86_64", true), Backend::Vulkan);
        assert_eq!(decide_backend("linux", "x86_64", true), Backend::Vulkan);
    }

    #[test]
    fn falls_back_to_cpu_without_gpu() {
        assert_eq!(decide_backend("windows", "x86_64", false), Backend::Cpu);
        assert_eq!(decide_backend("linux", "x86_64", false), Backend::Cpu);
    }

    #[test]
    fn backend_as_str_is_stable() {
        assert_eq!(Backend::Metal.as_str(), "metal");
        assert_eq!(Backend::Vulkan.as_str(), "vulkan");
        assert_eq!(Backend::Cpu.as_str(), "cpu");
    }

    #[test]
    fn gpu_prefers_turbo_cpu_prefers_small() {
        let installed = ["tiny", "base", "small", "large-v3-turbo-q5_0"];
        assert_eq!(
            pick_model(Backend::Metal, &installed, None).as_deref(),
            Some("large-v3-turbo-q5_0")
        );
        assert_eq!(
            pick_model(Backend::Vulkan, &installed, None).as_deref(),
            Some("large-v3-turbo-q5_0")
        );
        assert_eq!(
            pick_model(Backend::Cpu, &installed, None).as_deref(),
            Some("small")
        );
    }

    #[test]
    fn user_override_wins_when_installed() {
        let installed = ["small", "large-v3-turbo-q5_0"];
        assert_eq!(
            pick_model(Backend::Cpu, &installed, Some("large-v3-turbo-q5_0")).as_deref(),
            Some("large-v3-turbo-q5_0")
        );
        assert_eq!(
            pick_model(Backend::Cpu, &installed, Some("tiny")).as_deref(),
            Some("small")
        );
    }

    #[test]
    fn falls_back_to_any_installed_then_none() {
        assert_eq!(
            pick_model(Backend::Cpu, &["tiny"], None).as_deref(),
            Some("tiny")
        );
        assert_eq!(pick_model(Backend::Cpu, &[], None), None);
    }
}
