import { useEffect, useMemo, useState } from "react";

import AddIcon from "@mui/icons-material/Add";
import Button from "@mui/material/Button";
import Chip from "@mui/material/Chip";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import MenuItem from "@mui/material/MenuItem";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

import {
  BOARD_TAG_KEY,
  boardIdFromTaskTags,
  collectTagsForSubmit,
  defaultKanbanLane,
  isSingleSelectKey,
  recurrenceFromTags,
  removeTagsForKey,
  splitTags,
  tagColorStyle
} from "../lib/tags";
import type { TagSchema } from "../types/config";
import type { TaskDto, TaskPatch } from "../types/core";
import type { KanbanBoardDef, RecurrenceDraft } from "../types/ui";

interface TaskEditDialogProps {
  open: boolean;
  task: TaskDto | null;
  busy: boolean;
  tagSchema: TagSchema | null;
  tagColorMap: Record<string, string>;
  kanbanBoards: KanbanBoardDef[];
  onClose: () => void;
  onSubmit: (uuid: string, patch: TaskPatch) => Promise<boolean>;
}

const EMPTY_RECURRENCE: RecurrenceDraft = {
  pattern: "none",
  time: "",
  days: [],
  months: [],
  monthDay: ""
};

export function TaskEditDialog(props: TaskEditDialogProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [project, setProject] = useState("");
  const [due, setDue] = useState("");
  const [customTagInput, setCustomTagInput] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [pickerKey, setPickerKey] = useState<string>("");
  const [pickerValue, setPickerValue] = useState<string>("");
  const [boardId, setBoardId] = useState<string>("");
  const [recurrence, setRecurrence] = useState<RecurrenceDraft>(EMPTY_RECURRENCE);
  const [error, setError] = useState<string | null>(null);

  const keyOptions = useMemo(() => {
    return (props.tagSchema?.keys ?? []).filter((entry) => entry.id !== BOARD_TAG_KEY);
  }, [props.tagSchema]);

  const pickerValueOptions = useMemo(() => {
    const selected = keyOptions.find((entry) => entry.id === pickerKey);
    return selected?.values ?? [];
  }, [keyOptions, pickerKey]);

  useEffect(() => {
    if (!props.open || !props.task) {
      return;
    }
    const firstKey = keyOptions[0];
    const firstValue = firstKey?.values?.[0] ?? "";
    setTitle(props.task.title);
    setDescription(props.task.description);
    setProject(props.task.project ?? "");
    setDue(props.task.due ?? "");
    setCustomTagInput("");
    setSelectedTags([...props.task.tags]);
    setPickerKey(firstKey?.id ?? "");
    setPickerValue(firstValue);
    setBoardId(boardIdFromTaskTags(props.task.tags) ?? "");
    setRecurrence(recurrenceFromTags(props.task.tags));
    setError(null);
  }, [props.open, props.task, keyOptions]);

  const canSave = useMemo(() => props.task !== null && title.trim().length > 0 && !props.busy, [props.task, title, props.busy]);

  const handleAddCustomTags = () => {
    const values = splitTags(customTagInput);
    if (values.length === 0) {
      return;
    }
    setSelectedTags((prev) => {
      const next = [...prev];
      for (const value of values) {
        if (!next.includes(value)) {
          next.push(value);
        }
      }
      return next;
    });
    setCustomTagInput("");
  };

  const handleAddPickerTag = () => {
    if (!pickerKey || !pickerValue) {
      return;
    }
    const tag = `${pickerKey}:${pickerValue}`;
    setSelectedTags((prev) => {
      const next = [...prev];
      if (isSingleSelectKey(props.tagSchema, pickerKey)) {
        removeTagsForKey(next, pickerKey);
      }
      if (!next.includes(tag)) {
        next.push(tag);
      }
      return next;
    });
  };

  const handleSave = async () => {
    if (!props.task) {
      return;
    }
    if (!title.trim()) {
      setError("Title is required.");
      return;
    }

    const tags = collectTagsForSubmit({
      selectedTags,
      customTagInput,
      boardTag: boardId.trim() ? `${BOARD_TAG_KEY}:${boardId.trim()}` : null,
      allowRecurrence: true,
      recurrence,
      ensureKanbanLane: true,
      defaultKanbanLaneValue: defaultKanbanLane(props.tagSchema)
    });

    const patch: TaskPatch = {
      title: title.trim(),
      description: description.trim(),
      project: project.trim() ? project.trim() : null,
      due: due.trim() ? due.trim() : null,
      tags
    };

    const ok = await props.onSubmit(props.task.uuid, patch);
    if (ok) {
      props.onClose();
    }
  };

  return (
    <Dialog open={props.open} onClose={props.onClose} maxWidth="md" fullWidth>
      <DialogTitle>Edit Task</DialogTitle>
      <DialogContent dividers className="max-h-[calc(100vh-160px)]">
        <Stack spacing={2.25}>
          {error ? <Typography color="error">{error}</Typography> : null}

          <TextField
            label="Title"
            required
            value={title}
            onChange={(event) => {
              setTitle(event.target.value);
              if (error) {
                setError(null);
              }
            }}
          />

          <TextField
            label="Description (optional)"
            value={description}
            onChange={(event) => setDescription(event.target.value)}
            multiline
            minRows={2}
          />

          <TextField
            label="Project"
            value={project}
            onChange={(event) => setProject(event.target.value)}
          />

          <TextField
            select
            label="Kanban Board"
            value={boardId}
            onChange={(event) => setBoardId(event.target.value)}
          >
            <MenuItem value="">No board</MenuItem>
            {props.kanbanBoards.map((board) => (
              <MenuItem key={board.id} value={board.id}>
                {board.name}
              </MenuItem>
            ))}
          </TextField>

          <Stack direction={{ xs: "column", sm: "row" }} spacing={1.25}>
            <TextField
              label="Custom Tags"
              placeholder="topic:rust urgent"
              value={customTagInput}
              onChange={(event) => setCustomTagInput(event.target.value)}
              className="flex-1"
            />
            <Button variant="outlined" startIcon={<AddIcon fontSize="small" />} onClick={handleAddCustomTags}>
              Add
            </Button>
          </Stack>

          <Stack direction={{ xs: "column", sm: "row" }} spacing={1.25}>
            <TextField
              select
              label="Tag Key"
              value={pickerKey}
              onChange={(event) => {
                const key = event.target.value;
                setPickerKey(key);
                const firstValue = keyOptions.find((entry) => entry.id === key)?.values?.[0] ?? "";
                setPickerValue(firstValue);
              }}
              className="flex-1"
            >
              {keyOptions.length === 0 ? <MenuItem value="">No tag keys configured</MenuItem> : null}
              {keyOptions.map((entry) => (
                <MenuItem key={entry.id} value={entry.id}>
                  {entry.label ?? entry.id} ({entry.id})
                </MenuItem>
              ))}
            </TextField>
            <TextField
              select
              label="Tag Value"
              value={pickerValue}
              onChange={(event) => setPickerValue(event.target.value)}
              className="flex-1"
              disabled={!pickerKey}
            >
              {pickerValueOptions.length === 0 ? <MenuItem value="">No values</MenuItem> : null}
              {pickerValueOptions.map((entry) => (
                <MenuItem key={entry} value={entry}>
                  {entry}
                </MenuItem>
              ))}
            </TextField>
            <Button variant="outlined" startIcon={<AddIcon fontSize="small" />} onClick={handleAddPickerTag} disabled={!pickerKey || !pickerValue}>
              Add
            </Button>
          </Stack>

          <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
            {selectedTags.length === 0 ? (
              <Typography variant="body2" color="text.secondary">
                No tags selected yet.
              </Typography>
            ) : (
              selectedTags.map((tag) => {
                const color = tagColorStyle(tag, props.tagSchema, props.tagColorMap);
                return (
                  <Chip
                    key={tag}
                    label={tag}
                    onDelete={() => setSelectedTags((prev) => prev.filter((value) => value !== tag))}
                    size="small"
                    sx={{
                      borderColor: color,
                      color,
                      borderWidth: 1,
                      borderStyle: "solid",
                      "& .MuiChip-label": {
                        fontFamily: "\"Source Code Pro\", monospace",
                        fontSize: "0.72rem"
                      }
                    }}
                  />
                );
              })
            )}
          </Stack>

          <TextField
            label="Due"
            placeholder="e.g. tomorrow, 2028, march, wed, 3:23pm, 2026-02-20"
            value={due}
            onChange={(event) => setDue(event.target.value)}
          />

          <TextField
            select
            label="Recurrence"
            value={recurrence.pattern}
            onChange={(event) => setRecurrence((prev) => ({ ...prev, pattern: event.target.value as RecurrenceDraft["pattern"] }))}
          >
            <MenuItem value="none">None</MenuItem>
            <MenuItem value="daily">Daily</MenuItem>
            <MenuItem value="weekly">Weekly</MenuItem>
            <MenuItem value="months">Months</MenuItem>
            <MenuItem value="monthly">Monthly</MenuItem>
            <MenuItem value="yearly">Yearly</MenuItem>
          </TextField>

          {recurrence.pattern !== "none" ? (
            <TextField
              label="Recurring Time"
              value={recurrence.time}
              placeholder="03:23pm or 15:23"
              onChange={(event) => setRecurrence((prev) => ({ ...prev, time: event.target.value }))}
            />
          ) : null}

          {recurrence.pattern === "weekly" ? (
            <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
              {["mon", "tue", "wed", "thu", "fri", "sat", "sun"].map((day) => {
                const active = recurrence.days.includes(day);
                return (
                  <Button
                    key={day}
                    variant={active ? "contained" : "outlined"}
                    size="small"
                    onClick={() => {
                      setRecurrence((prev) => {
                        const exists = prev.days.includes(day);
                        return {
                          ...prev,
                          days: exists ? prev.days.filter((entry) => entry !== day) : [...prev.days, day]
                        };
                      });
                    }}
                  >
                    {day.toUpperCase()}
                  </Button>
                );
              })}
            </Stack>
          ) : null}

          {(recurrence.pattern === "months" || recurrence.pattern === "monthly" || recurrence.pattern === "yearly") ? (
            <>
              <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                {["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"].map((month) => {
                  const active = recurrence.months.includes(month);
                  return (
                    <Button
                      key={month}
                      variant={active ? "contained" : "outlined"}
                      size="small"
                      onClick={() => {
                        setRecurrence((prev) => {
                          const exists = prev.months.includes(month);
                          return {
                            ...prev,
                            months: exists ? prev.months.filter((entry) => entry !== month) : [...prev.months, month]
                          };
                        });
                      }}
                    >
                      {month.toUpperCase()}
                    </Button>
                  );
                })}
              </Stack>
              <TextField
                label="Month Day(s)"
                value={recurrence.monthDay}
                placeholder="1 or 1,15,28"
                onChange={(event) => setRecurrence((prev) => ({ ...prev, monthDay: event.target.value }))}
              />
            </>
          ) : null}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={props.onClose} disabled={props.busy}>
          Cancel
        </Button>
        <Button onClick={handleSave} disabled={!canSave} variant="contained">
          {props.busy ? "Saving..." : "Save"}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
