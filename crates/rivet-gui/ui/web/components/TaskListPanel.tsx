import { useMemo, useRef } from "react";

import { useVirtualizer } from "@tanstack/react-virtual";
import Checkbox from "@mui/material/Checkbox";
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
  selectMode: boolean;
  selectedTaskIds: string[];
  onTaskClick: (taskId: string, index: number, modifiers: { ctrlOrMeta: boolean; shift: boolean }) => void;
}

export function TaskListPanel(props: TaskListPanelProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const selectedTaskSet = useMemo(() => new Set(props.selectedTaskIds), [props.selectedTaskIds]);
  const virtualizer = useVirtualizer({
    count: props.tasks.length,
    getScrollElement: () => parentRef.current,
    getItemKey: (index) => props.tasks[index]?.uuid ?? index,
    estimateSize: () => 128,
    overscan: 10
  });

  return (
    <Paper className="min-h-[420px] overflow-hidden">
      <div className="border-b border-current/10 px-4 py-3">
        <Typography variant="h6">Tasks</Typography>
      </div>
      <div ref={parentRef} className="max-h-[calc(100vh-220px)] overflow-auto">
        {props.tasks.length === 0 ? (
          <div className="px-4 py-8 text-center text-sm opacity-70">No tasks match current filters.</div>
        ) : (
          <List
            className="relative py-0"
            sx={{
              height: `${virtualizer.getTotalSize()}px`
            }}
          >
            {virtualizer.getVirtualItems().map((item) => {
              const task = props.tasks[item.index];
              if (!task) {
                return null;
              }
              const isSelected = props.selectMode
                ? selectedTaskSet.has(task.uuid)
                : task.uuid === props.selectedTaskId;
              return (
                <ListItemButton
                  key={task.uuid}
                  selected={isSelected}
                  onClick={(event) => {
                    props.onTaskClick(task.uuid, item.index, {
                      ctrlOrMeta: event.ctrlKey || event.metaKey,
                      shift: event.shiftKey
                    });
                  }}
                  className="!absolute !left-0 !right-0 !items-start !px-4 !py-3"
                  data-index={item.index}
                  ref={virtualizer.measureElement}
                  sx={{
                    transform: `translateY(${item.start}px)`
                  }}
                >
                  <Stack spacing={1} className="w-full">
                    <Stack direction="row" justifyContent="space-between" alignItems="center" spacing={2}>
                      <Stack direction="row" spacing={1} alignItems="center" className="min-w-0 flex-1">
                        {props.selectMode ? (
                          <Checkbox
                            size="small"
                            checked={isSelected}
                            tabIndex={-1}
                            disableRipple
                            sx={{ p: 0.25 }}
                          />
                        ) : null}
                        <Typography
                          variant="subtitle2"
                          sx={{
                            whiteSpace: "normal",
                            wordBreak: "break-word",
                            overflowWrap: "anywhere"
                          }}
                        >
                          {task.title || "Untitled Task"}
                        </Typography>
                      </Stack>
                      <StatusChip status={task.status} />
                    </Stack>
                    <ListItemText
                      primary={task.project ?? "No project"}
                      secondary={task.description || "No description"}
                      primaryTypographyProps={{
                        variant: "caption",
                        sx: { whiteSpace: "normal", overflowWrap: "anywhere" }
                      }}
                      secondaryTypographyProps={{
                        variant: "caption",
                        sx: { whiteSpace: "normal", overflowWrap: "anywhere" }
                      }}
                    />
                  </Stack>
                </ListItemButton>
              );
            })}
          </List>
        )}
      </div>
    </Paper>
  );
}
