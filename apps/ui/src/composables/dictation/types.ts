import type { useVoiceRecorder } from '@/audio/useVoiceRecorder';

export type DictationPhase = 'idle' | 'recording' | 'transcribing';

export interface DictationOptions {
  /** Mic denegado, fallo de ASR, etc. — el host lo muestra. */
  onError?: (message: string) => void;
  /** El texto ya está en el portapapeles (fallback de inserción). */
  onClipboardFallback?: (text: string) => void;
  /** Confirmación de descarte para grabaciones largas. Default: confirma siempre. */
  confirmDiscard?: () => boolean;
}

/** Costura de inserción compartida por todos los modos (desacoplada del modo). */
export interface EditorDictationBuffer {
  /** Marca la posición actual del cursor como ancla de inserción. */
  beginPending(): void;
  /** Inserta texto confirmado en el ancla; false si no hay ancla. */
  commit(text: string): boolean;
  /** Descarta el ancla/marcador sin insertar. */
  clearPending(): void;
  /** Streaming: muestra/actualiza el fantasma gris en el cursor. */
  setGhost(text: string): void;
  /** Streaming: limpia el fantasma. */
  clearGhost(): void;
  /** Streaming: inserta texto confirmado en el cursor (avanza el cursor). */
  commitStreaming(text: string): boolean;
}

/** Handle del grabador tal como lo expone `useVoiceRecorder`. */
export type VoiceRecorderHandle = ReturnType<typeof useVoiceRecorder>;

/** Toolkit compartido que un modo necesita; ningún modo toca globals. */
export interface DictationContext {
  editor: EditorDictationBuffer;
  recorder: VoiceRecorderHandle;
  options: DictationOptions;
  setPhase(phase: DictationPhase): void;
  setProgress(value: number | null): void;
  /** Registra y reporta un error (setea `error` + llama `onError`). */
  fail(e: unknown): void;
  /** Copia al portapapeles y llama `onClipboardFallback`. */
  clipboardFallback(text: string): Promise<void>;
}

export interface DictationMode {
  readonly id: 'manual' | 'streaming';
  start(ctx: DictationContext): Promise<void>;
  stop(ctx: DictationContext): Promise<void>;
  cancel(ctx: DictationContext): void;
}
