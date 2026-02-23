import { describe, expect, it } from "vitest";

import {
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
