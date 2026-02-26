export type UiLogLevel = "debug" | "info" | "warn" | "error";

export interface LoggerBridge {
  (event: string, detail: string): Promise<void>;
}

let bridge: LoggerBridge | null = null;
let bridgeMinLevel: UiLogLevel = "warn";

const LEVEL_ORDER: Record<UiLogLevel, number> = {
  debug: 10,
  info: 20,
  warn: 30,
  error: 40
};

export function setLoggerBridge(next: LoggerBridge, minLevel: UiLogLevel = "warn"): void {
  bridge = next;
  bridgeMinLevel = minLevel;
}

function emit(level: UiLogLevel, event: string, detail: string): void {
  const timestamp = new Date().toISOString();
  const payload = `[${timestamp}] [${level}] ${event} :: ${detail}`;
  if (level === "error") {
    console.error(payload);
  } else if (level === "warn") {
    console.warn(payload);
  } else {
    console.log(payload);
  }

  if (bridge && LEVEL_ORDER[level] >= LEVEL_ORDER[bridgeMinLevel]) {
    void bridge(event, `${level}: ${detail}`);
  }
}

export const logger = {
  debug(event: string, detail: string): void {
    emit("debug", event, detail);
  },
  info(event: string, detail: string): void {
    emit("info", event, detail);
  },
  warn(event: string, detail: string): void {
    emit("warn", event, detail);
  },
  error(event: string, detail: string): void {
    emit("error", event, detail);
  }
};
