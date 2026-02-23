import { formatDueDateTime, parseTaskDueUtcMs } from "./calendar";
import type { TaskDto } from "../types/core";

export type DueNotificationPermission = "default" | "granted" | "denied" | "unsupported";

export interface DueNotificationConfig {
  enabled: boolean;
  pre_notify_enabled: boolean;
  pre_notify_minutes: number;
}

export interface DueNotificationEvent {
  key: string;
  title: string;
  body: string;
}

export function defaultDueNotificationConfig(): DueNotificationConfig {
  return {
    enabled: false,
    pre_notify_enabled: false,
    pre_notify_minutes: 15
  };
}

export function sanitizeDueNotificationConfig(config: Partial<DueNotificationConfig> | null | undefined): DueNotificationConfig {
  const base = defaultDueNotificationConfig();
  const minutes = Number(config?.pre_notify_minutes ?? base.pre_notify_minutes);
  const boundedMinutes = Number.isFinite(minutes) ? Math.max(1, Math.min(43_200, Math.floor(minutes))) : base.pre_notify_minutes;
  return {
    enabled: Boolean(config?.enabled ?? base.enabled),
    pre_notify_enabled: Boolean(config?.pre_notify_enabled ?? base.pre_notify_enabled),
    pre_notify_minutes: boundedMinutes
  };
}

export function browserDueNotificationPermission(): DueNotificationPermission {
  if (typeof window === "undefined" || typeof Notification === "undefined") {
    return "unsupported";
  }

  const permission = Notification.permission;
  if (permission === "granted" || permission === "denied" || permission === "default") {
    return permission;
  }
  return "unsupported";
}

export async function requestDueNotificationPermission(): Promise<DueNotificationPermission> {
  if (typeof window === "undefined" || typeof Notification === "undefined") {
    return "unsupported";
  }
  try {
    await Notification.requestPermission();
  } catch {
    // best effort
  }
  return browserDueNotificationPermission();
}

export function emitDueNotification(title: string, body: string): boolean {
  if (browserDueNotificationPermission() !== "granted") {
    return false;
  }
  try {
    const notification = new Notification(title, {
      body,
      tag: `rivet-${Date.now()}`
    });
    // Auto-close to avoid stale notifications accumulating.
    window.setTimeout(() => notification.close(), 20_000);
    return true;
  } catch {
    return false;
  }
}

function notificationTaskTitle(task: TaskDto): string {
  const title = task.title.trim();
  if (title.length > 0) {
    return title;
  }
  const description = task.description.trim();
  if (description.length > 0) {
    return description;
  }
  return `Task ${task.uuid}`;
}

export function collectDueNotificationEvents(
  tasks: TaskDto[],
  timezone: string,
  config: DueNotificationConfig,
  sent: Set<string>,
  nowUtcMs: number
): DueNotificationEvent[] {
  if (!config.enabled) {
    return [];
  }

  const preMinutes = Math.max(1, Math.min(43_200, config.pre_notify_minutes));
  const events: DueNotificationEvent[] = [];

  for (const task of tasks) {
    if (!(task.status === "Pending" || task.status === "Waiting")) {
      continue;
    }
    const dueRaw = task.due?.trim();
    if (!dueRaw) {
      continue;
    }
    const dueUtcMs = parseTaskDueUtcMs(dueRaw);
    if (dueUtcMs === null) {
      continue;
    }

    const title = notificationTaskTitle(task);
    const dueLabel = formatDueDateTime(dueUtcMs, timezone);

    if (config.pre_notify_enabled && nowUtcMs < dueUtcMs) {
      const preDueMs = dueUtcMs - preMinutes * 60_000;
      if (nowUtcMs >= preDueMs) {
        const preKey = `${task.uuid}:${dueUtcMs}:pre:${preMinutes}`;
        if (!sent.has(preKey)) {
          events.push({
            key: preKey,
            title: `Task due soon (${preMinutes}m)`,
            body: `${title}\nDue ${dueLabel}`
          });
        }
      }
    }

    if (nowUtcMs >= dueUtcMs) {
      const dueKey = `${task.uuid}:${dueUtcMs}:due`;
      if (!sent.has(dueKey)) {
        events.push({
          key: dueKey,
          title: "Task due now",
          body: `${title}\nDue ${dueLabel}`
        });
      }
    }
  }

  return events;
}
