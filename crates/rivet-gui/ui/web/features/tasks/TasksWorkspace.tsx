import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

import { TaskDetailsPanel } from "../../components/TaskDetailsPanel";
import { TaskListPanel } from "../../components/TaskListPanel";
import { useAppStore, useFilteredTasks, useSelectedTask } from "../../store/useAppStore";

export function TasksWorkspace() {
  const loading = useAppStore((state) => state.loading);
  const error = useAppStore((state) => state.error);
  const selectedTaskId = useAppStore((state) => state.selectedTaskId);
  const filters = useAppStore((state) => state.filters);

  const setSearchFilter = useAppStore((state) => state.setSearchFilter);
  const setStatusFilter = useAppStore((state) => state.setStatusFilter);
  const setProjectFilter = useAppStore((state) => state.setProjectFilter);
  const setTagFilter = useAppStore((state) => state.setTagFilter);
  const clearFilters = useAppStore((state) => state.clearFilters);
  const selectTask = useAppStore((state) => state.selectTask);
  const markTaskDone = useAppStore((state) => state.markTaskDone);
  const removeTask = useAppStore((state) => state.removeTask);

  const visibleTasks = useFilteredTasks();
  const selectedTask = useSelectedTask();

  return (
    <div className="grid h-full min-h-0 grid-cols-[260px_minmax(0,1fr)_360px] gap-3 p-3">
      <Paper className="p-4">
        <Stack spacing={2}>
          <Typography variant="h6">Task Filters</Typography>
          <TextField
            label="Search"
            value={filters.search}
            onChange={(event) => setSearchFilter(event.target.value)}
            size="small"
          />
          <TextField
            select
            label="Status"
            value={filters.status}
            onChange={(event) => setStatusFilter(event.target.value as typeof filters.status)}
            size="small"
          >
            <MenuItem value="all">All</MenuItem>
            <MenuItem value="Pending">Pending</MenuItem>
            <MenuItem value="Waiting">Waiting</MenuItem>
            <MenuItem value="Completed">Completed</MenuItem>
            <MenuItem value="Deleted">Deleted</MenuItem>
          </TextField>
          <TextField
            label="Project"
            value={filters.project}
            onChange={(event) => setProjectFilter(event.target.value)}
            size="small"
          />
          <TextField
            label="Tag"
            value={filters.tag}
            onChange={(event) => setTagFilter(event.target.value)}
            size="small"
          />
          <Button variant="outlined" onClick={clearFilters}>
            Clear Filters
          </Button>
          {loading ? <Alert severity="info">Loading...</Alert> : null}
          {error ? <Alert severity="error">{error}</Alert> : null}
        </Stack>
      </Paper>

      <TaskListPanel tasks={visibleTasks} selectedTaskId={selectedTaskId} onSelectTask={selectTask} />

      <TaskDetailsPanel task={selectedTask} busy={loading} onDone={markTaskDone} onDelete={removeTask} />
    </div>
  );
}
