import List from "@mui/material/List";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemText from "@mui/material/ListItemText";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";

import { StatusChip } from "./StatusChip";
import type { TaskDto } from "../types/core";

interface TaskListPanelProps {
  tasks: TaskDto[];
  selectedTaskId: string | null;
  onSelectTask: (taskId: string) => void;
}

export function TaskListPanel(props: TaskListPanelProps) {
  return (
    <Paper className="min-h-[420px] overflow-hidden">
      <div className="border-b border-current/10 px-4 py-3">
        <Typography variant="h6">Tasks</Typography>
      </div>
      <List className="max-h-[calc(100vh-220px)] overflow-auto py-0">
        {props.tasks.map((task) => (
          <ListItemButton
            key={task.uuid}
            selected={task.uuid === props.selectedTaskId}
            onClick={() => props.onSelectTask(task.uuid)}
            className="!items-start !px-4 !py-3"
          >
            <Stack spacing={1} className="w-full">
              <Stack direction="row" justifyContent="space-between" alignItems="center" spacing={2}>
                <Typography variant="subtitle2" className="truncate">
                  {task.title || "Untitled Task"}
                </Typography>
                <StatusChip status={task.status} />
              </Stack>
              <ListItemText
                primary={task.project ?? "No project"}
                secondary={task.description || "No description"}
                primaryTypographyProps={{ variant: "caption" }}
                secondaryTypographyProps={{ variant: "caption" }}
              />
            </Stack>
          </ListItemButton>
        ))}
        {props.tasks.length === 0 ? (
          <div className="px-4 py-8 text-center text-sm opacity-70">No tasks match current filters.</div>
        ) : null}
      </List>
    </Paper>
  );
}
