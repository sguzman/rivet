import { create } from "zustand";

import {
  addContact,
  commitContactsImport,
  decideContactsDedupe,
  deleteContact,
  deleteContactsBulk,
  listContacts,
  listContactsDedupeCandidates,
  mergeContacts,
  openContactAction,
  previewContactsImport,
  undoContactsMerge,
  updateContact
} from "../api/tauri";
import { logger } from "../lib/logger";
import type {
  ContactAddress,
  ContactCreate,
  ContactDto,
  ContactFieldValue,
  ContactOpenActionArgs,
  ContactPatch,
  ContactsDedupePreviewResult,
  ContactsImportCommitResult,
  ContactsImportPreviewResult,
  ContactsMergeUndoResult
} from "../types/core";

const CONTACTS_PAGE_SIZE = 200;
const CONTACTS_QUERY_CACHE_LIMIT = 24;

type ContactsLoadOptions = {
  append?: boolean;
  force?: boolean;
};

function emptyField(kind: string): ContactFieldValue {
  return {
    value: "",
    kind,
    is_primary: false
  };
}

function emptyAddress(kind: string): ContactAddress {
  return {
    kind,
    street: "",
    city: "",
    region: "",
    postal_code: "",
    country: ""
  };
}

export function emptyContactDraft(): ContactCreate {
  return {
    display_name: "",
    avatar_data_url: null,
    import_batch_id: null,
    source_file_name: null,
    given_name: "",
    family_name: "",
    nickname: "",
    notes: "",
    phones: [emptyField("mobile")],
    emails: [emptyField("home")],
    websites: [],
    birthday: null,
    organization: "",
    title: "",
    addresses: [emptyAddress("home")],
    source_id: "local",
    source_kind: "local",
    remote_id: null,
    link_group_id: null
  };
}

function cleanFieldValues(fields: ContactFieldValue[]): ContactFieldValue[] {
  return fields
    .map((field) => ({
      value: field.value.trim(),
      kind: field.kind.trim() || "other",
      is_primary: field.is_primary
    }))
    .filter((field) => field.value.length > 0)
    .map((field, index, all) => ({
      ...field,
      is_primary: all.some((item) => item.is_primary) ? field.is_primary : index === 0
    }));
}

function normalizeDraft(input: ContactCreate): ContactCreate {
  const addresses = input.addresses
    .map((address) => ({
      kind: address.kind?.trim() || "home",
      street: address.street?.trim() ?? "",
      city: address.city?.trim() ?? "",
      region: address.region?.trim() ?? "",
      postal_code: address.postal_code?.trim() ?? "",
      country: address.country?.trim() ?? ""
    }))
    .filter((address) => address.street.length > 0 || address.country.length > 0);

  return {
    ...input,
    display_name: input.display_name?.trim() ?? "",
    avatar_data_url: input.avatar_data_url?.trim() || null,
    import_batch_id: input.import_batch_id?.trim() || null,
    source_file_name: input.source_file_name?.trim() || null,
    given_name: input.given_name?.trim() ?? "",
    family_name: input.family_name?.trim() ?? "",
    nickname: input.nickname?.trim() ?? "",
    notes: input.notes?.trim() ?? "",
    phones: cleanFieldValues(input.phones),
    emails: cleanFieldValues(input.emails),
    websites: cleanFieldValues(input.websites),
    organization: input.organization?.trim() ?? "",
    title: input.title?.trim() ?? "",
    addresses,
    source_id: input.source_id?.trim() || "local",
    source_kind: input.source_kind?.trim() || "local",
    remote_id: input.remote_id?.trim() || null,
    link_group_id: input.link_group_id?.trim() || null
  };
}

function queryCacheKey(query: string, sourceFilter: string | null): string {
  return `${query.trim()}::${sourceFilter ?? "all"}`;
}

