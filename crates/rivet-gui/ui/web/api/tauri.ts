import { invoke } from "@tauri-apps/api/core";
import type { ZodType } from "zod";

import { logger, setLoggerBridge } from "../lib/logger";
import {
  ContactCreateSchema,
  ContactDtoArraySchema,
  ContactDtoSchema,
  ContactOpenActionResultSchema,
  ContactUpdateArgsSchema,
  ContactsDedupeDecideResultSchema,
  ContactsDedupePreviewResultSchema,
  ContactsImportCommitResultSchema,
  ContactsImportPreviewResultSchema,
  ContactsListResultSchema,
  ContactsMergeResultSchema,
  ContactsMergeUndoResultSchema,
  ExternalCalendarCacheEntryArraySchema,
  ExternalCalendarSourceSchema,
  ExternalCalendarSyncResultSchema,
  RivetRuntimeConfigSchema,
  TagSchemaSchema,
  TaskCreateSchema,
  TaskDtoArraySchema,
  TaskDtoSchema,
  TaskUpdateArgsSchema,
  describeSchemaError
} from "./schemas";
import type {
  ContactCreate,
  ContactFieldValue,
  ContactDto,
  ContactIdArg,
  ContactOpenActionArgs,
  ContactOpenActionResult,
  ContactUpdateArgs,
  ContactsDedupePreviewArgs,
  ContactsDedupeDecideArgs,
  ContactsDedupeDecideResult,
  ContactsDedupePreviewResult,
  ContactsDeleteBulkArgs,
  ContactsImportCommitArgs,
  ContactsImportCommitResult,
  ContactsImportPreviewArgs,
  ContactsImportPreviewResult,
  ContactsListArgs,
  ContactsListResult,
  ContactsMergeArgs,
  ContactsMergeResult,
  ContactsMergeUndoArgs,
  ContactsMergeUndoResult,
  ExternalCalendarCacheEntry,
  ExternalCalendarSource,
  ExternalCalendarSyncResult,
  TaskCreate,
  TaskDto,
  TaskIdArg,
  TasksListArgs,
  TaskUpdateArgs
} from "../types/core";
import type { RivetRuntimeConfig, TagSchema } from "../types/config";

const MOCK_TASKS_KEY = "rivet.mock.tasks";
const MOCK_CONTACTS_KEY = "rivet.mock.contacts";
const MOCK_CONTACTS_DEDUPE_DECISIONS_KEY = "rivet.mock.contacts.dedupe.decisions";
const MOCK_CONTACTS_MERGE_UNDO_KEY = "rivet.mock.contacts.merge.undo";
const DEFAULT_TIMEOUT_MS = 30_000;
const EXTERNAL_CALENDAR_TIMEOUT_MS = 90_000;
const DEFAULT_TASK_QUERY: TasksListArgs = {
  query: null,
  status: null,
  project: null,
  tag: null
};
const DEFAULT_CONTACTS_QUERY: ContactsListArgs = {
  query: null,
  limit: 200,
  cursor: null,
  source: null,
  updated_after: null
};

export interface CommandFailureRecord {
  command: string;
  request_id: string;
  duration_ms: number;
  error: string;
  timestamp: string;
}

type CommandFailureSink = (record: CommandFailureRecord) => void;

let commandFailureSink: CommandFailureSink | null = null;

type RuntimeTransportMode = "auto" | "tauri" | "mock";
export interface ConfigEntryUpdate {
  section: string;
  key: string;
  value: string | number | boolean;
}

function resolveRuntimeTransportMode(): RuntimeTransportMode {
  const raw = String(import.meta.env.VITE_RIVET_UI_RUNTIME_MODE ?? "auto").trim().toLowerCase();
  if (raw === "tauri") {
    return "tauri";
  }
  if (raw === "mock") {
    return "mock";
  }
  return "auto";
}

const runtimeTransportMode = resolveRuntimeTransportMode();
const verboseInvokeLogging = String(import.meta.env.VITE_RIVET_TRACE_INVOKE ?? "").trim() === "1";

export function setCommandFailureSink(sink: CommandFailureSink | null): void {
  commandFailureSink = sink;
}

const isTauriRuntime = (): boolean => {
  if (runtimeTransportMode === "mock") {
    return false;
  }
  if (runtimeTransportMode === "tauri") {
    return true;
  }
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
};

function parseWithSchema<T>(
  context: string,
  value: unknown,
  parser: ZodType<T>
): T {
  const result = parser.safeParse(value);
  if (!result.success) {
    const message = describeSchemaError(context, result.error);
    throw new Error(message);
  }
  return result.data;
}

function readLocalStorageJson(key: string): unknown {
  if (typeof window === "undefined") {
    return [];
  }
  const raw = window.localStorage.getItem(key);
  if (!raw) {
    return [];
  }
  try {
    return JSON.parse(raw);
  } catch {
    return [];
  }
}

function parseStoredTasks(): TaskDto[] {
  return parseWithSchema("mock.tasks", readLocalStorageJson(MOCK_TASKS_KEY), TaskDtoArraySchema);
}

function parseStoredContacts(): ContactDto[] {
  return parseWithSchema("mock.contacts", readLocalStorageJson(MOCK_CONTACTS_KEY), ContactDtoArraySchema);
}

type DedupeDecisionMap = Record<string, string>;
type MockMergeUndoEntry = {
  undo_id: string;
  contacts_before: ContactDto[];
};
function parseStoredDedupeDecisions(): DedupeDecisionMap {
  const raw = readLocalStorageJson(MOCK_CONTACTS_DEDUPE_DECISIONS_KEY);
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    return {};
  }
  const out: DedupeDecisionMap = {};
  for (const [key, value] of Object.entries(raw)) {
    if (typeof value === "string" && key.trim().length > 0) {
      out[key] = value;
    }
  }
  return out;
}

