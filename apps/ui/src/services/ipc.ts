import { invoke } from '@tauri-apps/api/core';
import type {
  BackupRecord,
  BibliographyImportSummary,
  Citation,
  CodexEntry,
  CodexInput,
  CodexKind,
  CodexUpdate,
  DailyWriting,
  DocNode,
  DocumentInput,
  DocumentStatus,
  ExportConfig,
  ExportFormat,
  CustomField,
  CustomFieldInput,
  ImportFormat,
  Label,
  LabelInput,
  MediaAsset,
  Project,
  ProjectInput,
  SearchHit,
  Snapshot,
  Template,
  WritingStats,
} from '@draffity/shared-types';

/** A collection of documents (I-01..I-03). */
export type CollectionKind = 'manual' | 'smart';
export interface CollectionQuery {
  tagsAny?: string[];
  statuses?: DocumentStatus[];
  titleContains?: string | null;
}
export interface Collection {
  id: string;
  projectId: string;
  name: string;
  kind: CollectionKind;
  query?: CollectionQuery | null;
  createdAt: number;
}
export interface CollectionInput {
  projectId: string;
  name: string;
  kind: CollectionKind;
  query?: CollectionQuery | null;
}

/** BYOK AI status reported by the backend. */
export interface AiStatus {
  /** AI usable now: a key is stored. */
  available: boolean;
  /** A key is stored. */
  hasKey: boolean;
}

/** Inline AI action request (F-08..F-11). Mirrors the Rust `AiActionRequest`. */
export interface AiActionRequestInput {
  requestId: string;
  action: 'continue' | 'expand' | 'rewrite' | 'describe';
  subMode?: string;
  projectId: string;
  docId?: string | null;
  selectedText: string;
  precedingText: string;
  customPrompt?: string;
  model?: string;
  maxTokens?: number;
}

export interface AiActionResult {
  requestId: string;
  text: string;
  cancelled: boolean;
  promptTokens: number | null;
  completionTokens: number | null;
}

/** Live delta event payload (`ai.suggestion.received`). */
export interface AiDeltaEvent {
  requestId: string;
  delta: string;
}

/** A single AI validator finding (Ã‰pica G). */
export interface ValidationFinding {
  validator: string;
  severity: 'critical' | 'warning' | 'info';
  title: string;
  detail: string;
  excerpt?: string;
  suggestion?: string;
  /** Stable i18n key for backend-generated findings; AI findings omit it and
   *  carry model text in `title`/`detail` (AUD-20). */
  code?: string;
  params?: Record<string, string>;
}

/** A persisted validation report; `resultsJson` is a `ValidationFinding[]`. */
export interface AiValidation {
  id: string;
  documentId: string;
  validatorName: string;
  resultsJson: string;
  severitySummary: string;
  createdAt: number;
}

/** Codex coverage pre-check (G-03). */
export interface CoverageReport {
  candidates: number;
  covered: number;
}

/** Transcription result (Ã‰pica H). */
export interface TranscriptSegment {
  text: string;
  startMs: number;
  endMs: number;
}
export interface Transcript {
  text: string;
  segments: TranscriptSegment[];
}

/** Voice runtime status (H). */
export interface VoiceStatus {
  dictationAvailable: boolean;
  binaryInstalled: boolean;
  installedModels: string[];
  ttsAvailable: boolean;
  piperInstalled: boolean;
}

/** A downloadable Piper voice (H-05). */
export interface VoiceVoice {
  id: string;
  name: string;
  lang: string;
  sizeMb: number;
  recommended: boolean;
  installed: boolean;
}

/** Synthesized speech (H-06): mono PCM16 + sample rate for Web Audio. */
export interface SynthesizedAudio {
  samplesPcm16: number[];
  sampleRate: number;
}

/** A downloadable Whisper model (H-02). */
export interface VoiceModel {
  id: string;
  filename: string;
  sizeMb: number;
  recommended: boolean;
  installed: boolean;
}

/** Model download progress event (`voice.download.progress`). */
export interface VoiceDownloadProgress {
  modelId: string;
  downloaded: number;
  total: number | null;
}