function mergeContactPages(previous: ContactDto[], next: ContactDto[]): ContactDto[] {
  const seen = new Set(previous.map((item) => item.id));
  const merged = [...previous];
  for (const contact of next) {
    if (!seen.has(contact.id)) {
      merged.push(contact);
      seen.add(contact.id);
    }
  }
  return merged;
}

interface ContactsStore {
  loading: boolean;
  error: string | null;
  query: string;
  sourceFilter: string | null;
  contacts: ContactDto[];
  selectedContactId: string | null;
  selectionMode: boolean;
  selectionIds: string[];
  total: number;
  nextCursor: string | null;
  dedupe: ContactsDedupePreviewResult | null;
  importPreview: ContactsImportPreviewResult | null;
  importCommitResult: ContactsImportCommitResult | null;
  importSource: string | null;
  importFileName: string | null;
  importContent: string | null;
  mergeUndoResult: ContactsMergeUndoResult | null;
  lastMergeUndoId: string | null;

  formDraft: ContactCreate;
  dirty: boolean;
  editorMode: "add" | "edit";

  loadToken: number;
  queryCache: Record<string, { contacts: ContactDto[]; nextCursor: string | null; total: number }>;
  queryCacheOrder: string[];

  bootstrap: () => Promise<void>;
  loadContacts: (options?: ContactsLoadOptions) => Promise<void>;
  loadMoreContacts: () => Promise<void>;
  setQuery: (value: string) => Promise<void>;
  setSourceFilter: (value: string | null) => Promise<void>;

  selectContact: (id: string | null) => void;
  toggleSelectionMode: () => void;
  toggleSelected: (id: string) => void;
  setSelectionIds: (ids: string[]) => void;
  clearSelection: () => void;

  setDraft: (draft: ContactCreate) => void;
  resetDraft: () => void;
  beginAddNew: () => void;
  loadDraftFromSelected: () => void;

  createFromDraft: () => Promise<void>;
  updateSelectedFromDraft: () => Promise<void>;
  removeSelectedContact: () => Promise<void>;
  removeBulkSelected: () => Promise<void>;

  refreshDedupe: () => Promise<void>;
  linkSelectedContacts: () => Promise<void>;
  unlinkSelectedContacts: () => Promise<void>;
  mergeSelected: () => Promise<void>;
  undoLastMerge: () => Promise<void>;
  decideDedupeGroup: (groupId: string, decision: "ignored" | "separate") => Promise<void>;

  previewImport: (source: string, fileName: string | null, content: string) => Promise<void>;
  commitImport: (mode: "safe" | "upsert" | "review") => Promise<void>;

  openAction: (args: ContactOpenActionArgs) => Promise<void>;
}

