//! Curated prompts for the inline AI actions (F-08..F-11). Pure builders, so
//! they're unit-tested without the network. The command layer parses the
//! request into an `AiAction`, asks `ProjectMemoryService` for context, then
//! calls `build_messages` to get the chat payload.

use crate::error::{AppError, AppResult};
use crate::services::ai::ChatMessage;

/// The six Rewrite sub-modes (F-10). `Custom` carries the user's instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RewriteMode {
    Rephrase,
    Shorter,
    Vivid,
    ShowNotTell,
    InnerConflict,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiAction {
    Continue,
    Expand,
    Describe,
    Rewrite(RewriteMode),
}

/// The text the action operates on, plus the rendered memory block (may be
/// empty). Borrowed — built per request.
pub struct ActionInput<'a> {
    pub selected_text: &'a str,
    pub preceding_text: &'a str,
    pub memory_block: &'a str,
}

/// Persona shared by every action: return only the requested text, preserve
/// the manuscript's language/voice/POV/tense.
const BASE_SYSTEM: &str = "Eres un asistente de escritura literaria integrado en un editor de texto. \
Devuelves ÚNICAMENTE el texto pedido: sin explicaciones, sin preámbulos y sin comillas que lo envuelvan. \
Conservás el idioma, la voz narrativa, el punto de vista y el tiempo verbal del texto del usuario.";

/// Parse the wire request into a typed action. `sub_mode` and `custom` only
/// matter for Rewrite.
pub fn parse_action(
    action: &str,
    sub_mode: Option<&str>,
    custom: Option<&str>,
) -> AppResult<AiAction> {
    match action {
        "continue" => Ok(AiAction::Continue),
        "expand" => Ok(AiAction::Expand),
        "describe" => Ok(AiAction::Describe),
        "rewrite" => {
            let mode = match sub_mode.unwrap_or("rephrase") {
                "rephrase" => RewriteMode::Rephrase,
                "shorter" => RewriteMode::Shorter,
                "vivid" => RewriteMode::Vivid,
                "show_not_tell" => RewriteMode::ShowNotTell,
                "inner_conflict" => RewriteMode::InnerConflict,
                "custom" => {
                    let prompt = custom.map(str::trim).filter(|s| !s.is_empty());
                    match prompt {
                        Some(p) => RewriteMode::Custom(p.to_string()),
                        None => {
                            return Err(AppError::Invariant(
                                "rewrite 'custom' requiere una instrucción".into(),
                            ))
                        }
                    }
                }
                other => {
                    return Err(AppError::Invariant(format!(
                        "sub-modo de rewrite desconocido: {other}"
                    )))
                }
            };
            Ok(AiAction::Rewrite(mode))
        }
        other => Err(AppError::Invariant(format!(
            "acción de IA desconocida: {other}"
        ))),
    }
}

/// Per-action instruction appended to the system persona.
fn instruction(action: &AiAction) -> String {
    match action {
        AiAction::Continue => "Tarea: continuá la narración de forma natural a partir del texto entregado. \
No repitas lo ya escrito; escribí lo que sigue."
            .to_string(),
        AiAction::Expand => "Tarea: expandí el fragmento entregado, ampliándolo aproximadamente entre un 30% y un 50%, \
agregando detalle y profundidad sin cambiar los hechos."
            .to_string(),
        AiAction::Describe => "Tarea: escribí una descripción sensorial y evocadora de lo indicado, apelando a vista, \
oído, olfato, tacto y atmósfera cuando sea pertinente."
            .to_string(),
        AiAction::Rewrite(mode) => match mode {
            RewriteMode::Rephrase => "Tarea: reescribí el fragmento con otras palabras, conservando el sentido y una longitud similar.".to_string(),
            RewriteMode::Shorter => "Tarea: reescribí el fragmento de forma más concisa, conservando lo esencial.".to_string(),
            RewriteMode::Vivid => "Tarea: reescribí el fragmento con lenguaje más vívido, concreto y sensorial.".to_string(),
            RewriteMode::ShowNotTell => "Tarea: reescribí mostrando mediante acciones, gestos y sensaciones en lugar de \
enunciar emociones o hechos ('show, don't tell').".to_string(),
            RewriteMode::InnerConflict => "Tarea: reescribí profundizando el conflicto interno, las dudas y la introspección \
del personaje punto de vista.".to_string(),
            RewriteMode::Custom(p) => format!("Tarea: reescribí el fragmento siguiendo esta instrucción del usuario: {p}"),
        },
    }
}