function parseStoredMergeUndoEntries(): MockMergeUndoEntry[] {
  const raw = readLocalStorageJson(MOCK_CONTACTS_MERGE_UNDO_KEY);
  if (!Array.isArray(raw)) {
    return [];
  }
  return raw
    .filter((entry): entry is MockMergeUndoEntry => {
      if (!entry || typeof entry !== "object" || Array.isArray(entry)) {
        return false;
      }
      const undo_id = (entry as { undo_id?: unknown }).undo_id;
      const contacts_before = (entry as { contacts_before?: unknown }).contacts_before;
      return typeof undo_id === "string" && Array.isArray(contacts_before);
    })
    .map((entry) => ({
      undo_id: entry.undo_id,
      contacts_before: parseWithSchema("mock.contacts_merge_undo.contacts_before", entry.contacts_before, ContactDtoArraySchema)
    }));
}

function writeStorageJson(key: string, value: unknown): void {
  if (typeof window === "undefined") {
    return;
  }
  window.localStorage.setItem(key, JSON.stringify(value));
}

function writeStoredTasks(tasks: TaskDto[]): void {
  writeStorageJson(MOCK_TASKS_KEY, tasks);
}

function writeStoredContacts(contacts: ContactDto[]): void {
  writeStorageJson(MOCK_CONTACTS_KEY, contacts);
}

function writeStoredDedupeDecisions(decisions: DedupeDecisionMap): void {
  writeStorageJson(MOCK_CONTACTS_DEDUPE_DECISIONS_KEY, decisions);
}

function writeStoredMergeUndoEntries(entries: MockMergeUndoEntry[]): void {
  writeStorageJson(MOCK_CONTACTS_MERGE_UNDO_KEY, entries);
}

function makeMockTask(input: TaskCreate): TaskDto {
  const now = new Date().toISOString();
  return {
    uuid: crypto.randomUUID(),
    id: null,
    title: input.title,
    description: input.description,
    status: "Pending",
    project: input.project,
    tags: input.tags,
    priority: input.priority,
    due: input.due,
    wait: input.wait,
    scheduled: input.scheduled,
    created: now,
    modified: now
  };
}

function makeMockContact(input: ContactCreate): ContactDto {
  const now = new Date().toISOString();
  const firstEmail = input.emails.find((item) => item.value.trim().length > 0)?.value ?? "";
  const firstPhone = input.phones.find((item) => item.value.trim().length > 0)?.value ?? "";
  const display = input.display_name?.trim() || [input.given_name ?? "", input.family_name ?? ""].join(" ").trim() || firstEmail || firstPhone || "Unnamed Contact";

  return {
    id: crypto.randomUUID(),
    display_name: display,
    avatar_data_url: input.avatar_data_url ?? null,
    import_batch_id: input.import_batch_id ?? null,
    source_file_name: input.source_file_name ?? null,
    given_name: input.given_name,
    family_name: input.family_name,
    nickname: input.nickname,
    notes: input.notes,
    phones: input.phones,
    emails: input.emails,
    websites: input.websites,
    birthday: input.birthday,
    organization: input.organization,
    title: input.title,
    addresses: input.addresses,
    source_id: input.source_id ?? "local",
    source_kind: input.source_kind ?? "local",
    remote_id: input.remote_id,
    link_group_id: input.link_group_id,
    created_at: now,
    updated_at: now
  };
}

function contactSearchMatches(contact: ContactDto, query: string): boolean {
  const normalize = (value: string) => value
    .normalize("NFKD")
    .replace(/\p{M}/gu, "")
    .toLowerCase();
  const q = normalize(query.trim());
  if (!q) {
    return true;
  }

  const fields = [
    contact.display_name,
    contact.given_name ?? "",
    contact.family_name ?? "",
    contact.nickname ?? "",
    contact.notes ?? "",
    contact.organization ?? "",
    ...contact.emails.map((item) => item.value),
    ...contact.phones.map((item) => item.value)
  ].join(" ");

  return normalize(fields).includes(q);
}

function normalizeMockSource(source: string): string {
  const token = source.trim().toLowerCase();
  if (token.includes("gmail")) {
    return "gmail";
  }
  if (token.includes("iphone") || token.includes("icloud")) {
    return "iphone";
  }
  return "vcard";
}

function normalizeMockToken(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .trim();
}

function normalizeMockEmail(value: string): string {
  return normalizeMockToken(value);
}

function normalizeMockPhone(value: string): string {
  return value.replace(/[^0-9+]/g, "").toLowerCase();
}

function mockFieldKind(header: string, fallback: string): string {
  const token = header.toLowerCase();
  if (token.includes("home")) {
    return "home";
  }
  if (token.includes("work")) {
    return "work";
  }
  if (token.includes("cell") || token.includes("mobile") || token.includes("iphone")) {
    return "mobile";
  }
  return fallback;
}

function mockFieldPrimary(header: string): boolean {
  const token = header.toLowerCase();
  return token.includes("pref") || token.includes("primary") || token.includes("main");
}

function unescapeVcard(value: string): string {
  return value
    .replace(/\\n/gi, "\n")
    .replace(/\\,/g, ",")
    .replace(/\\;/g, ";")
    .trim();
}

