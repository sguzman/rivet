import { useMemo, useState } from "react";

import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

import type { TaskCreate } from "../types/core";

interface AddTaskDialogProps {
  open: boolean;
  busy: boolean;
  onClose: () => void;
  onSubmit: (input: TaskCreate) => Promise<void>;
}

const EMPTY_FORM = {
  title: "",
  description: "",
  project: "",
  tags: "",
  due: ""
};

export function AddTaskDialog(props: AddTaskDialogProps) {
  const [title, setTitle] = useState(EMPTY_FORM.title);
  const [description, setDescription] = useState(EMPTY_FORM.description);
  const [project, setProject] = useState(EMPTY_FORM.project);
  const [tags, setTags] = useState(EMPTY_FORM.tags);
  const [due, setDue] = useState(EMPTY_FORM.due);
  const [error, setError] = useState<string | null>(null);

  const canSave = useMemo(() => title.trim().length > 0 && !props.busy, [title, props.busy]);

  const resetForm = () => {
    setTitle(EMPTY_FORM.title);
    setDescription(EMPTY_FORM.description);
    setProject(EMPTY_FORM.project);
    setTags(EMPTY_FORM.tags);
    setDue(EMPTY_FORM.due);
    setError(null);
  };

  const handleClose = () => {
    resetForm();
    props.onClose();
  };

  const handleSave = async () => {
    if (title.trim().length === 0) {
      setError("Title is required");
      return;
    }

    const tagList = tags
      .split(/\s+/)
      .map((value) => value.trim())
      .filter(Boolean);

    await props.onSubmit({
      title: title.trim(),
      description: description.trim(),
      project: project.trim() ? project.trim() : null,
      tags: tagList,
      priority: null,
      due: due.trim() ? due.trim() : null,
      wait: null,
      scheduled: null
    });

    resetForm();
  };

  return (
    <Dialog open={props.open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>Add Task</DialogTitle>
      <DialogContent>
        <Stack spacing={2} className="pt-2">
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
            label="Description"
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
            label="Tags (space separated)"
            value={tags}
            onChange={(event) => setTags(event.target.value)}
          />
          <TextField
            label="Due"
            placeholder="e.g. tomorrow, 2026-12-01"
            value={due}
            onChange={(event) => setDue(event.target.value)}
          />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose} disabled={props.busy}>
          Cancel
        </Button>
        <Button onClick={handleSave} disabled={!canSave} variant="contained">
          {props.busy ? "Saving..." : "Save"}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
