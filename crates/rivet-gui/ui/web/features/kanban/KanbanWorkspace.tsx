import { useMemo, useState } from "react";
import type { DragEvent } from "react";

import AddIcon from "@mui/icons-material/Add";
import CompressIcon from "@mui/icons-material/Compress";
import DeleteIcon from "@mui/icons-material/Delete";
import DriveFileMoveIcon from "@mui/icons-material/DriveFileMove";
import EditIcon from "@mui/icons-material/Edit";
import ExpandIcon from "@mui/icons-material/Expand";
import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

import { TagChip } from "../../components/TagChip";
import { humanizeLane, kanbanLaneFromTask } from "../../lib/tags";
import {
  useAppStore,
  useKanbanColumns,
  useKanbanViewData
} from "../../store/useAppStore";
import type { TaskDto } from "../../types/core";

function KanbanCard(props: {
  task: TaskDto;
  compact: boolean;
  columns: string[];
  onMove: (lane: string) => void;
  onDone: () => void;
  onDelete: () => void;
  onDragStart: (event: DragEvent<HTMLDivElement>) => void;
  onDragEnd: () => void;
}) {
  const lane = kanbanLaneFromTask(props.task.tags, props.columns, props.columns[0] ?? "todo");
  const laneIndex = props.columns.findIndex((entry) => entry === lane);
  const nextLane = props.columns[(laneIndex + 1) % props.columns.length] ?? lane;
  const showActionMove = props.columns.length > 1 && nextLane !== lane;

  return (
    <Paper
      draggable
      onDragStart={props.onDragStart}
      onDragEnd={props.onDragEnd}
      className="cursor-grab active:cursor-grabbing"
      sx={{
        p: 1.25
      }}
    >
      <Stack spacing={1.1}>
        <Typography variant="subtitle2">{props.task.title || "Untitled Task"}</Typography>
        {props.task.description.trim() ? (
          <Typography variant="caption" color="text.secondary">
            {props.task.description}
          </Typography>
        ) : null}
        {!props.compact ? (
          <>
            <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap>
              <Typography variant="caption" className="rounded-md border border-current/15 px-1.5 py-0.5">
                project:{props.task.project ?? "—"}
              </Typography>
              {props.task.due ? (
                <Typography variant="caption" className="rounded-md border border-current/15 px-1.5 py-0.5">
                  due:{props.task.due}
                </Typography>
              ) : null}
            </Stack>
            <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap>
              {props.task.tags.slice(0, 4).map((tag) => (
                <TagChip key={tag} tag={tag} size="small" />
              ))}
            </Stack>
          </>
        ) : null}
        <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
          {showActionMove ? (
            <Button size="small" variant="outlined" startIcon={<DriveFileMoveIcon fontSize="small" />} onClick={() => props.onMove(nextLane)}>
              {humanizeLane(nextLane)}
            </Button>
          ) : null}
          {(props.task.status === "Pending" || props.task.status === "Waiting") ? (
            <Button size="small" variant="contained" color="success" onClick={props.onDone}>
              Done
            </Button>
          ) : null}
          <Button size="small" variant="outlined" color="error" onClick={props.onDelete}>
            Delete
          </Button>
        </Stack>
      </Stack>
    </Paper>
  );
}