function parseMockVcardContacts(
  content: string,
  sourceKind: string,
  fileName: string | null,
  batchId: string
): { contacts: ContactDto[]; errors: string[] } {
  const unfolded = content
    .replace(/\r\n[ \t]/g, "")
    .replace(/\n[ \t]/g, "");
  const blocks = unfolded
    .split(/BEGIN:VCARD/gi)
    .map((block) => block.trim())
    .filter((block) => block.length > 0)
    .map((block) => `BEGIN:VCARD\n${block}`);
  const contacts: ContactDto[] = [];
  const errors: string[] = [];

  for (const [index, block] of blocks.entries()) {
    if (!/END:VCARD/i.test(block)) {
      errors.push(`row ${index + 1}: missing END:VCARD`);
      continue;
    }

    const lines = block.split(/\r?\n/);
    let displayName = "";
    let givenName = "";
    let familyName = "";
    let nickname = "";
    let notes = "";
    let organization = "";
    let title = "";
    let birthday: string | null = null;
    const emails: ContactFieldValue[] = [];
    const phones: ContactFieldValue[] = [];
    const websites: ContactFieldValue[] = [];
    const addresses: ContactCreate["addresses"] = [];

    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed.length === 0 || /^BEGIN:VCARD$/i.test(trimmed) || /^END:VCARD$/i.test(trimmed)) {
        continue;
      }
      const colonIndex = trimmed.indexOf(":");
      if (colonIndex < 0) {
        continue;
      }

      const header = trimmed.slice(0, colonIndex);
      const value = unescapeVcard(trimmed.slice(colonIndex + 1));
      const upper = header.toUpperCase();
      if (upper.startsWith("FN")) {
        displayName = value;
        continue;
      }
      if (upper.startsWith("N")) {
        const [family = "", given = ""] = value.split(";");
        familyName = familyName || family.trim();
        givenName = givenName || given.trim();
        continue;
      }
      if (upper.startsWith("NICKNAME")) {
        nickname = value;
        continue;
      }
      if (upper.startsWith("EMAIL")) {
        if (value.length > 0) {
          emails.push({
            value,
            kind: mockFieldKind(header, "home"),
            is_primary: mockFieldPrimary(header)
          });
        }
        continue;
      }
      if (upper.startsWith("TEL")) {
        if (value.length > 0) {
          phones.push({
            value,
            kind: mockFieldKind(header, "mobile"),
            is_primary: mockFieldPrimary(header)
          });
        }
        continue;
      }
      if (upper.startsWith("URL")) {
        if (value.length > 0) {
          websites.push({
            value,
            kind: mockFieldKind(header, "other"),
            is_primary: mockFieldPrimary(header)
          });
        }
        continue;
      }
      if (upper.startsWith("ADR")) {
        const parts = value.split(";");
        const street = (parts[2] ?? "").trim();
        const city = (parts[3] ?? "").trim();
        const region = (parts[4] ?? "").trim();
        const postal_code = (parts[5] ?? "").trim();
        const country = (parts[6] ?? "").trim();
        if (street || city || region || postal_code || country) {
          addresses.push({
            kind: mockFieldKind(header, "home"),
            street,
            city,
            region,
            postal_code,
            country
          });
        }
        continue;
      }
      if (upper.startsWith("NOTE")) {
        notes = notes ? `${notes}\n${value}` : value;
        continue;
      }
      if (upper.startsWith("ORG")) {
        organization = value;
        continue;
      }
      if (upper.startsWith("TITLE")) {
        title = value;
        continue;
      }
      if (upper.startsWith("BDAY")) {
        birthday = value || null;
      }
    }

    const firstEmail = emails.find((item) => item.value.trim().length > 0)?.value ?? "";
    const firstPhone = phones.find((item) => item.value.trim().length > 0)?.value ?? "";
    const resolvedName = displayName.trim() || `${givenName} ${familyName}`.trim() || firstEmail || firstPhone;
    if (!resolvedName && !firstEmail && !firstPhone) {
      errors.push(`row ${index + 1}: missing name, email, and phone`);
      continue;
    }
    if (emails.length > 0 && !emails.some((item) => item.is_primary)) {
      emails[0] = { ...emails[0], is_primary: true };
    }
    if (phones.length > 0 && !phones.some((item) => item.is_primary)) {
      phones[0] = { ...phones[0], is_primary: true };
    }
    if (websites.length > 0 && !websites.some((item) => item.is_primary)) {
      websites[0] = { ...websites[0], is_primary: true };
    }

    contacts.push(makeMockContact({
      display_name: resolvedName,
      avatar_data_url: null,
      import_batch_id: batchId,
      source_file_name: fileName,
      given_name: givenName || null,
      family_name: familyName || null,
      nickname: nickname || null,
      notes: notes || null,
      phones,
      emails,
      websites,
      birthday,
      organization: organization || null,
      title: title || null,
      addresses,
      source_id: `import:${sourceKind}`,
      source_kind: sourceKind,
      remote_id: null,
      link_group_id: null
    }));
  }

  return { contacts, errors };
}

function mockMergeFields(current: ContactFieldValue[], incoming: ContactFieldValue[]): ContactFieldValue[] {
  const seen = new Set(current.map((item) => `${item.kind}:${normalizeMockToken(item.value)}`));
  const out = [...current];
  for (const value of incoming) {
    const key = `${value.kind}:${normalizeMockToken(value.value)}`;
    if (!value.value.trim() || seen.has(key)) {
      continue;
    }
    out.push({ ...value, is_primary: false });
    seen.add(key);
  }
  const primaryIndex = out.findIndex((item) => item.is_primary);
  if (out.length > 0) {
    if (primaryIndex < 0) {
      out[0] = { ...out[0], is_primary: true };
    } else {
      for (let index = 0; index < out.length; index += 1) {
        out[index] = { ...out[index], is_primary: index === primaryIndex };
      }
    }
  }
  return out;
}