/// Build the chat messages for an action. Validates that the required text is
/// present. The memory block, when non-empty, rides in the system message.
pub fn build_messages(action: &AiAction, input: &ActionInput) -> AppResult<Vec<ChatMessage>> {
    let user_text = match action {
        AiAction::Continue => input.preceding_text.trim(),
        _ => input.selected_text.trim(),
    };
    if user_text.is_empty() {
        let need = match action {
            AiAction::Continue => "texto previo al cursor",
            _ => "una selección de texto",
        };
        return Err(AppError::Invariant(format!("esta acción necesita {need}")));
    }

    let mut system = format!("{BASE_SYSTEM}\n\n{}", instruction(action));
    if !input.memory_block.trim().is_empty() {
        system.push_str("\n\nContexto del proyecto (úsalo para mantener coherencia, no lo cites literalmente):\n");
        system.push_str(input.memory_block.trim());
    }

    Ok(vec![
        ChatMessage::system(system),
        ChatMessage::user(user_text),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ai::Role;

    #[test]
    fn parses_all_actions_and_rewrite_modes() {
        assert_eq!(
            parse_action("continue", None, None).unwrap(),
            AiAction::Continue
        );
        assert_eq!(
            parse_action("expand", None, None).unwrap(),
            AiAction::Expand
        );
        assert_eq!(
            parse_action("describe", None, None).unwrap(),
            AiAction::Describe
        );
        assert_eq!(
            parse_action("rewrite", Some("vivid"), None).unwrap(),
            AiAction::Rewrite(RewriteMode::Vivid)
        );
        // Default rewrite mode is rephrase.
        assert_eq!(
            parse_action("rewrite", None, None).unwrap(),
            AiAction::Rewrite(RewriteMode::Rephrase)
        );
    }

    #[test]
    fn custom_rewrite_requires_instruction() {
        assert!(parse_action("rewrite", Some("custom"), None).is_err());
        assert!(parse_action("rewrite", Some("custom"), Some("  ")).is_err());
        assert_eq!(
            parse_action("rewrite", Some("custom"), Some("más oscuro")).unwrap(),
            AiAction::Rewrite(RewriteMode::Custom("más oscuro".into()))
        );
    }

    #[test]
    fn unknown_action_and_submode_rejected() {
        assert!(parse_action("translate", None, None).is_err());
        assert!(parse_action("rewrite", Some("funkify"), None).is_err());
    }

    #[test]
    fn continue_uses_preceding_text_as_user_message() {
        let msgs = build_messages(
            &AiAction::Continue,
            &ActionInput {
                selected_text: "",
                preceding_text: "Era una noche oscura.",
                memory_block: "",
            },
        )
        .unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, Role::System);
        assert!(msgs[0].content.contains("continuá la narración"));
        assert_eq!(msgs[1].role, Role::User);
        assert_eq!(msgs[1].content, "Era una noche oscura.");
    }

    #[test]
    fn selection_actions_require_selected_text() {
        let empty = ActionInput {
            selected_text: "   ",
            preceding_text: "x",
            memory_block: "",
        };
        assert!(build_messages(&AiAction::Expand, &empty).is_err());
        assert!(build_messages(&AiAction::Describe, &empty).is_err());
        assert!(build_messages(&AiAction::Rewrite(RewriteMode::Shorter), &empty).is_err());
    }

    #[test]
    fn memory_block_is_embedded_in_system_when_present() {
        let msgs = build_messages(
            &AiAction::Rewrite(RewriteMode::Vivid),
            &ActionInput {
                selected_text: "Caminó.",
                preceding_text: "",
                memory_block: "== Memoria del proyecto ==\nAragorn (character): montaraz.",
            },
        )
        .unwrap();
        assert!(msgs[0].content.contains("Contexto del proyecto"));
        assert!(msgs[0].content.contains("Aragorn"));
    }

    #[test]
    fn no_memory_section_when_block_empty() {
        let msgs = build_messages(
            &AiAction::Expand,
            &ActionInput {
                selected_text: "Algo.",
                preceding_text: "",
                memory_block: "   ",
            },
        )
        .unwrap();
        assert!(!msgs[0].content.contains("Contexto del proyecto"));
    }
}
