// Shared types between Rust (apps/desktop) and Vue (apps/ui).
// Aligned with the SQLite v1 schema and the camelCase serde projection.
// Phase 1+ may auto-generate this file from Rust via specta or ts-rs.

export type ProjectStatus = 'active' | 'archived';

export interface Project {
  id: string;
  title: string;
  templateId: string;
  status: ProjectStatus;
  metadata?: Record<string, unknown> | null;
  createdAt: number;
  updatedAt: number;
}

export interface ProjectInput {
  title: string;
  templateId: string;
  metadata?: Record<string, unknown> | null;
}

export type DocumentType = 'chapter' | 'scene' | 'note' | 'folder' | 'manga_page';

export interface DocNode {
  id: string;
  projectId: string;
  parentId?: string | null;
  title: string;
  docType: DocumentType;
  content?: string | null;
  position: number;
  createdAt: number;
  updatedAt: number;
}

export interface DocumentInput {
  projectId: string;
  parentId?: string | null;
  title: string;
  docType: DocumentType;
  content?: string | null;
}

export interface Snapshot {
  id: string;
  documentId: string;
  content: string;
  label?: string | null;
  createdAt: number;
}

// Templates (schema v1)

export type TemplateKind = 'novel' | 'paper' | 'manga' | 'screenplay' | 'generic';
export type TemplateTier = 'free' | 'premium';
export type FieldType = 'string' | 'text' | 'number' | 'date';

export interface MetadataField {
  key: string;
  label: string;
  type: FieldType;
  required?: boolean;
  default?: unknown;
}

export interface TemplateNode {
  title: string;
  docType: DocumentType;
  synopsis?: string;
  children?: TemplateNode[];
}

export interface Template {
  schemaVersion: number;
  id: string;
  name: string;
  description?: string;
  kind: TemplateKind;
  locale: string;
  tier: TemplateTier;
  structure: TemplateNode[];
  metadataFields: MetadataField[];
}

// Export

export type ExportFormat = 'markdown' | 'docx' | 'epub' | 'pdf';

// Writing stats

export interface WritingStats {
  currentStreak: number;
  longestStreak: number;
  lastWritingDate?: string | null;
}

export type WireErrorCode =
  | 'io'
  | 'sqlite'
  | 'json'
  | 'invariant'
  | 'not_found'
  | 'unsupported'
  | 'unexpected';

export interface WireError {
  code: WireErrorCode;
  message: string;
}

/** Names of events emitted on the Tauri event bus. Stable wire identifiers. */
export const Events = {
  ProjectCreated: 'project.created',
  ProjectOpened: 'project.opened',
  ProjectArchived: 'project.archived',
  ProjectDeleted: 'project.deleted',
  DocumentCreated: 'document.created',
  DocumentSaved: 'document.saved',
  DocumentMoved: 'document.moved',
  DocumentDeleted: 'document.deleted',
  SnapshotCreated: 'snapshot.created',
} as const;

export type EventName = (typeof Events)[keyof typeof Events];
