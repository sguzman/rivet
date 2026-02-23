import Alert from "@mui/material/Alert";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";

import { useAppStore } from "../../store/useAppStore";

export function CalendarWorkspace() {
  const runtimeConfig = useAppStore((state) => state.runtimeConfig);
  const timezone = runtimeConfig?.calendar?.timezone ?? runtimeConfig?.time?.timezone ?? runtimeConfig?.timezone ?? "(default)";

  return (
    <div className="p-3">
      <Paper className="p-6">
        <Stack spacing={2}>
          <Typography variant="h5">Calendar Migration Workspace</Typography>
          <Typography variant="body2" color="text.secondary">
            Effective timezone from config snapshot: {timezone}
          </Typography>
          <Alert severity="info">
            Calendar migration is staged. Year/quarter/month/week/day parity and marker rendering will be ported next.
          </Alert>
        </Stack>
      </Paper>
    </div>
  );
}
