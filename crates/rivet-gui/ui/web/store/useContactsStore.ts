import { create } from "zustand";

import {
  addContact,
  commitContactsImport,
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
  ContactCreate,
  ContactDto,
  ContactPatch,
  ContactsImportCommitResult,
  ContactsImportPreviewResult,
  ContactsMergeUndoResult,
  ContactOpenActionArgs,
  ContactFieldValue,
  ContactsDedupePreviewResult
} from "../types/core";

function emptyField(kind: string): ContactFieldValue {
  return {
    value: "",
    kind,
    is_primary: false
  };
}

export function emptyContactDraft(): ContactCreate {
  return {
    display_name: "",
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
    addresses: [],
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
    .filter((field) => field.value.length > 0);
}

function normalizeDraft(input: ContactCreate): ContactCreate {
  return {
    ...input,
    display_name: input.display_name?.trim() ?? "",
    given_name: input.given_name?.trim() ?? "",
    family_name: input.family_name?.trim() ?? "",
    nickname: input.nickname?.trim() ?? "",
    notes: input.notes?.trim() ?? "",
    phones: cleanFieldValues(input.phones),
    emails: cleanFieldValues(input.emails),
    websites: cleanFieldValues(input.websites),
    organization: input.organization?.trim() ?? "",
    title: input.title?.trim() ?? "",
    source_id: input.source_id?.trim() || "local",
    source_kind: input.source_kind?.trim() || "local",
    remote_id: input.remote_id?.trim() || null,
    link_group_id: input.link_group_id?.trim() || null
  };
}

interface ContactsStore {
  loading: boolean;
  error: string | null;
  query: string;
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
  draft: ContactCreate;

  bootstrap: () => Promise<void>;
  loadContacts: () => Promise<void>;
  setQuery: (value: string) => Promise<void>;
  selectContact: (id: string | null) => void;
  toggleSelectionMode: () => void;
  toggleSelected: (id: string) => void;
  setSelectionIds: (ids: string[]) => void;
  clearSelection: () => void;

  setDraft: (draft: ContactCreate) => void;
  resetDraft: () => void;
  loadDraftFromSelected: () => void;

  createFromDraft: () => Promise<void>;
  updateSelectedFromDraft: () => Promise<void>;
  removeSelectedContact: () => Promise<void>;
  removeBulkSelected: () => Promise<void>;

  refreshDedupe: () => Promise<void>;
  mergeSelected: () => Promise<void>;
  undoLastMerge: () => Promise<void>;

  previewImport: (source: string, fileName: string | null, content: string) => Promise<void>;
  commitImport: (mode: "safe" | "upsert" | "review") => Promise<void>;

  openAction: (args: ContactOpenActionArgs) => Promise<void>;
}

export const useContactsStore = create<ContactsStore>((set, get) => ({
  loading: false,
  error: null,
  query: "",
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
  draft: emptyContactDraft(),

  async bootstrap() {
    if (get().contacts.length > 0) {
      return;
    }
    await get().loadContacts();
  },

  async loadContacts() {
    set({ loading: true, error: null });
    try {
      const result = await listContacts({
        query: get().query || null,
        limit: 500,
        cursor: null,
        source: null,
        updated_after: null
      });
      set((state) => {
        const selectedContactId = state.selectedContactId && result.contacts.some((contact) => contact.id === state.selectedContactId)
          ? state.selectedContactId
          : result.contacts[0]?.id ?? null;
        const selectionIds = state.selectionIds.filter((id) => result.contacts.some((contact) => contact.id === id));
        return {
          loading: false,
          contacts: result.contacts,
          total: result.total,
          nextCursor: result.next_cursor,
          selectedContactId,
          selectionIds
        };
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
      logger.error("contacts.load.error", message);
    }
  },

  async setQuery(value) {
    set({ query: value });
    await get().loadContacts();
  },

  selectContact(id) {
    set({ selectedContactId: id });
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

  setDraft(draft) {
    set({ draft });
  },

  resetDraft() {
    set({ draft: emptyContactDraft() });
  },

  loadDraftFromSelected() {
    const selected = get().contacts.find((contact) => contact.id === get().selectedContactId);
    if (!selected) {
      return;
    }
    set({
      draft: {
        display_name: selected.display_name,
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
        addresses: selected.addresses,
        source_id: selected.source_id,
        source_kind: selected.source_kind,
        remote_id: selected.remote_id,
        link_group_id: selected.link_group_id
      }
    });
  },

  async createFromDraft() {
    set({ loading: true, error: null });
    try {
      const created = await addContact(normalizeDraft(get().draft));
      set((state) => ({
        loading: false,
        contacts: [created, ...state.contacts],
        selectedContactId: created.id,
        draft: emptyContactDraft()
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
      const draft = normalizeDraft(get().draft);
      const patch: ContactPatch = {
        display_name: draft.display_name,
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
        contacts: state.contacts.map((contact) => (contact.id === updated.id ? updated : contact))
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
          selectionIds: state.selectionIds.filter((id) => id !== selectedId)
        };
      });
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
          selectionIds: []
        };
      });
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
        lastMergeUndoId: result.undo_id
      }));
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
      await get().loadContacts();
      await get().refreshDedupe();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ loading: false, error: message });
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
      await get().loadContacts();
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
