import { useEffect } from "react";

import AddIcon from "@mui/icons-material/Add";
import DarkModeIcon from "@mui/icons-material/DarkMode";
import LightModeIcon from "@mui/icons-material/LightMode";
import AppBar from "@mui/material/AppBar";
import Button from "@mui/material/Button";
import Stack from "@mui/material/Stack";
import Tab from "@mui/material/Tab";
import Tabs from "@mui/material/Tabs";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";

import { AddTaskDialog } from "../components/AddTaskDialog";
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
  const openAddTaskDialog = useAppStore((state) => state.openAddTaskDialog);
  const closeAddTaskDialog = useAppStore((state) => state.closeAddTaskDialog);
  const createTask = useAppStore((state) => state.createTask);
  const loading = useAppStore((state) => state.loading);
  const runtimeConfig = useAppStore((state) => state.runtimeConfig);

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  const runtimeMode = runtimeConfig?.app?.mode ?? runtimeConfig?.mode ?? "prod";

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
            <Button
              variant="outlined"
              size="small"
              startIcon={<AddIcon fontSize="small" />}
              onClick={openAddTaskDialog}
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
        onClose={closeAddTaskDialog}
        onSubmit={createTask}
      />
    </div>
  );
}
