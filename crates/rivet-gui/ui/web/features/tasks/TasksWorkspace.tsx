import { useEffect, useMemo, useState } from "react";

import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

import { TaskEditDialog } from "../../components/TaskEditDialog";
import { TaskDetailsPanel } from "../../components/TaskDetailsPanel";
import { TaskListPanel } from "../../components/TaskListPanel";
import { canManuallyCompleteTask, isCalendarEventTask } from "../../lib/calendar";
import { pushTagUnique, splitTags } from "../../lib/tags";
import {
  useSelectedTask,
  useTaskViewData
} from "../../store/useAppStore";
import { useTaskWorkspaceSlice } from "../../store/slices";

export function TasksWorkspace() {
  const {
    loading,
    error,
    selectedTaskId,
    tagSchema,
    tagColorMap,
    kanbanBoards,
    filters,
    setSearchFilter,
    setStatusFilter,
    setProjectFilter,
    setTagFilter,
    setPriorityFilter,
    setDueFilter,
    clearFilters,
    selectTask,
    updateTask,
    markTaskDone,
    markTaskUndone,
    removeTask,
    markTasksDoneBulk,
    markTasksUndoneBulk,
    removeTasksBulk
  } = useTaskWorkspaceSlice();

  const { visibleTasks, projectFacets, tagFacets } = useTaskViewData();
  const selectedTask = useSelectedTask();
  const [searchInput, setSearchInput] = useState(filters.search);
  const [editOpen, setEditOpen] = useState(false);
  const [selectMode, setSelectMode] = useState(false);
  const [selectedTaskIds, setSelectedTaskIds] = useState<string[]>([]);
  const [lastSelectedIndex, setLastSelectedIndex] = useState<number | null>(null);
  const [bulkProjectInput, setBulkProjectInput] = useState("");
  const [bulkTagInput, setBulkTagInput] = useState("");
  const [nowUtcMs, setNowUtcMs] = useState(() => Date.now());

  useEffect(() => {
    if (!selectedTask && editOpen) {
      setEditOpen(false);
    }
  }, [selectedTask, editOpen]);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setSearchFilter(searchInput);
    }, 180);
    return () => window.clearTimeout(timeout);
  }, [searchInput, setSearchFilter]);

  useEffect(() => {
    const intervalId = window.setInterval(() => {
      setNowUtcMs(Date.now());
    }, 30_000);
    return () => window.clearInterval(intervalId);
  }, []);

  useEffect(() => {
    const visibleIdSet = new Set(visibleTasks.map((task) => task.uuid));
    setSelectedTaskIds((previous) => {
      const next = previous.filter((uuid) => visibleIdSet.has(uuid));
      if (next.length === previous.length && next.every((uuid, index) => uuid === previous[index])) {
        return previous;
      }
      return next;
    });
    if (lastSelectedIndex !== null && lastSelectedIndex >= visibleTasks.length) {
      setLastSelectedIndex(visibleTasks.length > 0 ? visibleTasks.length - 1 : null);
    }
  }, [lastSelectedIndex, visibleTasks]);

  const visibleTaskIds = useMemo(() => visibleTasks.map((task) => task.uuid), [visibleTasks]);
  const selectedTaskSet = useMemo(() => new Set(selectedTaskIds), [selectedTaskIds]);
  const selectedTasks = useMemo(
    () => visibleTasks.filter((task) => selectedTaskSet.has(task.uuid)),
    [selectedTaskSet, visibleTasks]
  );

  const doneCandidates = visibleTasks.filter(
    (task) => (task.status === "Pending" || task.status === "Waiting") && canManuallyCompleteTask(task, nowUtcMs)
  );
  const blockedDoneCandidates = visibleTasks.filter(
    (task) => (task.status === "Pending" || task.status === "Waiting") && !canManuallyCompleteTask(task, nowUtcMs)
  );
  const undoneCandidates = visibleTasks.filter((task) => task.status === "Completed");
  const doneCandidateIds = doneCandidates.map((task) => task.uuid);
  const undoneCandidateIds = undoneCandidates.map((task) => task.uuid);
  const deleteCandidateIds = visibleTasks.map((task) => task.uuid);
  const selectedDoneIds = selectedTasks
    .filter((task) => (task.status === "Pending" || task.status === "Waiting") && canManuallyCompleteTask(task, nowUtcMs))
    .map((task) => task.uuid);
  const selectedUndoneIds = selectedTasks
    .filter((task) => task.status === "Completed")
    .map((task) => task.uuid);
  const selectedDeleteIds = selectedTasks.map((task) => task.uuid);

  const doneBlockedMessage = selectedTask
    && (selectedTask.status === "Pending" || selectedTask.status === "Waiting")
    && isCalendarEventTask(selectedTask)
    && !canManuallyCompleteTask(selectedTask, nowUtcMs)
    ? "Calendar events auto-complete once their due time passes."
    : null;
  const canSelectedTaskBeDone = Boolean(
    selectedTask
    && (selectedTask.status === "Pending" || selectedTask.status === "Waiting")
    && canManuallyCompleteTask(selectedTask, nowUtcMs)
  );

  const handleTaskClick = (taskId: string, index: number, modifiers: { ctrlOrMeta: boolean; shift: boolean }) => {
    selectTask(taskId);
    if (!selectMode) {
      return;
    }

    setSelectedTaskIds((previous) => {
      if (modifiers.shift && lastSelectedIndex !== null) {
        const start = Math.min(lastSelectedIndex, index);
        const end = Math.max(lastSelectedIndex, index);
        const range = visibleTaskIds.slice(start, end + 1);
        if (modifiers.ctrlOrMeta) {
          const merged = new Set(previous);
          for (const uuid of range) {
            merged.add(uuid);
          }
          return Array.from(merged);
        }
        return range;
      }

      if (modifiers.ctrlOrMeta) {
        if (previous.includes(taskId)) {
          return previous.filter((uuid) => uuid !== taskId);
        }
        return [...previous, taskId];
      }

      return [taskId];
    });
    setLastSelectedIndex(index);
  };

  const toggleSelectMode = () => {
    if (selectMode) {
      setSelectMode(false);
      setSelectedTaskIds([]);
      setLastSelectedIndex(null);
      return;
    }
    setSelectMode(true);
    if (selectedTaskId) {
      const selectedIndex = visibleTaskIds.findIndex((uuid) => uuid === selectedTaskId);
      setSelectedTaskIds([selectedTaskId]);
      setLastSelectedIndex(selectedIndex >= 0 ? selectedIndex : null);
    }
  };

  const applyProjectToSelected = async () => {
    if (selectedTasks.length === 0) {
      return;
    }
    const nextProject = bulkProjectInput.trim() ? bulkProjectInput.trim() : null;
    for (const task of selectedTasks) {
      await updateTask(task.uuid, { project: nextProject });
    }
  };

  const applyTagToSelected = async () => {
    const tokens = splitTags(bulkTagInput);
    if (selectedTasks.length === 0 || tokens.length === 0) {
      return;
    }
    for (const task of selectedTasks) {
      const nextTags = [...task.tags];
      for (const tag of tokens) {
        pushTagUnique(nextTags, tag);
      }
      await updateTask(task.uuid, { tags: nextTags });
    }
    setBulkTagInput("");
  };

  return (
    <div className="grid h-full min-h-0 grid-cols-[minmax(0,1fr)_360px] gap-3 p-3">
      <TaskListPanel
        tasks={visibleTasks}
        selectedTaskId={selectedTaskId}
        selectMode={selectMode}
        selectedTaskIds={selectedTaskIds}
        onTaskClick={handleTaskClick}
      />

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
            <Stack direction={{ xs: "column", sm: "row" }} spacing={1}>
              <Button variant={selectMode ? "contained" : "outlined"} onClick={toggleSelectMode}>
                {selectMode ? "Exit Select Mode" : "Select Mode"}
              </Button>
              <Button
                variant="outlined"
                disabled={!selectMode || visibleTaskIds.length === 0}
                onClick={() => {
                  setSelectedTaskIds(visibleTaskIds);
                  setLastSelectedIndex(visibleTaskIds.length > 0 ? visibleTaskIds.length - 1 : null);
                }}
              >
                Select All
              </Button>
              <Button
                variant="outlined"
                disabled={!selectMode || selectedTaskIds.length === 0}
                onClick={() => {
                  setSelectedTaskIds([]);
                  setLastSelectedIndex(null);
                }}
              >
                Deselect All
              </Button>
            </Stack>
            <Stack direction={{ xs: "column", sm: "row" }} spacing={1}>
              <Button
                variant="contained"
                color="success"
                disabled={loading || doneCandidateIds.length === 0}
                onClick={() => {
                  void markTasksDoneBulk(doneCandidateIds);
                }}
              >
                Complete Filtered ({doneCandidateIds.length})
              </Button>
              <Button
                variant="outlined"
                color="warning"
                disabled={loading || undoneCandidateIds.length === 0}
                onClick={() => {
                  void markTasksUndoneBulk(undoneCandidateIds);
                }}
              >
                Uncomplete Filtered ({undoneCandidateIds.length})
              </Button>
              <Button
                variant="outlined"
                color="error"
                disabled={loading || deleteCandidateIds.length === 0}
                onClick={() => {
                  if (!window.confirm(`Delete ${deleteCandidateIds.length} filtered task(s)? This cannot be undone.`)) {
                    return;
                  }
                  void removeTasksBulk(deleteCandidateIds);
                }}
              >
                Delete Filtered ({deleteCandidateIds.length})
              </Button>
            </Stack>
            {blockedDoneCandidates.length > 0 ? (
              <Alert severity="info">
                {blockedDoneCandidates.length} calendar task(s) are excluded from manual completion until due time.
              </Alert>
            ) : null}
            {selectMode ? (
              <Paper variant="outlined" className="p-3">
                <Stack spacing={1.2}>
                  <Typography variant="subtitle2">Selected Tasks: {selectedTaskIds.length}</Typography>
                  <Stack direction={{ xs: "column", sm: "row" }} spacing={1}>
                    <Button
                      variant="contained"
                      color="success"
                      disabled={loading || selectedDoneIds.length === 0}
                      onClick={() => {
                        void markTasksDoneBulk(selectedDoneIds);
                      }}
                    >
                      Complete Selected ({selectedDoneIds.length})
                    </Button>
                    <Button
                      variant="outlined"
                      color="warning"
                      disabled={loading || selectedUndoneIds.length === 0}
                      onClick={() => {
                        void markTasksUndoneBulk(selectedUndoneIds);
                      }}
                    >
                      Uncomplete Selected ({selectedUndoneIds.length})
                    </Button>
                    <Button
                      variant="outlined"
                      color="error"
                      disabled={loading || selectedDeleteIds.length === 0}
                      onClick={() => {
                        if (!window.confirm(`Delete ${selectedDeleteIds.length} selected task(s)? This cannot be undone.`)) {
                          return;
                        }
                        void removeTasksBulk(selectedDeleteIds);
                      }}
                    >
                      Delete Selected ({selectedDeleteIds.length})
                    </Button>
                  </Stack>
                  <TextField
                    size="small"
                    label="Set Project (selected)"
                    value={bulkProjectInput}
                    onChange={(event) => setBulkProjectInput(event.target.value)}
                    helperText="Leave empty and apply to clear project."
                  />
                  <Button
                    variant="outlined"
                    disabled={loading || selectedDeleteIds.length === 0}
                    onClick={() => {
                      void applyProjectToSelected();
                    }}
                  >
                    Apply Project To Selected
                  </Button>
                  <TextField
                    size="small"
                    label="Add Tags (selected)"
                    value={bulkTagInput}
                    onChange={(event) => setBulkTagInput(event.target.value)}
                    helperText="Space-separated tags."
                  />
                  <Button
                    variant="outlined"
                    disabled={loading || selectedDeleteIds.length === 0 || splitTags(bulkTagInput).length === 0}
                    onClick={() => {
                      void applyTagToSelected();
                    }}
                  >
                    Add Tags To Selected
                  </Button>
                </Stack>
              </Paper>
            ) : null}
            {loading ? <Alert severity="info">Loading...</Alert> : null}
            {error ? <Alert severity="error">{error}</Alert> : null}
          </Stack>
        </Paper>

        <TaskDetailsPanel
          task={selectedTask}
          busy={loading}
          onEdit={() => setEditOpen(true)}
          onDone={markTaskDone}
          onUndone={markTaskUndone}
          onDelete={removeTask}
          canMarkDone={canSelectedTaskBeDone}
          doneBlockedMessage={doneBlockedMessage}
        />
      </Stack>

      <TaskEditDialog
        open={editOpen}
        task={selectedTask}
        busy={loading}
        tagSchema={tagSchema}
        tagColorMap={tagColorMap}
        kanbanBoards={kanbanBoards}
        onClose={() => setEditOpen(false)}
        onSubmit={async (uuid, patch) => {
          const updated = await updateTask(uuid, patch);
          return updated !== null;
        }}
      />
    </div>
  );
}
