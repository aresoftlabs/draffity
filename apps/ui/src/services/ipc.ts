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
  ImportFormat,
  MediaAsset,
  Project,
  ProjectInput,
  SearchHit,
  Snapshot,
  Template,
  WritingStats,
} from '@draffity/shared-types';

/** Premium activation status reported by the backend (E-07). */
export interface PremiumStatus {
  /** `'free'` or `'premium'`. */
  tier: string;
  /** Convenience flag: `tier === 'premium'`. */
  active: boolean;
  /** Whether this build can validate licenses (has a public key baked in).
   * When false the UI hides the activation field. */
  licensingConfigured: boolean;
  /** Holder string from the validated license, when just activated. */
  holder: string | null;
}

/**
 * Thin typed wrapper over Tauri IPC commands.
 * Keep this file as the single boundary between Vue and Rust.
 */
export const ipc = {
  // System
  ping: () => invoke<string>('ping'),
  capabilityEnabled: (name: string) => invoke<boolean>('capability_enabled', { name }),
  getSetting: (key: string) => invoke<string | null>('get_setting', { key }),
  setSetting: (key: string, value: string) => invoke<void>('set_setting', { key, value }),
  getWritingStats: () => invoke<WritingStats>('get_writing_stats'),
  getRecentDailyWriting: (days: number) =>
    invoke<DailyWriting[]>('get_recent_daily_writing', { days }),
  getCrashReportingStatus: () =>
    invoke<{ active: boolean; enabled: boolean }>('get_crash_reporting_status'),
  setCrashReportingEnabled: (enabled: boolean) =>
    invoke<void>('set_crash_reporting_enabled', { enabled }),

  // Premium / license
  getPremiumStatus: () => invoke<PremiumStatus>('get_premium_status'),
  activatePremium: (key: string) => invoke<PremiumStatus>('activate_premium', { key }),
  deactivatePremium: () => invoke<PremiumStatus>('deactivate_premium'),

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