function mergeMockContact(existing: ContactDto, incoming: ContactDto): ContactDto {
  return {
    ...existing,
    display_name: existing.display_name.trim() || incoming.display_name,
    avatar_data_url: existing.avatar_data_url || incoming.avatar_data_url,
    import_batch_id: existing.import_batch_id || incoming.import_batch_id,
    source_file_name: existing.source_file_name || incoming.source_file_name,
    given_name: existing.given_name || incoming.given_name,
    family_name: existing.family_name || incoming.family_name,
    nickname: existing.nickname || incoming.nickname,
    notes: existing.notes || incoming.notes,
    phones: mockMergeFields(existing.phones, incoming.phones),
    emails: mockMergeFields(existing.emails, incoming.emails),
    websites: mockMergeFields(existing.websites, incoming.websites),
    birthday: existing.birthday || incoming.birthday,
    organization: existing.organization || incoming.organization,
    title: existing.title || incoming.title,
    addresses: existing.addresses.length > 0 ? existing.addresses : incoming.addresses,
    source_id: existing.source_id || incoming.source_id,
    source_kind: existing.source_kind || incoming.source_kind,
    remote_id: existing.remote_id || incoming.remote_id,
    link_group_id: existing.link_group_id || incoming.link_group_id,
    updated_at: new Date().toISOString()
  };
}

function bestMockConflict(
  incoming: ContactDto,
  existing: ContactDto[]
): { index: number; score: number; reason: string } | null {
  const incomingEmails = new Set(incoming.emails.map((item) => normalizeMockEmail(item.value)).filter(Boolean));
  const incomingPhones = new Set(incoming.phones.map((item) => normalizeMockPhone(item.value)).filter(Boolean));
  const incomingName = normalizeMockToken(incoming.display_name);
  const incomingOrg = normalizeMockToken(incoming.organization ?? "");
  const incomingDomains = new Set(
    incoming.emails
      .map((item) => normalizeMockEmail(item.value).split("@")[1] ?? "")
      .filter(Boolean)
  );

  for (let index = 0; index < existing.length; index += 1) {
    const candidate = existing[index];
    const candidateEmails = new Set(candidate.emails.map((item) => normalizeMockEmail(item.value)).filter(Boolean));
    for (const token of incomingEmails) {
      if (candidateEmails.has(token)) {
        return { index, score: 100, reason: "exact email match" };
      }
    }

    const candidatePhones = new Set(candidate.phones.map((item) => normalizeMockPhone(item.value)).filter(Boolean));
    for (const token of incomingPhones) {
      if (candidatePhones.has(token)) {
        return { index, score: 100, reason: "exact phone match" };
      }
    }

    const candidateName = normalizeMockToken(candidate.display_name);
    if (incomingName.length > 0 && incomingName === candidateName) {
      const candidateOrg = normalizeMockToken(candidate.organization ?? "");
      if (incomingOrg.length > 0 && incomingOrg === candidateOrg) {
        return { index, score: 70, reason: "same full name + org" };
      }
      const candidateDomains = new Set(
        candidate.emails
          .map((item) => normalizeMockEmail(item.value).split("@")[1] ?? "")
          .filter(Boolean)
      );
      for (const domain of incomingDomains) {
        if (candidateDomains.has(domain)) {
          return { index, score: 60, reason: "same full name + email domain" };
        }
      }
    }
  }

  return null;
}

