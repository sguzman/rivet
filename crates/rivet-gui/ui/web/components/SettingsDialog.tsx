import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import FormControlLabel from "@mui/material/FormControlLabel";
import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

import type { DueNotificationPermission } from "../lib/notifications";
import type { DueNotificationConfig } from "../types/ui";

interface SettingsDialogProps {
  open: boolean;
  runtimeMode: string;
  loggingDirectory: string;
  themeFollowSystem: boolean;
  dueConfig: DueNotificationConfig;
  duePermission: DueNotificationPermission;
  onClose: () => void;
  onToggleThemeFollowSystem: (enabled: boolean) => void;
  onToggleEnabled: (enabled: boolean) => void;
  onTogglePreEnabled: (enabled: boolean) => void;
  onPreMinutesChange: (minutes: number) => void;
  onRequestPermission: () => void;
}

function permissionLabel(permission: DueNotificationPermission): string {
  if (permission === "granted") {
    return "Permission granted";
  }
  if (permission === "denied") {
    return "Permission denied";
  }
  if (permission === "default") {
    return "Permission not requested";
  }
  return "Notifications unsupported";
}

export function SettingsDialog(props: SettingsDialogProps) {
  return (
    <Dialog open={props.open} onClose={props.onClose} maxWidth="sm" fullWidth>
      <DialogTitle>Settings</DialogTitle>
      <DialogContent dividers>
        <Stack spacing={2.25}>
          <Stack spacing={0.5}>
            <Typography variant="subtitle2">Runtime</Typography>
            <Typography variant="body2">mode: {props.runtimeMode}</Typography>
            <Typography variant="body2">logs: {props.loggingDirectory}</Typography>
          </Stack>

          <Stack spacing={1.25}>
            <Typography variant="subtitle2">Theme</Typography>
            <FormControlLabel
              control={(
                <Switch
                  checked={props.themeFollowSystem}
                  onChange={(event) => props.onToggleThemeFollowSystem(event.target.checked)}
                />
              )}
              label="Follow system day/night theme"
            />
          </Stack>

          <Stack spacing={1.25}>
            <Typography variant="subtitle2">Due Notifications</Typography>
            <FormControlLabel
              control={(
                <Switch
                  checked={props.dueConfig.enabled}
                  onChange={(event) => props.onToggleEnabled(event.target.checked)}
                />
              )}
              label="Enable OS due notifications"
            />

            <FormControlLabel
              control={(
                <Switch
                  checked={props.dueConfig.pre_notify_enabled}
                  disabled={!props.dueConfig.enabled}
                  onChange={(event) => props.onTogglePreEnabled(event.target.checked)}
                />
              )}
              label="Enable pre-notify"
            />

            <TextField
              label="Pre-notify minutes before due"
              type="number"
              value={props.dueConfig.pre_notify_minutes}
              onChange={(event) => props.onPreMinutesChange(Number(event.target.value) || 1)}
              inputProps={{ min: 1, max: 43_200 }}
              disabled={!props.dueConfig.enabled || !props.dueConfig.pre_notify_enabled}
              size="small"
            />

            <Typography variant="caption" color="text.secondary">
              Permission: {permissionLabel(props.duePermission)}
            </Typography>

            <Stack direction="row" spacing={1}>
              <Button
                variant="outlined"
                size="small"
                onClick={props.onRequestPermission}
                disabled={props.duePermission === "unsupported"}
              >
                Request Notification Permission
              </Button>
            </Stack>
          </Stack>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={props.onClose}>Close</Button>
      </DialogActions>
    </Dialog>
  );
}
