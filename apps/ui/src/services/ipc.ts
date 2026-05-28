import { invoke } from '@tauri-apps/api/core';
import type {
  DocNode,
  DocumentInput,
  ExportFormat,
  Project,
  ProjectInput,
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

  // Documents
  createDocument: (input: DocumentInput) => invoke<DocNode>('create_document', { input }),
  listDocuments: (projectId: string) => invoke<DocNode[]>('list_documents', { projectId }),
  getDocument: (id: string) => invoke<DocNode | null>('get_document', { id }),
  updateDocument: (params: { id: string; title?: string; content?: string }) =>
    invoke<DocNode>('update_document', params),
  moveDocument: (params: { id: string; parentId?: string | null; position: number }) =>
    invoke<void>('move_document', params),
  deleteDocument: (id: string) => invoke<void>('delete_document', { id }),

  // Snapshots
  createSnapshot: (params: { documentId: string; label?: string }) =>
    invoke<Snapshot>('create_snapshot', params),
  listSnapshots: (documentId: string) => invoke<Snapshot[]>('list_snapshots', { documentId }),
  restoreSnapshot: (snapshotId: string) => invoke<DocNode>('restore_snapshot', { snapshotId }),

  // Templates
  listTemplates: () => invoke<Template[]>('list_templates'),
  getTemplate: (id: string) => invoke<Template | null>('get_template', { id }),

  // Export
  exportProject: (params: { projectId: string; format: ExportFormat; outputPath: string }) =>
    invoke<string>('export_project', params),
  supportedExportFormats: () => invoke<ExportFormat[]>('supported_export_formats'),
};
