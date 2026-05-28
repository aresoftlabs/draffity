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
  /** Target word count for the whole project. `null` means no goal set. */
  goalWords?: number | null;
  createdAt: number;
  updatedAt: number;
}

export interface ProjectInput {
  title: string;
  templateId: string;
  metadata?: Record<string, unknown> | null;
}

export type DocumentType = 'chapter' | 'scene' | 'note' | 'folder' | 'manga_page';

/** Position in the writing pipeline. Defaults to `draft` on new documents. */
export type DocumentStatus = 'draft' | 'revised' | 'final' | 'trashed';

export interface DocNode {
  id: string;
  projectId: string;
  parentId?: string | null;
  title: string;
  docType: DocumentType;
  /** HTML render-cache of the document. */
  content?: string | null;
  /** Canonical ProseMirror state (JSON string). Preferred when present. */
  contentJson?: string | null;
  /** Short description surfaced in Corkboard / Outliner views. Independent
   *  of `content`. `null` means no synopsis. */
  synopsis?: string | null;
  position: number;
  status: DocumentStatus;
  tags: string[];
  /** Target word count for this document. `null` means no goal set. */
  goalWords?: number | null;
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

// Search

/** One match from a project-scoped FTS search.
 * `excerpt` contains the matched snippet with `<mark>` tags around hits. */
export interface SearchHit {
  documentId: string;
  projectId: string;
  title: string;
  excerpt: string;
}

// Bibliography

/** A bibliographic entry imported from BibTeX. `fields` is a flat
 *  lowercase map of BibTeX field name → cleaned string value. */
export interface Citation {
  id: string;
  projectId: string;
  key: string;
  entryType: string;
  fields: Record<string, string>;
  createdAt: number;
  updatedAt: number;
}

export interface BibliographyImportSummary {
  imported: Citation[];
  /** Entries the parser dropped because they were malformed. */
  skipped: number;
}

// Media

/** Catalogue row for a blob stored under `<app_data>/<pathRelative>`. The
 *  bytes themselves are fetched via `readMediaBytes`. */
export interface MediaAsset {
  id: string;
  projectId: string;
  /** Relative to `<app_data>` — e.g. `media/<project>/<sha256>.<ext>`. */
  pathRelative: string;
  mime: string;
  sha256: string;
  /** File size in bytes. `i64` on the Rust side so this is a regular JS number
   *  for the foreseeable file sizes a writer pastes. */
  bytes: number;
  createdAt: number;
}

// Backups

export type BackupKind = 'daily' | 'monthly' | 'manual';

export interface BackupRecord {
  id: string;
  path: string;
  createdAt: number;
  sizeBytes: number;
  kind: BackupKind;
}

// Codex

export type CodexKind = 'character' | 'place' | 'object' | 'note';

export interface CodexEntry {
  id: string;
  projectId: string;
  kind: CodexKind;
  name: string;
  body?: string | null;
  tags: string[];
  createdAt: number;
  updatedAt: number;
}

export interface CodexInput {
  projectId: string;
  kind: CodexKind;
  name: string;
  body?: string | null;
  tags?: string[];
}

export interface CodexUpdate {
  name?: string;
  kind?: CodexKind;
  body?: string | null;
  tags?: string[];
}

// Export

export type ExportFormat = 'markdown' | 'docx' | 'epub' | 'pdf';

export type PageSize =
  | 'a4'
  | 'letter'
  | 'legal'
  | { custom: { widthMm: number; heightMm: number } };

export interface Margins {
  topMm: number;
  rightMm: number;
  bottomMm: number;
  leftMm: number;
}

export type SceneSeparator =
  | { kind: 'stars' }
  | { kind: 'dashes' }
  | { kind: 'blank' }
  | { kind: 'custom'; value: string };

/** User-tunable export options. Persisted per-project. Backend tolerates
 *  partial payloads — any missing field falls back to a default. */
export interface ExportConfig {
  titleOverride?: string | null;
  author?: string | null;
  fontFamily?: string | null;
  pageSize: PageSize;
  margins: Margins;
  includeToc: boolean;
  includeTitlePage: boolean;
  sceneSeparator: SceneSeparator;
  coverImagePath?: string | null;
  /** When true, exporters append a "Codex" appendix at the end of the
   *  document listing every entry grouped by kind. */
  includeCodex: boolean;
}

/** Defaults mirrored from `services::exporter::config::ExportConfig::default()`. */
export const DEFAULT_EXPORT_CONFIG: ExportConfig = {
  titleOverride: null,
  author: null,
  fontFamily: null,
  pageSize: 'a4',
  margins: { topMm: 25, rightMm: 25, bottomMm: 25, leftMm: 25 },
  includeToc: true,
  includeTitlePage: true,
  sceneSeparator: { kind: 'stars' },
  coverImagePath: null,
  includeCodex: false,
};

// Writing stats

export interface WritingStats {
  currentStreak: number;
  longestStreak: number;
  lastWritingDate?: string | null;
}

/** One day in the writing-activity series. Date is `YYYY-MM-DD` local. */
export interface DailyWriting {
  date: string;
  words: number;
  sessions: number;
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
