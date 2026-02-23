import { useEffect, useState } from "react";

import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

import { TaskDetailsPanel } from "../../components/TaskDetailsPanel";
import { TaskListPanel } from "../../components/TaskListPanel";
import {
  useAppStore,
  useSelectedTask,
  useTaskViewData
} from "../../store/useAppStore";

export function TasksWorkspace() {
  const loading = useAppStore((state) => state.loading);
  const error = useAppStore((state) => state.error);
  const selectedTaskId = useAppStore((state) => state.selectedTaskId);
  const filters = useAppStore((state) => state.taskFilters);

  const setSearchFilter = useAppStore((state) => state.setTaskSearchFilter);
  const setStatusFilter = useAppStore((state) => state.setTaskStatusFilter);
  const setProjectFilter = useAppStore((state) => state.setTaskProjectFilter);
  const setTagFilter = useAppStore((state) => state.setTaskTagFilter);
  const setPriorityFilter = useAppStore((state) => state.setTaskPriorityFilter);
  const setDueFilter = useAppStore((state) => state.setTaskDueFilter);
  const clearFilters = useAppStore((state) => state.clearTaskFilters);
  const selectTask = useAppStore((state) => state.selectTask);
  const markTaskDone = useAppStore((state) => state.markTaskDone);
  const removeTask = useAppStore((state) => state.removeTask);

  const { visibleTasks, projectFacets, tagFacets } = useTaskViewData();
  const selectedTask = useSelectedTask();
  const [searchInput, setSearchInput] = useState(filters.search);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setSearchFilter(searchInput);
    }, 180);
    return () => window.clearTimeout(timeout);
  }, [searchInput, setSearchFilter]);

  return (
    <div className="grid h-full min-h-0 grid-cols-[minmax(0,1fr)_360px] gap-3 p-3">
      <TaskListPanel tasks={visibleTasks} selectedTaskId={selectedTaskId} onSelectTask={selectTask} />

      <Stack spacing={2} className="min-h-0">
        <Paper className="p-4">
          <Stack spacing={2}>
            <Typography variant="h6">Task Filters</Typography>
            <TextField
              label="Search"
              value={searchInput}
              onChange={(event) => setSearchInput(event.target.value)}
              size="small"
            />
            <TextField
              select
              label="Completion"
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
              select
              label="Project"
              value={filters.project}
              onChange={(event) => setProjectFilter(event.target.value)}
              size="small"
            >
              <MenuItem value="">All projects</MenuItem>
              {projectFacets.map((entry) => (
                <MenuItem key={entry.value} value={entry.value}>
                  {entry.value} ({entry.count})
                </MenuItem>
              ))}
            </TextField>
            <TextField
              select
              label="Tag"
              value={filters.tag}
              onChange={(event) => setTagFilter(event.target.value)}
              size="small"
            >
              <MenuItem value="">All tags</MenuItem>
              {tagFacets.map((entry) => (
                <MenuItem key={entry.value} value={entry.value}>
                  {entry.value} ({entry.count})
                </MenuItem>
              ))}
            </TextField>
            <TextField
              select
              label="Priority"
              value={filters.priority}
              onChange={(event) => setPriorityFilter(event.target.value as typeof filters.priority)}
              size="small"
            >
              <MenuItem value="all">All priorities</MenuItem>
              <MenuItem value="low">Low</MenuItem>
              <MenuItem value="medium">Medium</MenuItem>
              <MenuItem value="high">High</MenuItem>
              <MenuItem value="none">None</MenuItem>
            </TextField>
            <TextField
              select
              label="Due"
              value={filters.due}
              onChange={(event) => setDueFilter(event.target.value as typeof filters.due)}
              size="small"
            >
              <MenuItem value="all">All</MenuItem>
              <MenuItem value="has_due">Has due</MenuItem>
              <MenuItem value="no_due">No due</MenuItem>
            </TextField>
            <Button
              variant="outlined"
              onClick={() => {
                clearFilters();
                setSearchInput("");
              }}
            >
              Clear Filters
            </Button>
            {loading ? <Alert severity="info">Loading...</Alert> : null}
            {error ? <Alert severity="error">{error}</Alert> : null}
          </Stack>
        </Paper>

        <TaskDetailsPanel task={selectedTask} busy={loading} onDone={markTaskDone} onDelete={removeTask} />
      </Stack>
    </div>
  );
}
