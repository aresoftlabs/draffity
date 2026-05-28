import { invoke } from '@tauri-apps/api/core';
import type {
  BackupRecord,
  BibliographyImportSummary,
  Citation,
  DocNode,
  DocumentInput,
  DocumentStatus,
  ExportConfig,
  ExportFormat,
  Project,
  ProjectInput,
  SearchHit,
  Snapshot,
  Template,
  WritingStats,
} from '@draffity/shared-types';

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
};