/**
 * Thin typed wrapper over Tauri IPC commands.
 * Keep this file as the single boundary between Vue and Rust.
 */
export const ipc = {
  // System
  ping: () => invoke<string>('ping'),
  getSetting: (key: string) => invoke<string | null>('get_setting', { key }),
  setSetting: (key: string, value: string) => invoke<void>('set_setting', { key, value }),
  getWritingStats: () => invoke<WritingStats>('get_writing_stats'),
  getRecentDailyWriting: (days: number) =>
    invoke<DailyWriting[]>('get_recent_daily_writing', { days }),
  getDailyGoal: () => invoke<number | null>('get_daily_goal'),
  setDailyGoal: (goal: number | null) => invoke<void>('set_daily_goal', { goal }),
  getCrashReportingStatus: () =>
    invoke<{ active: boolean; enabled: boolean }>('get_crash_reporting_status'),
  setCrashReportingEnabled: (enabled: boolean) =>
    invoke<void>('set_crash_reporting_enabled', { enabled }),

  // AI (BYOK)
  getAiStatus: () => invoke<AiStatus>('get_ai_status'),
  setOpenrouterKey: (key: string) => invoke<AiStatus>('set_openrouter_key', { key }),
  clearOpenrouterKey: () => invoke<AiStatus>('clear_openrouter_key'),
  aiRunAction: (req: AiActionRequestInput) => invoke<AiActionResult>('ai_run_action', { req }),
  aiCancel: (requestId: string) => invoke<void>('ai_cancel', { requestId }),
  aiRecordAccepted: (input: {
    projectId: string;
    docId?: string | null;
    action: string;
    model?: string | null;
    response: string;
  }) => invoke<unknown>('ai_record_accepted', { input }),

  // AI validators (Ã‰pica G)
  checkCodexCoverage: (projectId: string, documentId: string) =>
    invoke<CoverageReport>('check_codex_coverage', { projectId, documentId }),
  runValidators: (projectId: string, documentId: string, validators: string[]) =>
    invoke<AiValidation[]>('run_validators', { projectId, documentId, validators }),
  listValidations: (documentId: string) =>
    invoke<AiValidation[]>('list_validations', { documentId }),

  // Voice (Ã‰pica H)
  getVoiceStatus: () => invoke<VoiceStatus>('get_voice_status'),
  listVoiceModels: () => invoke<VoiceModel[]>('list_voice_models'),
  downloadVoiceModel: (modelId: string) => invoke<void>('download_voice_model', { modelId }),
  deleteVoiceModel: (modelId: string) => invoke<void>('delete_voice_model', { modelId }),
  importVoiceBinary: (sourcePath: string) => invoke<void>('import_voice_binary', { sourcePath }),
  transcribeAudio: (wav: Uint8Array) =>
    invoke<Transcript>('transcribe_audio', { wav: Array.from(wav) }),
  listVoiceVoices: () => invoke<VoiceVoice[]>('list_voice_voices'),
  downloadVoiceVoice: (voiceId: string) => invoke<void>('download_voice_voice', { voiceId }),
  importPiperBinary: (sourcePath: string) => invoke<void>('import_piper_binary', { sourcePath }),
  synthesizeSpeech: (text: string, voiceId: string) =>
    invoke<SynthesizedAudio>('synthesize_speech', { text, voiceId }),
  saveVoiceNote: (params: {
    projectId: string;
    wav: Uint8Array;
    durationMs: number;
    transcribe: boolean;
  }) =>
    invoke<MediaAsset>('save_voice_note', {
      projectId: params.projectId,
      wav: Array.from(params.wav),
      durationMs: params.durationMs,
      transcribe: params.transcribe,
    }),
  listVoiceNotes: (projectId: string) => invoke<MediaAsset[]>('list_voice_notes', { projectId }),
  deleteVoiceNote: (id: string) => invoke<void>('delete_voice_note', { id }),

  // Projects
  createProject: (input: ProjectInput) => invoke<Project>('create_project', { input }),
  listProjects: () => invoke<Project[]>('list_projects'),
  getProject: (id: string) => invoke<Project | null>('get_project', { id }),
  getActiveProject: () => invoke<Project | null>('get_active_project'),
  openProject: (id: string) => invoke<Project>('open_project', { id }),
  archiveProject: (id: string) => invoke<void>('archive_project', { id }),
  deleteProject: (id: string) => invoke<void>('delete_project', { id }),
  setProjectGoal: (params: { id: string; goal: number | null }) =>
    invoke<Project>('set_project_goal', params),
  setProjectDeadline: (params: { id: string; deadline: number | null }) =>
    invoke<Project>('set_project_deadline', params),

  // Documents
  createDocument: (input: DocumentInput) => invoke<DocNode>('create_document', { input }),
  listDocuments: (projectId: string) => invoke<DocNode[]>('list_documents', { projectId }),
  getDocument: (id: string) => invoke<DocNode | null>('get_document', { id }),
  updateDocument: (params: {
    id: string;
    title?: string;
    content?: string;
    contentJson?: string;
  }) => invoke<DocNode>('update_document', params),
  moveDocument: (params: { id: string; parentId?: string | null; position: number }) =>
    invoke<void>('move_document', params),
  reorderDocuments: (params: {
    projectId: string;
    parentId: string | null;
    orderedIds: string[];
  }) => invoke<void>('reorder_documents', params),
  setDocumentStatus: (params: { id: string; status: DocumentStatus }) =>
    invoke<DocNode>('set_document_status', params),
  setDocumentTags: (params: { id: string; tags: string[] }) =>
    invoke<DocNode>('set_document_tags', params),
  listProjectTags: (projectId: string) => invoke<string[]>('list_project_tags', { projectId }),
  setDocumentGoal: (params: { id: string; goal: number | null }) =>
    invoke<DocNode>('set_document_goal', params),
  setDocumentSynopsis: (params: { id: string; synopsis: string | null }) =>
    invoke<DocNode>('set_document_synopsis', params),
  deleteDocument: (id: string) => invoke<void>('delete_document', { id }),

  // Snapshots
  createSnapshot: (params: { documentId: string; label?: string }) =>
    invoke<Snapshot>('create_snapshot', params),
  listSnapshots: (documentId: string) => invoke<Snapshot[]>('list_snapshots', { documentId }),
  restoreSnapshot: (snapshotId: string) => invoke<DocNode>('restore_snapshot', { snapshotId }),

  // Templates
  listTemplates: () => invoke<Template[]>('list_templates'),
  getTemplate: (id: string) => invoke<Template | null>('get_template', { id }),
  saveProjectAsTemplate: (params: {
    projectId: string;
    name: string;
    description?: string;
    locale?: string;
  }) => invoke<Template>('save_project_as_template', params),
  deleteUserTemplate: (id: string) => invoke<void>('delete_user_template', { id }),

  // Collections (Ã‰pica I)
  createCollection: (input: CollectionInput) => invoke<Collection>('create_collection', { input }),
  listCollections: (projectId: string) => invoke<Collection[]>('list_collections', { projectId }),
  renameCollection: (id: string, name: string) =>
    invoke<Collection>('rename_collection', { id, name }),
  setCollectionQuery: (id: string, query: CollectionQuery) =>
    invoke<Collection>('set_collection_query', { id, query }),
  deleteCollection: (id: string) => invoke<void>('delete_collection', { id }),
  setCollectionMembers: (collectionId: string, orderedIds: string[]) =>
    invoke<void>('set_collection_members', { collectionId, orderedIds }),
  resolveCollection: (id: string) => invoke<DocNode[]>('resolve_collection', { id }),

  // Labels (Ã‰pica I â€” I-05/I-06)
  createLabel: (input: LabelInput) => invoke<Label>('create_label', { input }),
  listLabels: (projectId: string) => invoke<Label[]>('list_labels', { projectId }),
  updateLabel: (id: string, name: string, color: string) =>
    invoke<Label>('update_label', { id, name, color }),
  deleteLabel: (id: string) => invoke<void>('delete_label', { id }),
  setDocumentLabels: (documentId: string, labelIds: string[]) =>
    invoke<DocNode>('set_document_labels', { documentId, labelIds }),

  // Custom metadata fields (Ã‰pica I â€” I-08/I-09)
  createCustomField: (input: CustomFieldInput) =>
    invoke<CustomField>('create_custom_field', { input }),
  listCustomFields: (projectId: string) =>
    invoke<CustomField[]>('list_custom_fields', { projectId }),
  updateCustomField: (id: string, name: string, options: string[]) =>
    invoke<CustomField>('update_custom_field', { id, name, options }),
  deleteCustomField: (id: string) => invoke<void>('delete_custom_field', { id }),
  setDocumentMetadata: (documentId: string, fieldId: string, value: string | null) =>
    invoke<DocNode>('set_document_metadata', { documentId, fieldId, value }),

  // Research folder (Ã‰pica I â€” I-10)
  setDocumentResearch: (id: string, isResearch: boolean) =>
    invoke<DocNode>('set_document_research', { id, isResearch }),
  // Front/back matter (Ã‰pica K â€” K-03)
  setDocumentMatter: (id: string, isFront: boolean, isBack: boolean) =>
    invoke<DocNode>('set_document_matter', { id, isFront, isBack }),

  // Search
  searchDocuments: (params: { projectId: string; query: string }) =>
    invoke<SearchHit[]>('search_documents', params),

  // Export
  exportProject: (params: {
    projectId: string;
    format: ExportFormat;
    outputPath: string;
    config?: ExportConfig;
  }) => invoke<string>('export_project', params),
  supportedExportFormats: () => invoke<ExportFormat[]>('supported_export_formats'),
  getExportConfig: (projectId: string) => invoke<ExportConfig>('get_export_config', { projectId }),
  setExportConfig: (params: { projectId: string; config: ExportConfig }) =>
    invoke<void>('set_export_config', params),

  // Import
  importProject: (params: { format: ImportFormat; bytes: number[]; filenameHint: string }) =>
    invoke<Project>('import_project', params),
  supportedImportFormats: () => invoke<ImportFormat[]>('supported_import_formats'),

  // Bibliography
  importBibliography: (params: { projectId: string; bibText: string }) =>
    invoke<BibliographyImportSummary>('import_bibliography', params),
  listCitations: (projectId: string) => invoke<Citation[]>('list_citations', { projectId }),
  listCitationKeys: (projectId: string) => invoke<string[]>('list_citation_keys', { projectId }),
  deleteCitation: (id: string) => invoke<void>('delete_citation', { id }),

  // Backups
  listBackups: () => invoke<BackupRecord[]>('list_backups'),
  createManualBackup: () => invoke<BackupRecord>('create_manual_backup'),
  restoreBackup: (id: string) => invoke<void>('restore_backup', { id }),
  pruneBackups: () => invoke<number>('prune_backups'),

  // Media
  uploadMedia: (params: { projectId: string; mime: string; bytes: number[] }) =>
    invoke<MediaAsset>('upload_media', params),
  readMediaBytes: (id: string) => invoke<number[]>('read_media_bytes', { id }),
  getMediaAsset: (id: string) => invoke<MediaAsset | null>('get_media_asset', { id }),
  listProjectMedia: (projectId: string) =>
    invoke<MediaAsset[]>('list_project_media', { projectId }),
  deleteMedia: (id: string) => invoke<void>('delete_media', { id }),

  // Codex
  createCodexEntry: (input: CodexInput) => invoke<CodexEntry>('create_codex_entry', { input }),
  listCodexEntries: (projectId: string) =>
    invoke<CodexEntry[]>('list_codex_entries', { projectId }),
  getCodexEntry: (id: string) => invoke<CodexEntry | null>('get_codex_entry', { id }),
  updateCodexEntry: (params: { id: string; patch: CodexUpdate }) =>
    invoke<CodexEntry>('update_codex_entry', params),
  deleteCodexEntry: (id: string) => invoke<void>('delete_codex_entry', { id }),
  searchCodexEntries: (params: { projectId: string; query: string; kind?: CodexKind }) =>
    invoke<CodexEntry[]>('search_codex_entries', params),
};