async function invokeCommand<R>(command: string, args?: unknown): Promise<R> {
  const requestId = crypto.randomUUID();
  const startedAt = performance.now();
  const timeoutMs = command.startsWith("external_calendar_") ? EXTERNAL_CALENDAR_TIMEOUT_MS : DEFAULT_TIMEOUT_MS;
  const instrumentCommand = command !== "ui_log";
  if (instrumentCommand && verboseInvokeLogging) {
    logger.debug("invoke.start", `${command} request_id=${requestId}`);
  }

  const run = async (): Promise<R> => {
    if (isTauriRuntime()) {
      const payload = typeof args === "undefined" ? undefined : args;
      if (typeof payload === "undefined") {
        return invoke<R>(command, { request_id: requestId });
      }
      return invoke<R>(command, { args: payload, request_id: requestId });
    }

    switch (command) {
      case "tasks_list": {
        return parseStoredTasks() as R;
      }
      case "task_add": {
        const payload = args as TaskCreate;
        const tasks = parseStoredTasks();
        const task = makeMockTask(payload);
        tasks.unshift(task);
        writeStoredTasks(tasks);
        return task as R;
      }
      case "task_done": {
        const payload = args as TaskIdArg;
        const tasks = parseStoredTasks().map((entry) => {
          if (entry.uuid !== payload.uuid) {
            return entry;
          }
          return {
            ...entry,
            status: "Completed" as const,
            modified: new Date().toISOString()
          };
        });
        writeStoredTasks(tasks);
        const target = tasks.find((entry) => entry.uuid === payload.uuid);
        if (!target) {
          throw new Error(`task not found: ${payload.uuid}`);
        }
        return target as R;
      }
      case "task_uncomplete": {
        const payload = args as TaskIdArg;
        const tasks = parseStoredTasks().map((entry) => {
          if (entry.uuid !== payload.uuid) {
            return entry;
          }
          return {
            ...entry,
            status: "Pending" as const,
            modified: new Date().toISOString()
          };
        });
        writeStoredTasks(tasks);
        const target = tasks.find((entry) => entry.uuid === payload.uuid);
        if (!target) {
          throw new Error(`task not found: ${payload.uuid}`);
        }
        return target as R;
      }
      case "task_delete": {
        const payload = args as TaskIdArg;
        const tasks = parseStoredTasks().filter((entry) => entry.uuid !== payload.uuid);
        writeStoredTasks(tasks);
        return undefined as R;
      }
      case "task_update": {
        const payload = args as TaskUpdateArgs;
        const tasks = parseStoredTasks().map((entry) => {
          if (entry.uuid !== payload.uuid) {
            return entry;
          }

          return {
            ...entry,
            title: payload.patch.title ?? entry.title,
            description: payload.patch.description ?? entry.description,
            project: typeof payload.patch.project === "undefined" ? entry.project : payload.patch.project,
            tags: payload.patch.tags ?? entry.tags,
            priority: typeof payload.patch.priority === "undefined" ? entry.priority : payload.patch.priority,
            due: typeof payload.patch.due === "undefined" ? entry.due : payload.patch.due,
            wait: typeof payload.patch.wait === "undefined" ? entry.wait : payload.patch.wait,
            scheduled: typeof payload.patch.scheduled === "undefined" ? entry.scheduled : payload.patch.scheduled,
            modified: new Date().toISOString()
          };
        });
        writeStoredTasks(tasks);
        const target = tasks.find((entry) => entry.uuid === payload.uuid);
        if (!target) {
          throw new Error(`task not found: ${payload.uuid}`);
        }
        return target as R;
      }
      case "contacts_list": {
        const payload = (args ?? DEFAULT_CONTACTS_QUERY) as ContactsListArgs;
        const all = parseStoredContacts();
        const filtered = all.filter((contact) => contactSearchMatches(contact, payload.query ?? ""));
        const total = filtered.length;
        const limit = Math.max(1, payload.limit ?? 200);
        const offset = Number(payload.cursor ?? "0") || 0;
        const contacts = filtered.slice(offset, offset + limit);
        const next_cursor = offset + contacts.length < total ? String(offset + contacts.length) : null;
        return { contacts, next_cursor, total } as R;
      }
      case "contact_add": {
        const payload = args as ContactCreate;
        const contacts = parseStoredContacts();
        const created = makeMockContact(payload);
        contacts.unshift(created);
        writeStoredContacts(contacts);
        return created as R;
      }
      case "contact_update": {
        const payload = args as ContactUpdateArgs;
        const contacts = parseStoredContacts().map((entry) => {
          if (entry.id !== payload.id) {
            return entry;
          }
          return {
            ...entry,
            display_name: typeof payload.patch.display_name === "undefined" ? entry.display_name : (payload.patch.display_name ?? ""),
            avatar_data_url: typeof payload.patch.avatar_data_url === "undefined" ? entry.avatar_data_url : payload.patch.avatar_data_url,
            import_batch_id: typeof payload.patch.import_batch_id === "undefined" ? entry.import_batch_id : payload.patch.import_batch_id,
            source_file_name: typeof payload.patch.source_file_name === "undefined" ? entry.source_file_name : payload.patch.source_file_name,
            given_name: typeof payload.patch.given_name === "undefined" ? entry.given_name : payload.patch.given_name,
            family_name: typeof payload.patch.family_name === "undefined" ? entry.family_name : payload.patch.family_name,
            nickname: typeof payload.patch.nickname === "undefined" ? entry.nickname : payload.patch.nickname,
            notes: typeof payload.patch.notes === "undefined" ? entry.notes : payload.patch.notes,
            phones: payload.patch.phones ?? entry.phones,
            emails: payload.patch.emails ?? entry.emails,
            websites: payload.patch.websites ?? entry.websites,
            birthday: typeof payload.patch.birthday === "undefined" ? entry.birthday : payload.patch.birthday,
            organization: typeof payload.patch.organization === "undefined" ? entry.organization : payload.patch.organization,
            title: typeof payload.patch.title === "undefined" ? entry.title : payload.patch.title,
            addresses: payload.patch.addresses ?? entry.addresses,
            source_id: typeof payload.patch.source_id === "undefined" ? entry.source_id : (payload.patch.source_id ?? ""),
            source_kind: typeof payload.patch.source_kind === "undefined" ? entry.source_kind : (payload.patch.source_kind ?? ""),
            remote_id: typeof payload.patch.remote_id === "undefined" ? entry.remote_id : payload.patch.remote_id,
            link_group_id: typeof payload.patch.link_group_id === "undefined" ? entry.link_group_id : payload.patch.link_group_id,
            updated_at: new Date().toISOString()
          };
        });
        writeStoredContacts(contacts);
        const target = contacts.find((entry) => entry.id === payload.id);
        if (!target) {
          throw new Error(`contact not found: ${payload.id}`);
        }
        return target as R;
      }
      case "contact_delete": {
        const payload = args as ContactIdArg;
        const contacts = parseStoredContacts().filter((entry) => entry.id !== payload.id);
        writeStoredContacts(contacts);
        return undefined as R;
      }
      case "contacts_delete_bulk": {
        const payload = args as ContactsDeleteBulkArgs;
        const ids = new Set(payload.ids);
        const contacts = parseStoredContacts();
        const kept = contacts.filter((entry) => !ids.has(entry.id));
        const deleted = contacts.length - kept.length;
        writeStoredContacts(kept);
        return deleted as R;
      }
      case "contacts_dedupe_preview":
      case "contacts_dedupe_candidates": {
        const all = parseStoredContacts();
        const decisions = parseStoredDedupeDecisions();
        const groups = new Map<string, ContactDto[]>();
        for (const contact of all) {
          const key = contact.display_name.trim().toLowerCase();
          if (!key) {
            continue;
          }
          const current = groups.get(key) ?? [];
          current.push(contact);
          groups.set(key, current);
        }
        const out = [...groups.entries()]
          .filter(([, contacts]) => contacts.length > 1)
          .map(([key, contacts]) => ({
            group_id: `group:${contacts.map((contact) => contact.id).sort().join(",")}`,
            reason: `same name: ${key}`,
            score: 60,
            contacts
          }))
          .filter((group) => {
            const decision = (decisions[group.group_id] ?? "").toLowerCase();
            return decision !== "ignored" && decision !== "separate" && decision !== "merged";
          });
        return { groups: out } as R;
      }
      case "contacts_dedupe_decide": {
        const payload = args as ContactsDedupeDecideArgs;
        const decisions = parseStoredDedupeDecisions();
        const decision = (payload.decision ?? "").trim().toLowerCase() || "ignored";
        decisions[payload.candidate_group_id] = decision;
        writeStoredDedupeDecisions(decisions);
        return {
          candidate_group_id: payload.candidate_group_id,
          decision,
          actor: payload.actor ?? "user",
          decided_at: new Date().toISOString()
        } as R;
      }
      case "contact_open_action": {
        const payload = args as ContactOpenActionArgs;
        const url = payload.action === "tel" || payload.action === "phone"
          ? `tel:${payload.value ?? ""}`
          : `mailto:${payload.value ?? ""}`;
        return {
          launched: false,
          url
        } as R;
      }
      case "contacts_import_preview": {
        const payload = args as ContactsImportPreviewArgs;
        const source = normalizeMockSource(payload.source);
        const batch_id = crypto.randomUUID();
        const parsed = parseMockVcardContacts(payload.content, source, payload.file_name ?? null, batch_id);
        const existing = parseStoredContacts();
        const conflicts = parsed.contacts
          .map((imported) => {
            const match = bestMockConflict(imported, existing);
            if (!match || match.score < 80) {
              return null;
            }
            return {
              imported,
              existing: existing[match.index]!,
              score: match.score,
              reason: match.reason
            };
          })
          .filter((item): item is NonNullable<typeof item> => item !== null);
        return {
          batch_id,
          source,
          total_rows: parsed.contacts.length + parsed.errors.length,
          valid_rows: parsed.contacts.length,
          skipped_rows: parsed.errors.length,
          potential_duplicates: conflicts.length,
          contacts: parsed.contacts,
          conflicts,
          errors: parsed.errors
        } as R;
      }
      case "contacts_import_commit": {
        const payload = args as ContactsImportCommitArgs;
        const source = normalizeMockSource(payload.source);
        const batch_id = crypto.randomUUID();
        const parsed = parseMockVcardContacts(payload.content, source, payload.file_name ?? null, batch_id);
        const mode = String(payload.mode ?? "safe").trim().toLowerCase();
        const contacts = parseStoredContacts();
        let created = 0;
        let updated = 0;
        let skipped = 0;
        let conflicts = 0;

        for (const incoming of parsed.contacts) {
          const match = bestMockConflict(incoming, contacts);
          if (match && match.score >= 80) {
            conflicts += 1;
            if (mode === "upsert") {
              const current = contacts[match.index];
              if (current) {
                contacts[match.index] = mergeMockContact(current, incoming);
                updated += 1;
              } else {
                skipped += 1;
              }
            } else {
              skipped += 1;
            }
            continue;
          }

          contacts.unshift(incoming);
          created += 1;
        }
        writeStoredContacts(contacts);

        return {
          batch_id,
          created,
          updated,
          skipped,
          failed: parsed.errors.length,
          conflicts,
          errors: parsed.errors
        } as R;
      }
      case "contacts_merge": {
        const payload = args as ContactsMergeArgs;
        const contacts = parseStoredContacts();
        const ids = new Set(payload.ids);
        const selected = contacts.filter((entry) => ids.has(entry.id));
        if (selected.length < 2) {
          throw new Error("need at least two contacts to merge");
        }
        const targetId = payload.target_id ?? selected[0]!.id;
        let merged = selected.find((entry) => entry.id === targetId) ?? selected[0]!;
        const removed = selected.filter((entry) => entry.id !== merged.id).map((entry) => entry.id);
        for (const entry of selected) {
          if (entry.id === merged.id) {
            continue;
          }
          merged = mergeMockContact(merged, entry);
        }

        const next = contacts
          .filter((entry) => !removed.includes(entry.id))
          .map((entry) => (entry.id === merged.id ? merged : entry));
        writeStoredContacts(next);

        const snapshots = parseStoredMergeUndoEntries();
        const undo_id = crypto.randomUUID();
        snapshots.push({
          undo_id,
          contacts_before: contacts
        });
        writeStoredMergeUndoEntries(snapshots.slice(-20));

        const decisionGroupId = `group:${selected.map((entry) => entry.id).sort().join(",")}`;
        const decisions = parseStoredDedupeDecisions();
        decisions[decisionGroupId] = "merged";
        writeStoredDedupeDecisions(decisions);

        return {
          merged,
          removed_ids: removed,
          undo_id
        } as R;
      }
      case "contacts_merge_undo": {
        const payload = args as ContactsMergeUndoArgs;
        const snapshots = parseStoredMergeUndoEntries();
        if (snapshots.length === 0) {
          return {
            restored: 0,
            undo_id: payload?.undo_id ?? ""
          } as R;
        }

        const index = payload?.undo_id
          ? snapshots.findIndex((entry) => entry.undo_id === payload.undo_id)
          : snapshots.length - 1;
        if (index < 0) {
          return {
            restored: 0,
            undo_id: payload?.undo_id ?? ""
          } as R;
        }

        const [entry] = snapshots.splice(index, 1);
        writeStoredMergeUndoEntries(snapshots);
        writeStoredContacts(entry?.contacts_before ?? []);
        return {
          restored: entry?.contacts_before.length ?? 0,
          undo_id: entry?.undo_id ?? ""
        } as R;
      }
      case "config_snapshot": {
        return {
          mode: "dev",
          app: {
            mode: "dev"
          },
          logging: {
            directory: "logs",
            file_prefix: "rivet"
          },
          ui: {
            features: {
              contacts: true
            }
          }
        } as R;
      }
      case "config_apply_updates": {
        return {} as R;
      }
      case "external_calendar_cache_list": {
        return [] as R;
      }
      case "external_calendar_import_cached": {
        const payload = args as { source: ExternalCalendarSource; cache_id: string };
        return {
          calendar_id: payload.source.id,
          created: 0,
          updated: 0,
          deleted: 0,
          remote_events: 0,
          refresh_minutes: payload.source.refresh_minutes
        } as R;
      }
      case "tag_schema_snapshot": {
        return { version: 1, keys: [] } as R;
      }
      case "ui_log": {
        return undefined as R;
      }
      default:
        throw new Error(`unsupported mock command: ${command}`);
    }
  };

  let timeoutId: number | null = null;
  const timeout = new Promise<never>((_, reject) => {
    timeoutId = window.setTimeout(() => {
      reject(new Error(`invoke timeout (${command}) after ${timeoutMs}ms request_id=${requestId}`));
    }, timeoutMs);
  });

  try {
    const result = await Promise.race([run(), timeout]);
    const elapsed = Math.round((performance.now() - startedAt) * 100) / 100;
    if (instrumentCommand && verboseInvokeLogging) {
      logger.info("invoke.success", `${command} request_id=${requestId} duration_ms=${elapsed}`);
    }
    return result;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const elapsed = Math.round((performance.now() - startedAt) * 100) / 100;
    if (instrumentCommand) {
      logger.error("invoke.error", `${command} request_id=${requestId} duration_ms=${elapsed} error=${message}`);
      commandFailureSink?.({
        command,
        request_id: requestId,
        duration_ms: elapsed,
        error: message,
        timestamp: new Date().toISOString()
      });
    }
    throw error;
  } finally {
    if (timeoutId !== null) {
      window.clearTimeout(timeoutId);
    }
  }
}

