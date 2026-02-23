import { useEffect, useState } from "react";

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
import { KanbanWorkspace } from "../features/kanban/KanbanWorkspace";
import { TasksWorkspace } from "../features/tasks/TasksWorkspace";
import { useAppStore } from "../store/useAppStore";

export function AppShell() {
  const bootstrap = useAppStore((state) => state.bootstrap);
  const activeTab = useAppStore((state) => state.activeTab);
  const setActiveTab = useAppStore((state) => state.setActiveTab);
  const themeMode = useAppStore((state) => state.themeMode);
  const toggleTheme = useAppStore((state) => state.toggleTheme);
  const addTaskDialogOpen = useAppStore((state) => state.addTaskDialogOpen);
  const addTaskDialogContext = useAppStore((state) => state.addTaskDialogContext);
  const openAddTaskDialog = useAppStore((state) => state.openAddTaskDialog);
  const closeAddTaskDialog = useAppStore((state) => state.closeAddTaskDialog);
  const createTask = useAppStore((state) => state.createTask);
  const loading = useAppStore((state) => state.loading);
  const runtimeConfig = useAppStore((state) => state.runtimeConfig);
  const tagSchema = useAppStore((state) => state.tagSchema);
  const tagColorMap = useAppStore((state) => state.tagColorMap);
  const kanbanBoards = useAppStore((state) => state.kanbanBoards);
  const settingsOpen = useAppStore((state) => state.settingsOpen);
  const openSettings = useAppStore((state) => state.openSettings);
  const closeSettings = useAppStore((state) => state.closeSettings);
  const dueConfig = useAppStore((state) => state.dueNotificationConfig);
  const duePermission = useAppStore((state) => state.dueNotificationPermission);
  const setDueNotificationsEnabled = useAppStore((state) => state.setDueNotificationsEnabled);
  const setDuePreNotifyEnabled = useAppStore((state) => state.setDuePreNotifyEnabled);
  const setDuePreNotifyMinutes = useAppStore((state) => state.setDuePreNotifyMinutes);
  const requestDueNotificationPermission = useAppStore((state) => state.requestDueNotificationPermission);
  const scanDueNotifications = useAppStore((state) => state.scanDueNotifications);
  const commandFailures = useAppStore((state) => state.commandFailures);
  const clearCommandFailures = useAppStore((state) => state.clearCommandFailures);

  const [diagnosticsOpen, setDiagnosticsOpen] = useState(false);

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  const runtimeMode = runtimeConfig?.app?.mode ?? runtimeConfig?.mode ?? "prod";
  const loggingDirectory = runtimeConfig?.logging?.directory ?? "logs";
  const isDevMode = runtimeMode === "dev";

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

      if (!isEditable && key === "escape") {
        if (settingsOpen) {
          closeSettings();
        }
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [closeSettings, openAddTaskDialog, openSettings, setActiveTab, settingsOpen]);

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
            onChange={(_, value: string) => setActiveTab(value as "tasks" | "kanban" | "calendar")}
            textColor="primary"
            indicatorColor="primary"
          >
            <Tab value="tasks" label="Tasks" />
            <Tab value="kanban" label="Kanban" />
            <Tab value="calendar" label="Calendar" />
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
              startIcon={themeMode === "day" ? <DarkModeIcon fontSize="small" /> : <LightModeIcon fontSize="small" />}
            >
              {themeMode === "day" ? "Night" : "Day"}
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
        {activeTab === "tasks" ? <TasksWorkspace /> : null}
        {activeTab === "kanban" ? <KanbanWorkspace /> : null}
        {activeTab === "calendar" ? <CalendarWorkspace /> : null}
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
        onClose={closeSettings}
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
