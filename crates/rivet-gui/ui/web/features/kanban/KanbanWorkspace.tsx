import Alert from "@mui/material/Alert";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";

export function KanbanWorkspace() {
  return (
    <div className="p-3">
      <Paper className="p-6">
        <Stack spacing={2}>
          <Typography variant="h5">Kanban Migration Workspace</Typography>
          <Alert severity="info">
            Kanban migration is staged. This React shell is now active; board/lane/card parity is next in the roadmap.
          </Alert>
        </Stack>
      </Paper>
    </div>
  );
}