setLoggerBridge(async (event, detail) => {
  try {
    await invokeCommand<void>("ui_log", { event, detail });
  } catch {
    // avoid recursive logger calls for logging failures.
  }
}, "warn");

export async function healthCheck(): Promise<void> {
  const response = await invokeCommand<unknown>("tasks_list", DEFAULT_TASK_QUERY);
  parseWithSchema("tasks_list healthcheck", response, TaskDtoArraySchema);
}

export async function listTasks(args: TasksListArgs = DEFAULT_TASK_QUERY): Promise<TaskDto[]> {
  const response = await invokeCommand<unknown>("tasks_list", args);
  return parseWithSchema("tasks_list response", response, TaskDtoArraySchema);
}

export async function addTask(args: TaskCreate): Promise<TaskDto> {
  logger.info("invoke.task_add", "adding task from React shell");
  const payload = parseWithSchema("task_add args", args, TaskCreateSchema);
  const response = await invokeCommand<unknown>("task_add", payload);
  return parseWithSchema("task_add response", response, TaskDtoSchema);
}

export async function updateTask(args: TaskUpdateArgs): Promise<TaskDto> {
  const payload = parseWithSchema("task_update args", args, TaskUpdateArgsSchema);
  const response = await invokeCommand<unknown>("task_update", payload);
  return parseWithSchema("task_update response", response, TaskDtoSchema);
}

