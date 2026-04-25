import { Profiler, memo, useCallback, useEffect, useState } from "react";
import type { ProfilerOnRenderCallback } from "react";

import AddIcon from "@mui/icons-material/Add";
import BugReportIcon from "@mui/icons-material/BugReport";
import CalendarMonthIcon from "@mui/icons-material/CalendarMonth";
import ChecklistIcon from "@mui/icons-material/Checklist";
import ContactsIcon from "@mui/icons-material/Contacts";
import DarkModeIcon from "@mui/icons-material/DarkMode";
import LightModeIcon from "@mui/icons-material/LightMode";
import MapIcon from "@mui/icons-material/Map";
import MenuBookIcon from "@mui/icons-material/MenuBook";
import MenuIcon from "@mui/icons-material/Menu";
import SettingsIcon from "@mui/icons-material/Settings";
import ViewKanbanIcon from "@mui/icons-material/ViewKanban";
import AppBar from "@mui/material/AppBar";
import Button from "@mui/material/Button";
import Divider from "@mui/material/Divider";
import Drawer from "@mui/material/Drawer";
import IconButton from "@mui/material/IconButton";
import List from "@mui/material/List";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Stack from "@mui/material/Stack";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";

import { AddTaskDialog } from "../components/AddTaskDialog";
import { DiagnosticsPanel } from "../components/DiagnosticsPanel";
import { SettingsDialog } from "../components/SettingsDialog";
import { CalendarWorkspace } from "../features/calendar/CalendarWorkspace";
import { ContactsWorkspace } from "../features/contacts/ContactsWorkspace";
import { DictionaryWorkspace } from "../features/dictionary/DictionaryWorkspace";
import { KanbanWorkspace } from "../features/kanban/KanbanWorkspace";
import { MapWorkspace } from "../features/map/MapWorkspace";
import { TasksWorkspace } from "../features/tasks/TasksWorkspace";
import { logger } from "../lib/logger";
import { useDiagnosticsSlice, useSettingsSlice, useShellSlice } from "../store/slices";