export function KanbanWorkspace() {
  const error = useAppStore((state) => state.error);
  const loading = useAppStore((state) => state.loading);
  const openAddTaskDialog = useAppStore((state) => state.openAddTaskDialog);

  const boards = useAppStore((state) => state.kanbanBoards);
  const activeBoardId = useAppStore((state) => state.activeKanbanBoardId);
  const compactCards = useAppStore((state) => state.kanbanCompactCards);
  const draggingTaskId = useAppStore((state) => state.draggingKanbanTaskId);
  const dragOverLane = useAppStore((state) => state.dragOverKanbanLane);
  const filters = useAppStore((state) => state.kanbanFilters);

  const setActiveBoard = useAppStore((state) => state.setActiveKanbanBoard);
  const createBoard = useAppStore((state) => state.createKanbanBoard);
  const renameBoard = useAppStore((state) => state.renameActiveKanbanBoard);
  const deleteBoard = useAppStore((state) => state.deleteActiveKanbanBoard);
  const toggleCompact = useAppStore((state) => state.toggleKanbanCompactCards);
  const setDragging = useAppStore((state) => state.setDraggingKanbanTask);
  const setDragOver = useAppStore((state) => state.setDragOverKanbanLane);
  const moveTask = useAppStore((state) => state.moveKanbanTask);
  const markTaskDone = useAppStore((state) => state.markTaskDone);
  const removeTask = useAppStore((state) => state.removeTask);

  const setStatusFilter = useAppStore((state) => state.setKanbanStatusFilter);
  const setProjectFilter = useAppStore((state) => state.setKanbanProjectFilter);
  const setTagFilter = useAppStore((state) => state.setKanbanTagFilter);
  const setPriorityFilter = useAppStore((state) => state.setKanbanPriorityFilter);
  const setDueFilter = useAppStore((state) => state.setKanbanDueFilter);
  const clearFilters = useAppStore((state) => state.clearKanbanFilters);

  const columns = useKanbanColumns();
  const { visibleTasks: tasks, projectFacets, tagFacets } = useKanbanViewData();

  const activeBoard = boards.find((entry) => entry.id === activeBoardId) ?? null;

  const [createOpen, setCreateOpen] = useState(false);
  const [createDraft, setCreateDraft] = useState("");
  const [renameOpen, setRenameOpen] = useState(false);
  const [renameDraft, setRenameDraft] = useState("");

  const tasksByLane = useMemo(() => {
    const fallbackLane = columns[0] ?? "todo";
    return columns.map((column) => ({
      column,
      tasks: tasks.filter((task) => kanbanLaneFromTask(task.tags, columns, fallbackLane) === column)
    }));
  }, [columns, tasks]);

  const handleCreateBoard = () => {
    if (!createDraft.trim()) {
      return;
    }
    createBoard(createDraft.trim());
    setCreateDraft("");
    setCreateOpen(false);
  };

  const handleRenameBoard = () => {
    if (!renameDraft.trim()) {
      return;
    }
    renameBoard(renameDraft.trim());
    setRenameOpen(false);
    setRenameDraft("");
  };

  return (
    <div className="grid h-full min-h-0 grid-cols-[250px_minmax(0,1fr)_320px] gap-3 p-3">
      <Paper className="min-h-0 p-3">
        <Stack spacing={1.25} className="h-full min-h-0">
          <Typography variant="h6">Kanban Boards</Typography>
          <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
            <Button size="small" variant="outlined" startIcon={<AddIcon fontSize="small" />} onClick={() => setCreateOpen(true)}>
              New
            </Button>
            <Button
              size="small"
              variant="outlined"
              startIcon={<EditIcon fontSize="small" />}
              disabled={!activeBoard}
              onClick={() => {
                if (!activeBoard) {
                  return;
                }
                setRenameDraft(activeBoard.name);
                setRenameOpen(true);
              }}
            >
              Rename
            </Button>
            <Button
              size="small"
              variant="outlined"
              color="error"
              startIcon={<DeleteIcon fontSize="small" />}
              disabled={!activeBoard}
              onClick={() => {
                void deleteBoard();
              }}
            >
              Delete
            </Button>
          </Stack>
          <Button
            size="small"
            variant="outlined"
            startIcon={compactCards ? <ExpandIcon fontSize="small" /> : <CompressIcon fontSize="small" />}
            onClick={toggleCompact}
          >
            {compactCards ? "Show Full Cards" : "Compact Cards"}
          </Button>
          <Button
            size="small"
            variant="contained"
            disabled={!activeBoard}
            onClick={() => {
              if (!activeBoard) {
                return;
              }
              openAddTaskDialog({
                boardId: activeBoard.id,
                lockBoardSelection: true,
                allowRecurrence: true
              });
            }}
          >
            Add Task To Board
          </Button>

          <Stack spacing={1} className="min-h-0 overflow-y-auto pr-1">
            {boards.map((board) => (
              <Button
                key={board.id}
                variant={board.id === activeBoardId ? "contained" : "outlined"}
                onClick={() => setActiveBoard(board.id)}
                className="!justify-start"
                sx={{
                  borderColor: board.color,
                  color: board.id === activeBoardId ? "primary.contrastText" : board.color
                }}
              >
                {board.name}
              </Button>
            ))}
          </Stack>
        </Stack>
      </Paper>

      <Paper className="min-h-0 overflow-hidden p-3">
        <Stack spacing={1.5} className="h-full min-h-0">
          <Stack direction="row" justifyContent="space-between" alignItems="center">
            <Typography variant="h6">
              {activeBoard ? `Kanban: ${activeBoard.name}` : "Kanban"}
            </Typography>
            <Typography variant="caption" color="text.secondary">
              cards: {tasks.length}
            </Typography>
          </Stack>
          <div className="grid min-h-0 flex-1 grid-cols-3 gap-2">
            {tasksByLane.map((entry) => (
              <Box
                key={entry.column}
                onDragOver={(event) => {
                  event.preventDefault();
                  setDragOver(entry.column);
                }}
                onDrop={(event) => {
                  event.preventDefault();
                  const taskId = event.dataTransfer.getData("text/plain");
                  if (taskId) {
                    void moveTask(taskId, entry.column);
                  }
                  setDragging(null);
                  setDragOver(null);
                }}
                onDragEnter={() => setDragOver(entry.column)}
                sx={{
                  border: "1px solid",
                  borderColor: dragOverLane === entry.column ? "primary.main" : "divider",
                  borderRadius: 2,
                  p: 1,
                  backgroundColor: dragOverLane === entry.column ? "action.hover" : "background.default",
                  minHeight: 120
                }}
              >
                <Stack spacing={1} className="h-full min-h-0">
                  <Stack direction="row" justifyContent="space-between" alignItems="center">
                    <Typography variant="subtitle2">{humanizeLane(entry.column)}</Typography>
                    <Typography variant="caption" className="rounded-md border border-current/15 px-1.5 py-0.5">
                      {entry.tasks.length}
                    </Typography>
                  </Stack>
                  <Stack spacing={1} className="min-h-0 overflow-y-auto pr-1">
                    {entry.tasks.length === 0 ? (
                      <Typography variant="caption" color="text.secondary">
                        No tasks
                      </Typography>
                    ) : (
                      entry.tasks.map((task) => (
                        <KanbanCard
                          key={task.uuid}
                          task={task}
                          compact={compactCards}
                          columns={columns}
                          onMove={(nextLane) => {
                            void moveTask(task.uuid, nextLane);
                          }}
                          onDone={() => {
                            void markTaskDone(task.uuid);
                          }}
                          onDelete={() => {
                            void removeTask(task.uuid);
                          }}
                          onDragStart={(event) => {
                            event.dataTransfer.setData("text/plain", task.uuid);
                            event.dataTransfer.effectAllowed = "move";
                            setDragging(task.uuid);
                          }}
                          onDragEnd={() => {
                            setDragging(null);
                            setDragOver(null);
                          }}
                        />
                      ))
                    )}
                  </Stack>
                </Stack>
              </Box>
            ))}
          </div>
          {draggingTaskId ? (
            <Typography variant="caption" color="text.secondary">
              dragging task: {draggingTaskId}
            </Typography>
          ) : null}
        </Stack>
      </Paper>

      <Stack spacing={2} className="min-h-0">
        <Paper className="p-4">
          <Stack spacing={1}>
            <Typography variant="h6">Kanban Summary</Typography>
            <Typography variant="body2">board: {activeBoard?.name ?? "None"}</Typography>
            <Typography variant="body2">cards shown: {tasks.length}</Typography>
          </Stack>
        </Paper>

        <Paper className="p-4">
          <Stack spacing={2}>
            <Typography variant="h6">Kanban Filters</Typography>
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
            <Button variant="outlined" onClick={clearFilters}>
              Clear Filters
            </Button>
            {loading ? <Alert severity="info">Working...</Alert> : null}
            {error ? <Alert severity="error">{error}</Alert> : null}
          </Stack>
        </Paper>
      </Stack>

      <Dialog open={createOpen} onClose={() => setCreateOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>New Kanban Board</DialogTitle>
        <DialogContent dividers>
          <TextField
            autoFocus
            fullWidth
            label="Board Name"
            value={createDraft}
            onChange={(event) => setCreateDraft(event.target.value)}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateOpen(false)}>Cancel</Button>
          <Button onClick={handleCreateBoard} variant="contained" disabled={!createDraft.trim()}>
            Create
          </Button>
        </DialogActions>
      </Dialog>

      <Dialog open={renameOpen} onClose={() => setRenameOpen(false)} maxWidth="xs" fullWidth>
        <DialogTitle>Rename Kanban Board</DialogTitle>
        <DialogContent dividers>
          <TextField
            autoFocus
            fullWidth
            label="Board Name"
            value={renameDraft}
            onChange={(event) => setRenameDraft(event.target.value)}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setRenameOpen(false)}>Cancel</Button>
          <Button onClick={handleRenameBoard} variant="contained" disabled={!renameDraft.trim()}>
            Save
          </Button>
        </DialogActions>
      </Dialog>
    </div>
  );
}