export async function doneTask(uuid: string): Promise<TaskDto> {
  const response = await invokeCommand<unknown>("task_done", { uuid });
  return parseWithSchema("task_done response", response, TaskDtoSchema);
}

export async function uncompleteTask(uuid: string): Promise<TaskDto> {
  const response = await invokeCommand<unknown>("task_uncomplete", { uuid });
  return parseWithSchema("task_uncomplete response", response, TaskDtoSchema);
}

export async function deleteTask(uuid: string): Promise<void> {
  return invokeCommand<void>("task_delete", { uuid });
}

export async function listContacts(args: ContactsListArgs = DEFAULT_CONTACTS_QUERY): Promise<ContactsListResult> {
  const response = await invokeCommand<unknown>("contacts_list", args);
  return parseWithSchema("contacts_list response", response, ContactsListResultSchema);
}

export async function addContact(args: ContactCreate): Promise<ContactDto> {
  const payload = parseWithSchema("contact_add args", args, ContactCreateSchema);
  const response = await invokeCommand<unknown>("contact_add", payload);
  return parseWithSchema("contact_add response", response, ContactDtoSchema);
}

export async function updateContact(args: ContactUpdateArgs): Promise<ContactDto> {
  const payload = parseWithSchema("contact_update args", args, ContactUpdateArgsSchema);
  const response = await invokeCommand<unknown>("contact_update", payload);
  return parseWithSchema("contact_update response", response, ContactDtoSchema);
}

