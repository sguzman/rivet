import { Profiler, memo, useCallback, useEffect, useState } from "react";
import type { ProfilerOnRenderCallback } from "react";

import AddIcon from "@mui/icons-material/Add";
import BugReportIcon from "@mui/icons-material/BugReport";
import DarkModeIcon from "@mui/icons-material/DarkMode";
import LightModeIcon from "@mui/icons-material/LightMode";
import SettingsIcon from "@mui/icons-material/Settings";
import AppBar from "@mui/material/AppBar";
import Button from "@mui/material/Button";
import Stack from "@mui/material/Stack";
import Tab from "@mui/material/Tab";
import Tabs from "@mui/material/Tabs";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";

import { AddTaskDialog } from "../components/AddTaskDialog";
import { DiagnosticsPanel } from "../components/DiagnosticsPanel";
import { SettingsDialog } from "../components/SettingsDialog";
import { CalendarWorkspace } from "../features/calendar/CalendarWorkspace";
import { ContactsWorkspace } from "../features/contacts/ContactsWorkspace";
import { KanbanWorkspace } from "../features/kanban/KanbanWorkspace";
import { TasksWorkspace } from "../features/tasks/TasksWorkspace";
import { logger } from "../lib/logger";
import { useDiagnosticsSlice, useSettingsSlice, useShellSlice } from "../store/slices";

const TasksWorkspaceMemo = memo(TasksWorkspace);
const KanbanWorkspaceMemo = memo(KanbanWorkspace);
const CalendarWorkspaceMemo = memo(CalendarWorkspace);
const ContactsWorkspaceMemo = memo(ContactsWorkspace);

