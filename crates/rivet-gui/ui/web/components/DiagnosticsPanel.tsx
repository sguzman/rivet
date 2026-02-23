import CloseIcon from "@mui/icons-material/Close";
import DeleteSweepIcon from "@mui/icons-material/DeleteSweep";
import Alert from "@mui/material/Alert";
import IconButton from "@mui/material/IconButton";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";

import type { CommandFailureRecord } from "../api/tauri";

interface DiagnosticsPanelProps {
  open: boolean;
  failures: CommandFailureRecord[];
  onClose: () => void;
  onClear: () => void;
}

export function DiagnosticsPanel(props: DiagnosticsPanelProps) {
  if (!props.open) {
    return null;
  }

  return (
    <Paper
      className="fixed bottom-3 right-3 z-[1800] w-[min(680px,calc(100vw-24px))] border border-current/20 p-3"
      elevation={8}
    >
      <Stack spacing={1}>
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Typography variant="subtitle2">Diagnostics (last invoke failures)</Typography>
          <Stack direction="row" spacing={0.25}>
            <IconButton size="small" onClick={props.onClear} title="Clear diagnostics">
              <DeleteSweepIcon fontSize="small" />
            </IconButton>
            <IconButton size="small" onClick={props.onClose} title="Close diagnostics">
              <CloseIcon fontSize="small" />
            </IconButton>
          </Stack>
        </Stack>

        {props.failures.length === 0 ? (
          <Typography variant="caption" color="text.secondary">
            No command failures recorded.
          </Typography>
        ) : (
          <div className="max-h-[260px] overflow-y-auto pr-1">
            <Stack spacing={1}>
              {props.failures.map((failure) => (
                <Alert key={`${failure.timestamp}:${failure.request_id}`} severity="error" variant="outlined">
                  <Typography variant="caption" className="block">
                    {failure.timestamp}
                  </Typography>
                  <Typography variant="body2" className="font-mono">
                    {failure.command} ({failure.duration_ms}ms)
                  </Typography>
                  <Typography variant="caption" className="font-mono">
                    request_id={failure.request_id}
                  </Typography>
                  <Typography variant="body2">{failure.error}</Typography>
                </Alert>
              ))}
            </Stack>
          </div>
        )}
      </Stack>
    </Paper>
  );
}