export const useContactsStore = create<ContactsStore>((set, get) => ({
  loading: false,
  error: null,
  query: "",
  sourceFilter: null,
  contacts: [],
  selectedContactId: null,
  selectionMode: false,
  selectionIds: [],
  total: 0,
  nextCursor: null,
  dedupe: null,
  importPreview: null,
  importCommitResult: null,
  importSource: null,
  importFileName: null,
  importContent: null,
  mergeUndoResult: null,
  lastMergeUndoId: null,

  formDraft: emptyContactDraft(),
  dirty: false,
  editorMode: "add",

  loadToken: 0,
  queryCache: {},
  queryCacheOrder: [],

  async bootstrap() {
    if (get().contacts.length > 0) {
      return;
    }
    await get().loadContacts();
  },

  async loadContacts(options) {
    const append = options?.append ?? false;
    const force = options?.force ?? false;
    const state = get();

    if (!append && !force) {
      const cached = state.queryCache[queryCacheKey(state.query, state.sourceFilter)];
      if (cached) {
        set((current) => ({
          contacts: cached.contacts,
          total: cached.total,
          nextCursor: cached.nextCursor,
          selectedContactId: current.selectedContactId && cached.contacts.some((contact) => contact.id === current.selectedContactId)
            ? current.selectedContactId
            : cached.contacts[0]?.id ?? null,
          selectionIds: current.selectionIds.filter((id) => cached.contacts.some((contact) => contact.id === id)),
          loading: false,
          error: null
        }));
        return;
      }
    }

    const token = state.loadToken + 1;
    const cursor = append ? state.nextCursor : null;
    if (append && !cursor) {
      return;
    }

    set({ loading: true, error: null, loadToken: token });
    const startedAt = performance.now();

    try {
      const result = await listContacts({
        query: get().query || null,
        limit: CONTACTS_PAGE_SIZE,
        cursor,
        source: get().sourceFilter,
        updated_after: null
      });

      if (get().loadToken !== token) {
        return;
      }

      set((current) => {
        const contacts = append
          ? mergeContactPages(current.contacts, result.contacts)
          : result.contacts;
        const selectedContactId = current.selectedContactId && contacts.some((contact) => contact.id === current.selectedContactId)
          ? current.selectedContactId
          : contacts[0]?.id ?? null;
        const selectionIds = current.selectionIds.filter((id) => contacts.some((contact) => contact.id === id));

        const nextState: Partial<ContactsStore> = {
          loading: false,
          contacts,
          total: result.total,
          nextCursor: result.next_cursor,
          selectedContactId,
          selectionIds
        };

        if (!append) {
          const cacheKey = queryCacheKey(current.query, current.sourceFilter);
          const queryCache = {
            ...current.queryCache,
            [cacheKey]: {
              contacts,
              total: result.total,
              nextCursor: result.next_cursor
            }
          };
          const nextOrder = [
            cacheKey,
            ...current.queryCacheOrder.filter((item) => item !== cacheKey)
          ].slice(0, CONTACTS_QUERY_CACHE_LIMIT);
          for (const staleKey of Object.keys(queryCache)) {
            if (!nextOrder.includes(staleKey)) {
              delete queryCache[staleKey];
            }
          }
          nextState.queryCache = queryCache;
          nextState.queryCacheOrder = nextOrder;
        }

        return nextState as ContactsStore;
      });

      const durationMs = performance.now() - startedAt;
      if (durationMs > 100) {
        logger.warn("contacts.perf.query", `query_ms=${durationMs.toFixed(2)} query=${get().query.trim()} source=${get().sourceFilter ?? "all"}`);
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (get().loadToken === token) {
        set({ loading: false, error: message });
      }
      logger.error("contacts.load.error", message);
    }
  },

  async loadMoreContacts() {
    await get().loadContacts({ append: true });
  },

  async setQuery(value) {
    set({ query: value });
    await get().loadContacts();
  },

  async setSourceFilter(value) {
    set({ sourceFilter: value });
    await get().loadContacts();
  },

  selectContact(id) {
    set({
      selectedContactId: id,
      editorMode: id ? "edit" : "add"
    });
    if (id) {
      get().loadDraftFromSelected();
    }
  },

  toggleSelectionMode() {
    set((state) => ({
      selectionMode: !state.selectionMode,
      selectionIds: state.selectionMode ? [] : state.selectionIds
    }));
  },

  toggleSelected(id) {
    set((state) => {
      if (state.selectionIds.includes(id)) {
        return {
          selectionIds: state.selectionIds.filter((entry) => entry !== id)
        };
      }
      return {
        selectionIds: [...state.selectionIds, id]
      };
    });
  },

  setSelectionIds(ids) {
    set({ selectionIds: [...new Set(ids)] });
  },

  clearSelection() {
    set({ selectionIds: [] });
  },

  setDraft(formDraft) {
    set({ formDraft, dirty: true });
  },

  resetDraft() {
    set({ formDraft: emptyContactDraft(), dirty: false, editorMode: "add" });
  },

  beginAddNew() {
    set({
      selectedContactId: null,
      formDraft: emptyContactDraft(),
      dirty: false,
      editorMode: "add"
    });
  },

  loadDraftFromSelected() {
    const selected = get().contacts.find((contact) => contact.id === get().selectedContactId);
    if (!selected) {
      return;
    }

    set({
      formDraft: {
        display_name: selected.display_name,
        avatar_data_url: selected.avatar_data_url,
        import_batch_id: selected.import_batch_id,
        source_file_name: selected.source_file_name,
        given_name: selected.given_name,
        family_name: selected.family_name,
        nickname: selected.nickname,
        notes: selected.notes,
        phones: selected.phones.length > 0 ? selected.phones : [emptyField("mobile")],
        emails: selected.emails.length > 0 ? selected.emails : [emptyField("home")],
        websites: selected.websites,
        birthday: selected.birthday,
        organization: selected.organization,
        title: selected.title,
        addresses: selected.addresses.length > 0 ? selected.addresses : [emptyAddress("home")],
        source_id: selected.source_id,
        source_kind: selected.source_kind,
        remote_id: selected.remote_id,
        link_group_id: selected.link_group_id
      },
      dirty: false,
      editorMode: "edit"
    });
  },

  async createFromDraft() {
    set({ loading: true, error: null });
    try {
      const created = await addContact(normalizeDraft(get().formDraft));
      const cacheKey = queryCacheKey(get().query, get().sourceFilter);
      set((state) => ({
        loading: false,
        contacts: [created, ...state.contacts],
        selectedContactId: created.id,
        formDraft: emptyContactDraft(),
        dirty: false,
        editorMode: "add",
        queryCache: {
          ...state.queryCache,
          [cacheKey]: {
            contacts: [created, ...state.contacts],
            total: state.total + 1,
            nextCursor: state.nextCursor
          }
        }
      }));
      await get().refreshDedupe();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },

  async updateSelectedFromDraft() {
    const selectedId = get().selectedContactId;
    if (!selectedId) {
      return;
    }

    set({ loading: true, error: null });
    try {
      const draft = normalizeDraft(get().formDraft);
      const patch: ContactPatch = {
        display_name: draft.display_name,
        avatar_data_url: draft.avatar_data_url,
        import_batch_id: draft.import_batch_id,
        source_file_name: draft.source_file_name,
        given_name: draft.given_name,
        family_name: draft.family_name,
        nickname: draft.nickname,
        notes: draft.notes,
        phones: draft.phones,
        emails: draft.emails,
        websites: draft.websites,
        birthday: draft.birthday,
        organization: draft.organization,
        title: draft.title,
        addresses: draft.addresses,
        source_id: draft.source_id,
        source_kind: draft.source_kind,
        remote_id: draft.remote_id,
        link_group_id: draft.link_group_id
      };

      const updated = await updateContact({ id: selectedId, patch });
      set((state) => ({
        loading: false,
        contacts: state.contacts.map((contact) => (contact.id === updated.id ? updated : contact)),
        dirty: false
      }));
      await get().refreshDedupe();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },

  async removeSelectedContact() {
    const selectedId = get().selectedContactId;
    if (!selectedId) {
      return;
    }

    set({ loading: true, error: null });
    try {
      await deleteContact(selectedId);
      set((state) => {
        const contacts = state.contacts.filter((contact) => contact.id !== selectedId);
        return {
          loading: false,
          contacts,
          selectedContactId: contacts[0]?.id ?? null,
          selectionIds: state.selectionIds.filter((id) => id !== selectedId),
          editorMode: contacts.length > 0 ? "edit" : "add"
        };
      });
      if (get().selectedContactId) {
        get().loadDraftFromSelected();
      } else {
        get().beginAddNew();
      }
      await get().refreshDedupe();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },

  async removeBulkSelected() {
    const ids = [...new Set(get().selectionIds)];
    if (ids.length === 0) {
      return;
    }

    set({ loading: true, error: null });
    try {
      await deleteContactsBulk({ ids });
      set((state) => {
        const contacts = state.contacts.filter((contact) => !ids.includes(contact.id));
        return {
          loading: false,
          contacts,
          selectedContactId: contacts[0]?.id ?? null,
          selectionIds: [],
          editorMode: contacts.length > 0 ? "edit" : "add"
        };
      });
      if (get().selectedContactId) {
        get().loadDraftFromSelected();
      } else {
        get().beginAddNew();
      }
      await get().refreshDedupe();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },

  async refreshDedupe() {
    try {
      const dedupe = await listContactsDedupeCandidates({ query: get().query || null });
      set({ dedupe });
    } catch (error) {
      logger.warn("contacts.dedupe.refresh", String(error));
    }
  },

  async linkSelectedContacts() {
    const ids = [...new Set(get().selectionIds)];
    if (ids.length < 2) {
      return;
    }
    set({ loading: true, error: null });
    try {
      const groupId = crypto.randomUUID();
      for (const id of ids) {
        await updateContact({
          id,
          patch: {
            link_group_id: groupId
          }
        });
      }
      set({ loading: false });
      await get().loadContacts({ force: true });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },

  async unlinkSelectedContacts() {
    const rawIds = get().selectionIds.length > 0
      ? get().selectionIds
      : (get().selectedContactId ? [get().selectedContactId] : []);
    const ids = [...new Set(rawIds.filter((id): id is string => typeof id === "string" && id.length > 0))];
    if (ids.length === 0) {
      return;
    }
    set({ loading: true, error: null });
    try {
      for (const id of ids) {
        await updateContact({
          id,
          patch: {
            link_group_id: null
          }
        });
      }
      set({ loading: false });
      await get().loadContacts({ force: true });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },

  async mergeSelected() {
    const ids = [...new Set(get().selectionIds)];
    if (ids.length < 2) {
      return;
    }

    set({ loading: true, error: null });
    try {
      const result = await mergeContacts({
        ids,
        target_id: get().selectedContactId
      });
      set((state) => ({
        loading: false,
        contacts: state.contacts
          .filter((contact) => !result.removed_ids.includes(contact.id))
          .map((contact) => (contact.id === result.merged.id ? result.merged : contact)),
        selectedContactId: result.merged.id,
        selectionIds: [result.merged.id],
        lastMergeUndoId: result.undo_id,
        editorMode: "edit"
      }));
      get().loadDraftFromSelected();
      await get().refreshDedupe();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },

  async undoLastMerge() {
    const undoId = get().lastMergeUndoId;
    if (!undoId) {
      return;
    }

    set({ loading: true, error: null });
    try {
      const result = await undoContactsMerge({ undo_id: undoId });
      set({
        loading: false,
        mergeUndoResult: result,
        lastMergeUndoId: null
      });
      await get().loadContacts({ force: true });
      await get().refreshDedupe();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },

  async decideDedupeGroup(groupId, decision) {
    try {
      await decideContactsDedupe({
        candidate_group_id: groupId,
        decision,
        actor: "user"
      });
      await get().refreshDedupe();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ error: message });
    }
  },

  async previewImport(source, fileName, content) {
    set({ loading: true, error: null, importCommitResult: null });
    try {
      const preview = await previewContactsImport({ source, file_name: fileName, content });
      set({
        loading: false,
        importPreview: preview,
        importSource: source,
        importFileName: fileName,
        importContent: content
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },

  async commitImport(mode) {
    const source = get().importSource;
    const fileName = get().importFileName;
    const content = get().importContent;
    if (!source || !content) {
      return;
    }

    set({ loading: true, error: null });
    try {
      const result = await commitContactsImport({
        source,
        file_name: fileName,
        content,
        mode
      });
      set({
        loading: false,
        importCommitResult: result
      });
      await get().loadContacts({ force: true });
      await get().refreshDedupe();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
    }
  },

  async openAction(args) {
    try {
      const result = await openContactAction(args);
      if (result.url && typeof window !== "undefined") {
        window.open(result.url, "_blank", "noopener,noreferrer");
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ error: message });
    }
  }
}));
