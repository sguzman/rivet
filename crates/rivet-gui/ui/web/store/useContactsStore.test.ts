import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ContactDto } from "../types/core";

const mocks = vi.hoisted(() => ({
  addContactMock: vi.fn(),
  commitContactsImportMock: vi.fn(),
  decideContactsDedupeMock: vi.fn(),
  deleteContactMock: vi.fn(),
  deleteContactsBulkMock: vi.fn(),
  listContactsMock: vi.fn(),
  listContactsDedupeCandidatesMock: vi.fn(),
  mergeContactsMock: vi.fn(),
  openContactActionMock: vi.fn(),
  previewContactsImportMock: vi.fn(),
  undoContactsMergeMock: vi.fn(),
  updateContactMock: vi.fn()
}));

vi.mock("../api/tauri", () => ({
  addContact: mocks.addContactMock,
  commitContactsImport: mocks.commitContactsImportMock,
  decideContactsDedupe: mocks.decideContactsDedupeMock,
  deleteContact: mocks.deleteContactMock,
  deleteContactsBulk: mocks.deleteContactsBulkMock,
  listContacts: mocks.listContactsMock,
  listContactsDedupeCandidates: mocks.listContactsDedupeCandidatesMock,
  mergeContacts: mocks.mergeContactsMock,
  openContactAction: mocks.openContactActionMock,
  previewContactsImport: mocks.previewContactsImportMock,
  undoContactsMerge: mocks.undoContactsMergeMock,
  updateContact: mocks.updateContactMock
}));

import { emptyContactDraft, useContactsStore } from "./useContactsStore";

function sampleContact(id: string, name: string): ContactDto {
  const now = new Date().toISOString();
  return {
    id,
    display_name: name,
    avatar_data_url: null,
    import_batch_id: null,
    source_file_name: null,
    given_name: null,
    family_name: null,
    nickname: null,
    notes: null,
    phones: [{ value: "+1 555 0000", kind: "mobile", is_primary: true }],
    emails: [{ value: `${id}@example.com`, kind: "home", is_primary: true }],
    websites: [],
    birthday: null,
    organization: null,
    title: null,
    addresses: [],
    source_id: "local",
    source_kind: "local",
    remote_id: null,
    link_group_id: null,
    created_at: now,
    updated_at: now
  };
}

const initialState = useContactsStore.getState();

describe("useContactsStore", () => {
  beforeEach(() => {
    mocks.addContactMock.mockReset();
    mocks.commitContactsImportMock.mockReset();
    mocks.decideContactsDedupeMock.mockReset();
    mocks.deleteContactMock.mockReset();
    mocks.deleteContactsBulkMock.mockReset();
    mocks.listContactsMock.mockReset();
    mocks.listContactsDedupeCandidatesMock.mockReset();
    mocks.mergeContactsMock.mockReset();
    mocks.openContactActionMock.mockReset();
    mocks.previewContactsImportMock.mockReset();
    mocks.undoContactsMergeMock.mockReset();
    mocks.updateContactMock.mockReset();

    useContactsStore.setState(initialState, true);
  });

  it("loads first contacts page with selection", async () => {
    const contacts = [sampleContact("c-1", "Ada")];
    mocks.listContactsMock.mockResolvedValueOnce({
      contacts,
      next_cursor: null,
      total: 1
    });

    await useContactsStore.getState().loadContacts();

    const state = useContactsStore.getState();
    expect(state.contacts).toEqual(contacts);
    expect(state.selectedContactId).toBe("c-1");
    expect(state.total).toBe(1);
    expect(mocks.listContactsMock).toHaveBeenCalledWith({
      query: null,
      limit: 200,
      cursor: null,
      source: null,
      updated_after: null
    });
  });

  it("uses query cache for repeated query and source", async () => {
    const contacts = [sampleContact("c-1", "Ada")];
    mocks.listContactsMock.mockResolvedValue({
      contacts,
      next_cursor: null,
      total: 1
    });

    await useContactsStore.getState().loadContacts();
    await useContactsStore.getState().loadContacts();

    expect(mocks.listContactsMock).toHaveBeenCalledTimes(1);
  });

  it("ignores stale in-flight responses", async () => {
    let resolveFirst!: (value: { contacts: ContactDto[]; next_cursor: string | null; total: number }) => void;
    const firstPromise = new Promise<{ contacts: ContactDto[]; next_cursor: string | null; total: number }>((resolve) => {
      resolveFirst = resolve;
    });

    mocks.listContactsMock
      .mockReturnValueOnce(firstPromise)
      .mockResolvedValueOnce({
        contacts: [sampleContact("c-2", "Zoe")],
        next_cursor: null,
        total: 1
      });

    const first = useContactsStore.getState().setQuery("a");
    const second = useContactsStore.getState().setQuery("zoe");

    await second;
    resolveFirst({
      contacts: [sampleContact("c-1", "Ada")],
      next_cursor: null,
      total: 1
    });
    await first;

    expect(useContactsStore.getState().contacts[0]?.display_name).toBe("Zoe");
  });

  it("creates from draft and resets dirty state", async () => {
    const created = sampleContact("new-1", "New Contact");
    mocks.addContactMock.mockResolvedValueOnce(created);

    useContactsStore.setState({
      formDraft: {
        ...emptyContactDraft(),
        display_name: "New Contact"
      },
      dirty: true
    });

    await useContactsStore.getState().createFromDraft();

    const state = useContactsStore.getState();
    expect(state.contacts[0]?.id).toBe("new-1");
    expect(state.dirty).toBe(false);
    expect(state.editorMode).toBe("add");
  });

  it("records dedupe decisions and refreshes dedupe list", async () => {
    mocks.decideContactsDedupeMock.mockResolvedValueOnce({
      candidate_group_id: "group:1,2",
      decision: "ignored",
      actor: "user",
      decided_at: new Date().toISOString()
    });
    mocks.listContactsDedupeCandidatesMock.mockResolvedValueOnce({ groups: [] });

    await useContactsStore.getState().decideDedupeGroup("group:1,2", "ignored");

    expect(mocks.decideContactsDedupeMock).toHaveBeenCalledWith({
      candidate_group_id: "group:1,2",
      decision: "ignored",
      actor: "user"
    });
    expect(mocks.listContactsDedupeCandidatesMock).toHaveBeenCalled();
  });
});
