import { describe, expect, it } from "vitest";

import {
  ContactCreateSchema,
  ContactDtoSchema,
  ContactUpdateArgsSchema,
  ContactsDedupeDecideResultSchema,
  ContactsDedupePreviewResultSchema,
  ContactsImportCommitResultSchema,
  ContactsImportPreviewResultSchema,
  ContactsListResultSchema,
  ContactsMergeResultSchema,
  ContactsMergeUndoResultSchema,
  ExternalCalendarSourceSchema,
  ExternalCalendarSyncResultSchema,
  RivetRuntimeConfigSchema,
  TagSchemaSchema,
  TaskCreateSchema,
  TaskDtoArraySchema,
  TaskDtoSchema,
  TaskUpdateArgsSchema
} from "./schemas";

describe("tauri command contract schemas", () => {
  it("accepts canonical task payloads", () => {
    const task = {
      uuid: "0f84cb8d-6239-4ae4-9f89-3680af7bd836",
      id: 17,
      title: "Ship migration checklist",
      description: "Validate task/kanban/calendar parity",
      status: "Pending",
      project: "rivet/gui",
      tags: ["kanban:todo", "board:default"],
      priority: "High",
      due: "2026-03-01T11:00:00Z",
      wait: null,
      scheduled: null,
      created: "2026-02-20T10:11:12Z",
      modified: "2026-02-20T10:11:12Z"
    };

    expect(TaskDtoSchema.parse(task)).toEqual(task);
    expect(TaskDtoArraySchema.parse([task])).toEqual([task]);
  });

  it("rejects invalid task status values", () => {
    const result = TaskDtoSchema.safeParse({
      uuid: "a",
      id: null,
      title: "bad",
      description: "",
      status: "Open",
      project: null,
      tags: [],
      priority: null,
      due: null,
      wait: null,
      scheduled: null,
      created: null,
      modified: null
    });

    expect(result.success).toBe(false);
  });

  it("accepts task create/update args", () => {
    const createArgs = {
      title: "Create typed API tests",
      description: "",
      project: null,
      tags: ["qa:contracts"],
      priority: null,
      due: null,
      wait: null,
      scheduled: null
    };
    const updateArgs = {
      uuid: "update-uuid",
      patch: {
        title: "Updated title",
        due: "2026-03-01T00:00:00Z"
      }
    };

    expect(TaskCreateSchema.parse(createArgs)).toEqual(createArgs);
    expect(TaskUpdateArgsSchema.parse(updateArgs)).toEqual(updateArgs);
  });

  it("accepts external calendar source and sync result", () => {
    const source = {
      id: "cal-nyse",
      name: "Nyse Holidays",
      color: "#00c2c7",
      location: "webcal://example.com/nyse.ics",
      refresh_minutes: 30,
      enabled: true,
      imported_ics_file: false,
      read_only: true,
      show_reminders: true,
      offline_support: true
    };
    const result = {
      calendar_id: "cal-nyse",
      created: 2,
      updated: 1,
      deleted: 0,
      remote_events: 3,
      refresh_minutes: 30
    };

    expect(ExternalCalendarSourceSchema.parse(source)).toEqual(source);
    expect(ExternalCalendarSyncResultSchema.parse(result)).toEqual(result);
  });

  it("accepts contacts schemas including import/merge payloads", () => {
    const contact = {
      id: "7cf58c36-4ed7-4893-ab89-d7c3c1dbf0ff",
      display_name: "Alex Morgan",
      avatar_data_url: null,
      import_batch_id: null,
      source_file_name: "contacts.vcf",
      given_name: "Alex",
      family_name: "Morgan",
      nickname: null,
      notes: null,
      phones: [{ value: "+1 555 0100", kind: "mobile", is_primary: true }],
      emails: [{ value: "alex@example.com", kind: "home", is_primary: true }],
      websites: [],
      birthday: null,
      organization: "Rivet",
      title: "PM",
      addresses: [{ kind: "home", street: "1 Main", city: "SF", region: "CA", postal_code: "94105", country: "United States" }],
      source_id: "local",
      source_kind: "local",
      remote_id: null,
      link_group_id: null,
      created_at: "2026-02-26T00:00:00Z",
      updated_at: "2026-02-26T00:00:00Z"
    };

    expect(ContactDtoSchema.parse(contact)).toEqual(contact);
    expect(ContactsListResultSchema.parse({ contacts: [contact], next_cursor: null, total: 1 })).toEqual({ contacts: [contact], next_cursor: null, total: 1 });
    expect(ContactCreateSchema.parse({
      display_name: "Alex Morgan",
      avatar_data_url: null,
      import_batch_id: null,
      source_file_name: "contacts.vcf",
      given_name: "Alex",
      family_name: "Morgan",
      nickname: null,
      notes: null,
      phones: [{ value: "+1 555 0100", kind: "mobile", is_primary: true }],
      emails: [{ value: "alex@example.com", kind: "home", is_primary: true }],
      websites: [],
      birthday: null,
      organization: null,
      title: null,
      addresses: [],
      source_id: "local",
      source_kind: "local",
      remote_id: null,
      link_group_id: null
    })).toBeTruthy();
    expect(ContactUpdateArgsSchema.parse({ id: contact.id, patch: { source_file_name: "import.vcf", import_batch_id: "batch-1" } })).toEqual({
      id: contact.id,
      patch: { source_file_name: "import.vcf", import_batch_id: "batch-1" }
    });
    expect(ContactsDedupePreviewResultSchema.parse({ groups: [{ group_id: "g1", reason: "exact email match", score: 100, contacts: [contact] }] })).toBeTruthy();
    expect(ContactsDedupeDecideResultSchema.parse({
      candidate_group_id: "g1",
      decision: "ignored",
      actor: "user",
      decided_at: "2026-02-26T01:00:00Z"
    })).toBeTruthy();
    expect(ContactsImportPreviewResultSchema.parse({
      batch_id: "batch-1",
      source: "gmail_file",
      total_rows: 1,
      valid_rows: 1,
      skipped_rows: 0,
      potential_duplicates: 0,
      contacts: [contact],
      conflicts: [],
      errors: []
    })).toBeTruthy();
    expect(ContactsImportCommitResultSchema.parse({
      batch_id: "batch-1",
      created: 1,
      updated: 0,
      skipped: 0,
      failed: 0,
      conflicts: 0,
      errors: []
    })).toBeTruthy();
    expect(ContactsMergeResultSchema.parse({
      merged: contact,
      removed_ids: [],
      undo_id: "undo-1"
    })).toBeTruthy();
    expect(ContactsMergeUndoResultSchema.parse({
      restored: 1,
      undo_id: "undo-1"
    })).toBeTruthy();
  });

  it("accepts runtime config and tag schema passthrough fields", () => {
    const config = {
      version: 1,
      mode: "dev",
      app: {
        mode: "dev",
        custom_extra: "ok"
      },
      calendar: {
        timezone: "America/Mexico_City",
        policies: {
          week_start: "monday",
          red_dot_limit: 8
        },
        visibility: {
          pending: true,
          completed: false
        }
      },
      custom_root: {
        enabled: true
      }
    };
    const tags = {
      version: 2,
      keys: [
        {
          id: "kanban",
          selection: "single",
          color: "#ff6b6b",
          values: ["todo", "working_on_it", "finished"]
        }
      ],
      custom_tag_field: true
    };

    expect(RivetRuntimeConfigSchema.parse(config)).toMatchObject(config);
    expect(TagSchemaSchema.parse(tags)).toMatchObject(tags);
  });
});