const TasksWorkspaceMemo = memo(TasksWorkspace);
const KanbanWorkspaceMemo = memo(KanbanWorkspace);
const CalendarWorkspaceMemo = memo(CalendarWorkspace);
const DictionaryWorkspaceMemo = memo(DictionaryWorkspace);
const MapWorkspaceMemo = memo(MapWorkspace);
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
    dictionaryLanguages,
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
  const [dictionaryTaskSplitOpen, setDictionaryTaskSplitOpen] = useState(false);
  const [tabDrawerOpen, setTabDrawerOpen] = useState(false);
  const [mountedTabs, setMountedTabs] = useState<{
    tasks: boolean;
    kanban: boolean;
    calendar: boolean;
    dictionary: boolean;
    map: boolean;
    contacts: boolean;
  }>({
    tasks: true,
    kanban: false,
    calendar: false,
    dictionary: false,
    map: false,
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
  const dictionaryHideWhenUnavailable = runtimeConfig?.dictionary?.hide_when_unavailable ?? false;
  const dictionaryAvailable = dictionaryLanguages.length > 0;
  const dictionaryFeatureEnabled = (runtimeConfig?.ui?.features?.dictionary ?? true)
    && (!dictionaryHideWhenUnavailable || dictionaryAvailable);
  const mapFeatureEnabled = (runtimeConfig?.ui?.features?.map ?? true)
    && (runtimeConfig?.map?.enabled ?? true);
  const mapBaseUrl = String(runtimeConfig?.map?.martin_base_url ?? "http://127.0.0.1:3002").trim();
  const mapHideWhenUnavailable = runtimeConfig?.map?.hide_when_unavailable ?? false;
  const themeIconMode = themeFollowSystem ? systemThemeMode : themeMode;
  const tabItems = [
    { value: "tasks", label: "Tasks", icon: <ChecklistIcon fontSize="small" />, enabled: true },
    { value: "kanban", label: "Kanban", icon: <ViewKanbanIcon fontSize="small" />, enabled: true },
    { value: "calendar", label: "Calendar", icon: <CalendarMonthIcon fontSize="small" />, enabled: true },
    { value: "dictionary", label: "Dictionary", icon: <MenuBookIcon fontSize="small" />, enabled: dictionaryFeatureEnabled },
    { value: "map", label: "Map", icon: <MapIcon fontSize="small" />, enabled: mapFeatureEnabled },
    { value: "contacts", label: "Contacts", icon: <ContactsIcon fontSize="small" />, enabled: contactsFeatureEnabled }
  ] as const;
  const visibleTabItems = tabItems.filter((item) => item.enabled);
  const activeTabLabel = visibleTabItems.find((item) => item.value === activeTab)?.label ?? activeTab;
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
    logger.info(
      "map.config.effective",
      `enabled=${mapFeatureEnabled} base_url=${mapBaseUrl} hide_when_unavailable=${mapHideWhenUnavailable} max_parallel_image_requests=${runtimeConfig?.map?.max_parallel_image_requests ?? "(default)"}`
    );
  }, [mapBaseUrl, mapFeatureEnabled, mapHideWhenUnavailable, runtimeConfig?.map?.max_parallel_image_requests]);

  useEffect(() => {
    const openSplit = () => {
      setMountedTabs((previous) => ({ ...previous, dictionary: true, tasks: true }));
      setActiveTab("dictionary");
      setDictionaryTaskSplitOpen(true);
      logger.info("dictionary.split.open", "dictionary/tasks split view enabled");
    };
    const closeSplit = () => {
      setDictionaryTaskSplitOpen(false);
      logger.info("dictionary.split.close", "dictionary/tasks split view disabled");
    };
    window.addEventListener("rivet:dictionary-open-tasks-split", openSplit);
    window.addEventListener("rivet:dictionary-close-tasks-split", closeSplit);
    return () => {
      window.removeEventListener("rivet:dictionary-open-tasks-split", openSplit);
      window.removeEventListener("rivet:dictionary-close-tasks-split", closeSplit);
    };
  }, [setActiveTab]);

  useEffect(() => {
    if (!contactsFeatureEnabled && activeTab === "contacts") {
      setActiveTab("tasks");
      return;
    }
    if (!dictionaryFeatureEnabled && activeTab === "dictionary") {
      setActiveTab("tasks");
      return;
    }
    if (!mapFeatureEnabled && activeTab === "map") {
      setActiveTab("tasks");
    }
  }, [activeTab, contactsFeatureEnabled, dictionaryFeatureEnabled, mapFeatureEnabled, setActiveTab]);

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
        if (!dictionaryFeatureEnabled) {
          return;
        }
        event.preventDefault();
        setActiveTab("dictionary");
        return;
      }
      if (isMeta && key === "5") {
        if (!mapFeatureEnabled) {
          return;
        }
        event.preventDefault();
        setActiveTab("map");
        return;
      }
      if (isMeta && key === "6") {
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
  }, [activeTab, closeSettings, contactsFeatureEnabled, dictionaryFeatureEnabled, mapFeatureEnabled, openAddTaskDialog, openSettings, setActiveTab, settingsOpen]);

  return (
    <div className="flex h-screen min-h-screen flex-col overflow-hidden">
      <AppBar position="static" elevation={0} color="transparent" className="!border-b !border-solid !border-current/10 !backdrop-blur">
        <Toolbar variant="dense" className="!min-h-[40px] !gap-3">
          <IconButton
            size="small"
            edge="start"
            aria-label="open feature drawer"
            onClick={() => setTabDrawerOpen(true)}
          >
            <MenuIcon fontSize="small" />
          </IconButton>
          <img src="/assets/icons/mascot-square.png" alt="Rivet mascot" className="h-5 w-5 rounded-sm border border-current/20" />
          <Typography variant="subtitle1" className="min-w-[72px]">
            Rivet
          </Typography>
          <Typography variant="caption" color="text.secondary">
            view: {activeTabLabel}
          </Typography>
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
        {dictionaryFeatureEnabled && mountedTabs.dictionary ? (
          <div className={activeTab === "dictionary" ? "h-full" : "hidden h-full"} aria-hidden={activeTab !== "dictionary"}>
            <Profiler id="dictionary.workspace" onRender={onProfilerRender}>
              {dictionaryTaskSplitOpen ? (
                <div className="grid h-full min-h-0 grid-cols-[minmax(0,1fr)_minmax(440px,48%)] gap-3 p-3">
                  <div className="min-h-0 overflow-hidden rounded-md border border-current/10">
                    <DictionaryWorkspaceMemo />
                  </div>
                  <div className="min-h-0 overflow-hidden rounded-md border border-current/10">
                    <TasksWorkspaceMemo />
                  </div>
                </div>
              ) : (
                <DictionaryWorkspaceMemo />
              )}
            </Profiler>
          </div>
        ) : null}
        {mapFeatureEnabled && mountedTabs.map ? (
          <div className={activeTab === "map" ? "h-full" : "hidden h-full"} aria-hidden={activeTab !== "map"}>
            <Profiler id="map.workspace" onRender={onProfilerRender}>
              <MapWorkspaceMemo />
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

      <Drawer anchor="left" open={tabDrawerOpen} onClose={() => setTabDrawerOpen(false)}>
        <div className="w-[280px] max-w-[80vw]">
          <div className="px-4 py-3">
            <Typography variant="subtitle1">Feature Tabs</Typography>
            <Typography variant="caption" color="text.secondary">Select a workspace</Typography>
          </div>
          <Divider />
          <List>
            {visibleTabItems.map((item) => (
              <ListItemButton
                key={item.value}
                selected={activeTab === item.value}
                onClick={() => {
                  setActiveTab(item.value);
                  setTabDrawerOpen(false);
                }}
              >
                <ListItemIcon>{item.icon}</ListItemIcon>
                <ListItemText primary={item.label} />
              </ListItemButton>
            ))}
          </List>
        </div>
      </Drawer>

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
