//! Spawns de procesos hijo sin abrir una ventana de consola en Windows.
//!
//! En un build GUI (`windows_subsystem = "windows"`) no hay consola adjunta, así
//! que cada proceso de consola (whisper, piper) abre una ventana cmd que parpadea.
//! `CREATE_NO_WINDOW` lo evita. En dev (corriendo desde una consola) no se nota,
//! pero el flag es inocuo ahí también. No-op fuera de Windows.

use std::process::Command;

/// Aplica `CREATE_NO_WINDOW` en Windows; identidad en el resto.
#[cfg(windows)]
pub fn no_window(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW)
}

/// No-op fuera de Windows.
#[cfg(not(windows))]
pub fn no_window(cmd: &mut Command) -> &mut Command {
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_a_chainable_command_handle() {
        // Contrato mínimo válido en todas las plataformas: devuelve el &mut Command
        // para encadenar. En Windows además setea el flag (no observable por API).
        let mut cmd = Command::new("echo");
        no_window(&mut cmd).arg("ok");
    }
}