export function AppShell() {
  const {
    bootstrap,
    activeTab,
    setActiveTab,
    themeMode,
    themeFollowSystem,
    systemThemeMode,
    toggleTheme,
    addTaskDialogOpen,
    addTaskDialogContext,
    openAddTaskDialog,
    closeAddTaskDialog,
    createTask,
    loading,
    runtimeConfig,
    tagSchema,
    tagColorMap,
    kanbanBoards
  } = useShellSlice();
  const {
    settingsOpen,
    openSettings,
    closeSettings,
    dueConfig,
    duePermission,
    setThemeFollowSystem,
    setDueNotificationsEnabled,
    setDuePreNotifyEnabled,
    setDuePreNotifyMinutes,
    requestDueNotificationPermission,
    scanDueNotifications
  } = useSettingsSlice();
  const { commandFailures, clearCommandFailures } = useDiagnosticsSlice();

  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);
  const [mountedTabs, setMountedTabs] = useState<{
    tasks: boolean;
    kanban: boolean;
    calendar: boolean;
    contacts: boolean;
  }>({
    tasks: true,
    kanban: false,
    calendar: false,
    contacts: false
  });

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  const runtimeMode = runtimeConfig?.app?.mode ?? runtimeConfig?.mode ?? "prod";
  const loggingDirectory = runtimeConfig?.logging?.directory ?? "logs";
  const isDevMode = runtimeMode === "dev";
  const verboseRenderProfiling = String(import.meta.env.VITE_RIVET_PROFILE_VERBOSE ?? "").trim() === "1";
  const contactsFeatureEnabled = runtimeConfig?.ui?.features?.contacts ?? true;
  const themeIconMode = themeFollowSystem ? systemThemeMode : themeMode;
  const onProfilerRender = useCallback<ProfilerOnRenderCallback>(
    (id, phase, actualDuration, baseDuration) => {
      if (!isDevMode) {
        return;
      }
      if (id.endsWith(".workspace") && actualDuration > 120) {
        logger.warn(
          "render.budget",
          `${id} phase=${phase} actual_ms=${actualDuration.toFixed(2)} budget_ms=120`
        );
      }
      if (verboseRenderProfiling) {
        logger.debug(
          "render.profile",
          `${id} phase=${phase} actual_ms=${actualDuration.toFixed(2)} base_ms=${baseDuration.toFixed(2)}`
        );
      }
    },
    [isDevMode, verboseRenderProfiling]
  );

  useEffect(() => {
    setMountedTabs((previous) => ({
      ...previous,
      [activeTab]: true
    }));
  }, [activeTab]);

  useEffect(() => {
    if (!contactsFeatureEnabled && activeTab === "contacts") {
      setActiveTab("tasks");
    }
  }, [activeTab, contactsFeatureEnabled, setActiveTab]);

  useEffect(() => {
    scanDueNotifications();
    const id = window.setInterval(() => {
      scanDueNotifications();
    }, 30_000);
    return () => window.clearInterval(id);
  }, [scanDueNotifications]);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const isEditable = Boolean(
        target
          && (target.tagName === "INPUT"
            || target.tagName === "TEXTAREA"
            || target.tagName === "SELECT"
            || target.isContentEditable)
      );
      const isMeta = event.metaKey || event.ctrlKey;
      const key = event.key.toLowerCase();

      if (isMeta && key === "n") {
        if (activeTab === "contacts" && contactsFeatureEnabled) {
          return;
        }
        event.preventDefault();
        openAddTaskDialog();
        return;
      }

      if (isMeta && key === ",") {
        event.preventDefault();
        openSettings();
        return;
      }

      if (isMeta && key === "1") {
        event.preventDefault();
        setActiveTab("tasks");
        return;
      }
      if (isMeta && key === "2") {
        event.preventDefault();
        setActiveTab("kanban");
        return;
      }
      if (isMeta && key === "3") {
        event.preventDefault();
        setActiveTab("calendar");
        return;
      }
      if (isMeta && key === "4") {
        if (!contactsFeatureEnabled) {
          return;
        }
        event.preventDefault();
        setActiveTab("contacts");
        return;
      }

      if (!isEditable && key === "escape") {
        if (settingsOpen) {
          closeSettings();
        }
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [activeTab, closeSettings, contactsFeatureEnabled, openAddTaskDialog, openSettings, setActiveTab, settingsOpen]);

  return (
    <div className="flex h-screen min-h-screen flex-col overflow-hidden">
      <AppBar position="static" elevation={0} color="transparent" className="!border-b !border-solid !border-current/10 !backdrop-blur">
        <Toolbar variant="dense" className="!min-h-[40px] !gap-3">
          <img src="/assets/icons/mascot-square.png" alt="Rivet mascot" className="h-5 w-5 rounded-sm border border-current/20" />
          <Typography variant="subtitle1" className="min-w-[72px]">
            Rivet
          </Typography>
          <Tabs
            value={activeTab}
            onChange={(_, value: string) => setActiveTab(value as "tasks" | "kanban" | "calendar" | "contacts")}
            textColor="primary"
            indicatorColor="primary"
          >
            <Tab value="tasks" label="Tasks" />
            <Tab value="kanban" label="Kanban" />
            <Tab value="calendar" label="Calendar" />
            {contactsFeatureEnabled ? <Tab value="contacts" label="Contacts" /> : null}
          </Tabs>
          <div className="ml-auto" />
          <Stack direction="row" spacing={1} alignItems="center">
            <Typography variant="caption" color="text.secondary">
              mode: {runtimeMode}
            </Typography>
            <Typography variant="caption" color="text.secondary">
              logs: {loggingDirectory}
            </Typography>
            <Button
              variant="outlined"
              size="small"
              startIcon={<AddIcon fontSize="small" />}
              onClick={() => openAddTaskDialog()}
            >
              Add Task
            </Button>
            <Button
              variant="outlined"
              size="small"
              onClick={toggleTheme}
              disabled={themeFollowSystem}
              startIcon={themeIconMode === "day" ? <DarkModeIcon fontSize="small" /> : <LightModeIcon fontSize="small" />}
            >
              {themeFollowSystem ? `System (${systemThemeMode})` : (themeMode === "day" ? "Night" : "Day")}
            </Button>
            <Button
              variant="outlined"
              size="small"
              onClick={openSettings}
              startIcon={<SettingsIcon fontSize="small" />}
            >
              Settings
            </Button>
            {isDevMode ? (
              <Button
                variant="outlined"
                size="small"
                onClick={() => setDiagnosticsOpen((open) => !open)}
                startIcon={<BugReportIcon fontSize="small" />}
              >
                Diagnostics
              </Button>
            ) : null}
          </Stack>
        </Toolbar>
      </AppBar>

      <main className="min-h-0 flex-1 overflow-hidden">
        {mountedTabs.tasks ? (
          <div className={activeTab === "tasks" ? "h-full" : "hidden h-full"} aria-hidden={activeTab !== "tasks"}>
            <Profiler id="tasks.workspace" onRender={onProfilerRender}>
              <TasksWorkspaceMemo />
            </Profiler>
          </div>
        ) : null}
        {mountedTabs.kanban ? (
          <div className={activeTab === "kanban" ? "h-full" : "hidden h-full"} aria-hidden={activeTab !== "kanban"}>
            <Profiler id="kanban.workspace" onRender={onProfilerRender}>
              <KanbanWorkspaceMemo />
            </Profiler>
          </div>
        ) : null}
        {mountedTabs.calendar ? (
          <div className={activeTab === "calendar" ? "h-full" : "hidden h-full"} aria-hidden={activeTab !== "calendar"}>
            <Profiler id="calendar.workspace" onRender={onProfilerRender}>
              <CalendarWorkspaceMemo />
            </Profiler>
          </div>
        ) : null}
        {contactsFeatureEnabled && mountedTabs.contacts ? (
          <div className={activeTab === "contacts" ? "h-full" : "hidden h-full"} aria-hidden={activeTab !== "contacts"}>
            <Profiler id="contacts.workspace" onRender={onProfilerRender}>
              <ContactsWorkspaceMemo />
            </Profiler>
          </div>
        ) : null}
      </main>

      <AddTaskDialog
        open={addTaskDialogOpen}
        busy={loading}
        context={addTaskDialogContext}
        tagSchema={tagSchema}
        tagColorMap={tagColorMap}
        kanbanBoards={kanbanBoards}
        onClose={closeAddTaskDialog}
        onSubmit={createTask}
      />

      <SettingsDialog
        open={settingsOpen}
        runtimeMode={runtimeMode}
        loggingDirectory={loggingDirectory}
        dueConfig={dueConfig}
        duePermission={duePermission}
        themeFollowSystem={themeFollowSystem}
        onClose={closeSettings}
        onToggleThemeFollowSystem={setThemeFollowSystem}
        onToggleEnabled={setDueNotificationsEnabled}
        onTogglePreEnabled={setDuePreNotifyEnabled}
        onPreMinutesChange={setDuePreNotifyMinutes}
        onRequestPermission={() => {
          void requestDueNotificationPermission();
        }}
      />

      <DiagnosticsPanel
        open={isDevMode && diagnosticsOpen}
        failures={commandFailures}
        onClear={clearCommandFailures}
        onClose={() => setDiagnosticsOpen(false)}
      />
    </div>
  );
}
