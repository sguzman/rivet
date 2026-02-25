export type UiLogLevel = "debug" | "info" | "warn" | "error";

export interface LoggerBridge {
  (event: string, detail: string): Promise<void>;
}

let bridge: LoggerBridge | null = null;

export function setLoggerBridge(next: LoggerBridge): void {
  bridge = next;
}

function shouldBridge(level: UiLogLevel): boolean {
  // Keep backend command-bridge logging light.
  // Debug/info logs are high-volume and can
  // create UI latency when forwarded via
  // Tauri invoke on every interaction.
  return level === "warn" || level === "error";
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

  if (bridge && shouldBridge(level)) {
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
