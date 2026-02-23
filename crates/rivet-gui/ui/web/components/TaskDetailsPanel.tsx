import Button from "@mui/material/Button";
import Divider from "@mui/material/Divider";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";

import { StatusChip } from "./StatusChip";
import { TagChip } from "./TagChip";
import type { TaskDto } from "../types/core";

interface TaskDetailsPanelProps {
  task: TaskDto | null;
  busy: boolean;
  onEdit: (taskId: string) => void;
  onDone: (taskId: string) => void;
  onDelete: (taskId: string) => void;
}

export function TaskDetailsPanel(props: TaskDetailsPanelProps) {
  return (
    <Paper className="min-h-[420px] p-4">
      <Typography variant="h6" gutterBottom>
        Task Details
      </Typography>
      {props.task ? (
        <Stack spacing={2}>
          <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={2}>
            <Typography variant="subtitle1">{props.task.title || "Untitled Task"}</Typography>
            <StatusChip status={props.task.status} />
          </Stack>
          <Divider />
          <Stack spacing={1}>
            <Typography variant="caption" color="text.secondary">
              Description
            </Typography>
            <Typography variant="body2">{props.task.description || "No description"}</Typography>
          </Stack>
          <Stack spacing={1}>
            <Typography variant="caption" color="text.secondary">
              Project
            </Typography>
            <Typography variant="body2">{props.task.project || "None"}</Typography>
          </Stack>
          <Stack spacing={1}>
            <Typography variant="caption" color="text.secondary">
              Due
            </Typography>
            <Typography variant="body2">{props.task.due || "No due date"}</Typography>
          </Stack>
          <Stack spacing={1}>
            <Typography variant="caption" color="text.secondary">
              Tags
            </Typography>
            <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
              {props.task.tags.length > 0
                ? props.task.tags.map((tag) => <TagChip key={tag} tag={tag} size="small" />)
                : <Typography variant="body2">No tags</Typography>}
            </Stack>
          </Stack>
          <Divider />
          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              disabled={props.busy}
              onClick={() => props.onEdit(props.task!.uuid)}
            >
              Edit
            </Button>
            <Button
              variant="contained"
              color="success"
              disabled={props.busy || props.task.status === "Completed"}
              onClick={() => props.onDone(props.task!.uuid)}
            >
              Done
            </Button>
            <Button
              variant="outlined"
              color="error"
              disabled={props.busy}
              onClick={() => props.onDelete(props.task!.uuid)}
            >
              Delete
            </Button>
          </Stack>
        </Stack>
      ) : (
        <Typography variant="body2" color="text.secondary">
          Select a task to inspect details.
        </Typography>
      )}
    </Paper>
  );
}
