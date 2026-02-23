import Chip from "@mui/material/Chip";

import type { TaskStatus } from "../types/core";

interface StatusChipProps {
  status: TaskStatus;
}

export function StatusChip(props: StatusChipProps) {
  if (props.status === "Completed") {
    return <Chip size="small" color="success" label="Completed" />;
  }
  if (props.status === "Deleted") {
    return <Chip size="small" color="error" label="Deleted" />;
  }
  if (props.status === "Waiting") {
    return <Chip size="small" color="warning" label="Waiting" />;
  }
  return <Chip size="small" color="primary" label="Pending" />;
}