export async function deleteContact(id: string): Promise<void> {
  return invokeCommand<void>("contact_delete", { id });
}

export async function deleteContactsBulk(args: ContactsDeleteBulkArgs): Promise<number> {
  const response = await invokeCommand<unknown>("contacts_delete_bulk", args);
  return Number(response);
}

export async function previewContactsDedupe(args: ContactsDedupePreviewArgs): Promise<ContactsDedupePreviewResult> {
  const response = await invokeCommand<unknown>("contacts_dedupe_preview", args);
  return parseWithSchema("contacts_dedupe_preview response", response, ContactsDedupePreviewResultSchema);
}

export async function listContactsDedupeCandidates(args: ContactsDedupePreviewArgs): Promise<ContactsDedupePreviewResult> {
  const response = await invokeCommand<unknown>("contacts_dedupe_candidates", args);
  return parseWithSchema("contacts_dedupe_candidates response", response, ContactsDedupePreviewResultSchema);
}

export async function decideContactsDedupe(args: ContactsDedupeDecideArgs): Promise<ContactsDedupeDecideResult> {
  const response = await invokeCommand<unknown>("contacts_dedupe_decide", args);
  return parseWithSchema("contacts_dedupe_decide response", response, ContactsDedupeDecideResultSchema);
}

export async function openContactAction(args: ContactOpenActionArgs): Promise<ContactOpenActionResult> {
  const response = await invokeCommand<unknown>("contact_open_action", args);
  return parseWithSchema("contact_open_action response", response, ContactOpenActionResultSchema);
}

export async function previewContactsImport(args: ContactsImportPreviewArgs): Promise<ContactsImportPreviewResult> {
  const response = await invokeCommand<unknown>("contacts_import_preview", args);
  return parseWithSchema("contacts_import_preview response", response, ContactsImportPreviewResultSchema);
}

export async function commitContactsImport(args: ContactsImportCommitArgs): Promise<ContactsImportCommitResult> {
  const response = await invokeCommand<unknown>("contacts_import_commit", args);
  return parseWithSchema("contacts_import_commit response", response, ContactsImportCommitResultSchema);
}

export async function mergeContacts(args: ContactsMergeArgs): Promise<ContactsMergeResult> {
  const response = await invokeCommand<unknown>("contacts_merge", args);
  return parseWithSchema("contacts_merge response", response, ContactsMergeResultSchema);
}

export async function undoContactsMerge(args: ContactsMergeUndoArgs): Promise<ContactsMergeUndoResult> {
  const response = await invokeCommand<unknown>("contacts_merge_undo", args);
  return parseWithSchema("contacts_merge_undo response", response, ContactsMergeUndoResultSchema);
}

export async function syncExternalCalendar(source: ExternalCalendarSource): Promise<ExternalCalendarSyncResult> {
  const payload = parseWithSchema("external_calendar_sync args", source, ExternalCalendarSourceSchema);
  const response = await invokeCommand<unknown>("external_calendar_sync", payload);
  return parseWithSchema("external_calendar_sync response", response, ExternalCalendarSyncResultSchema);
}

export async function importExternalCalendarIcs(source: ExternalCalendarSource, icsText: string): Promise<ExternalCalendarSyncResult> {
  const payload = {
    source: parseWithSchema("external_calendar_import_ics args source", source, ExternalCalendarSourceSchema),
    ics_text: icsText
  };
  const response = await invokeCommand<unknown>("external_calendar_import_ics", payload);
  return parseWithSchema("external_calendar_import_ics response", response, ExternalCalendarSyncResultSchema);
}

export async function listExternalCalendarCache(): Promise<ExternalCalendarCacheEntry[]> {
  const response = await invokeCommand<unknown>("external_calendar_cache_list");
  return parseWithSchema("external_calendar_cache_list response", response, ExternalCalendarCacheEntryArraySchema);
}

export async function importExternalCalendarCached(source: ExternalCalendarSource, cacheId: string): Promise<ExternalCalendarSyncResult> {
  const payload = {
    source: parseWithSchema("external_calendar_import_cached args source", source, ExternalCalendarSourceSchema),
    cache_id: cacheId
  };
  const response = await invokeCommand<unknown>("external_calendar_import_cached", payload);
  return parseWithSchema("external_calendar_import_cached response", response, ExternalCalendarSyncResultSchema);
}

export async function loadConfigSnapshot(): Promise<RivetRuntimeConfig> {
  try {
    const response = await invokeCommand<unknown>("config_snapshot");
    return parseWithSchema("config_snapshot response", response, RivetRuntimeConfigSchema);
  } catch (error) {
    logger.warn("config_snapshot", String(error));
    return {};
  }
}

export async function applyConfigUpdates(updates: ConfigEntryUpdate[]): Promise<RivetRuntimeConfig> {
  const payload = {
    updates
  };
  const response = await invokeCommand<unknown>("config_apply_updates", payload);
  return parseWithSchema("config_apply_updates response", response, RivetRuntimeConfigSchema);
}

export async function loadTagSchemaSnapshot(): Promise<TagSchema> {
  try {
    const response = await invokeCommand<unknown>("tag_schema_snapshot");
    return parseWithSchema("tag_schema_snapshot response", response, TagSchemaSchema);
  } catch (error) {
    logger.warn("tag_schema_snapshot", String(error));
    return { version: 1, keys: [] };
  }
}
