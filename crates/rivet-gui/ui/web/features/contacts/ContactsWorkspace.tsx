import { useEffect, useMemo, useRef, useState } from "react";
import type { ChangeEvent } from "react";

import DeleteIcon from "@mui/icons-material/Delete";
import LinkIcon from "@mui/icons-material/Link";
import PersonAddIcon from "@mui/icons-material/PersonAdd";
import RedoIcon from "@mui/icons-material/Redo";
import UploadFileIcon from "@mui/icons-material/UploadFile";
import Alert from "@mui/material/Alert";
import Button from "@mui/material/Button";
import Checkbox from "@mui/material/Checkbox";
import Divider from "@mui/material/Divider";
import IconButton from "@mui/material/IconButton";
import MenuItem from "@mui/material/MenuItem";
import Paper from "@mui/material/Paper";
import Stack from "@mui/material/Stack";
import TextField from "@mui/material/TextField";
import Typography from "@mui/material/Typography";

import { useContactsStore } from "../../store/useContactsStore";
import type { ContactFieldValue } from "../../types/core";

function fieldTemplate(kind: string): ContactFieldValue {
  return {
    value: "",
    kind,
    is_primary: false
  };
}

export function ContactsWorkspace() {
  const {
    bootstrap,
    loading,
    error,
    query,
    contacts,
    selectedContactId,
    selectionMode,
    selectionIds,
    dedupe,
    importPreview,
    importCommitResult,
    mergeUndoResult,
    draft,
    setQuery,
    selectContact,
    toggleSelectionMode,
    toggleSelected,
    setSelectionIds,
    clearSelection,
    setDraft,
    resetDraft,
    loadDraftFromSelected,
    createFromDraft,
    updateSelectedFromDraft,
    removeSelectedContact,
    removeBulkSelected,
    refreshDedupe,
    mergeSelected,
    undoLastMerge,
    previewImport,
    commitImport,
    openAction
  } = useContactsStore();

  const [searchInput, setSearchInput] = useState(query);
  const [importSource, setImportSource] = useState("gmail_export");
  const [importMode, setImportMode] = useState<"safe" | "upsert" | "review">("safe");
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  useEffect(() => {
    const id = window.setTimeout(() => {
      void setQuery(searchInput);
    }, 180);
    return () => window.clearTimeout(id);
  }, [searchInput, setQuery]);

  useEffect(() => {
    void refreshDedupe();
  }, [refreshDedupe]);

  const selectedContact = useMemo(
    () => contacts.find((contact) => contact.id === selectedContactId) ?? null,
    [contacts, selectedContactId]
  );

  const selectionSet = useMemo(() => new Set(selectionIds), [selectionIds]);

  const updateDraftField = (key: keyof typeof draft, value: unknown) => {
    setDraft({
      ...draft,
      [key]: value
    });
  };

  const updateDraftFieldArray = (key: "phones" | "emails" | "websites", next: ContactFieldValue[]) => {
    setDraft({
      ...draft,
      [key]: next
    });
  };

  const handleImportFile = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    const text = await file.text();
    await previewImport(importSource, file.name, text);
    event.target.value = "";
  };

  return (
    <div className="grid h-full min-h-0 grid-cols-[360px_minmax(0,1fr)] gap-3 p-3">
      <Paper className="min-h-0 p-3">
        <Stack spacing={1.25} className="h-full min-h-0">
          <Typography variant="h6">Contacts</Typography>

          <TextField
            label="Search contacts"
            value={searchInput}
            onChange={(event) => setSearchInput(event.target.value)}
            size="small"
          />

          <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
            <Button size="small" variant={selectionMode ? "contained" : "outlined"} onClick={toggleSelectionMode}>
              {selectionMode ? "Exit Select" : "Select"}
            </Button>
            <Button size="small" variant="outlined" onClick={() => void refreshDedupe()}>
              Refresh Dedupe
            </Button>
            <Button
              size="small"
              variant="outlined"
              color="error"
              startIcon={<DeleteIcon fontSize="small" />}
              disabled={selectionIds.length === 0}
              onClick={() => {
                void removeBulkSelected();
              }}
            >
              Delete Selected
            </Button>
          </Stack>

          <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
            <Button
              size="small"
              variant="outlined"
              startIcon={<LinkIcon fontSize="small" />}
              disabled={selectionIds.length < 2}
              onClick={() => {
                void mergeSelected();
              }}
            >
              Merge Selected
            </Button>
            <Button
              size="small"
              variant="outlined"
              startIcon={<RedoIcon fontSize="small" />}
              onClick={() => {
                void undoLastMerge();
              }}
            >
              Undo Merge
            </Button>
            <Button
              size="small"
              variant="outlined"
              onClick={() => {
                clearSelection();
              }}
              disabled={selectionIds.length === 0}
            >
              Clear Selection
            </Button>
          </Stack>

          <Divider />

          <Stack spacing={1}>
            <Typography variant="subtitle2">Import</Typography>
            <TextField
              select
              size="small"
              label="Source"
              value={importSource}
              onChange={(event) => setImportSource(event.target.value)}
            >
              <MenuItem value="gmail_export">Gmail Export</MenuItem>
              <MenuItem value="iphone_export">iPhone/iCloud Export</MenuItem>
              <MenuItem value="generic_vcard">Generic vCard</MenuItem>
            </TextField>
            <TextField
              select
              size="small"
              label="Import Mode"
              value={importMode}
              onChange={(event) => setImportMode(event.target.value as typeof importMode)}
            >
              <MenuItem value="safe">Safe (skip conflicts)</MenuItem>
              <MenuItem value="upsert">Upsert (merge conflicts)</MenuItem>
              <MenuItem value="review">Review (preview first)</MenuItem>
            </TextField>
            <Stack direction="row" spacing={1}>
              <Button
                size="small"
                variant="outlined"
                startIcon={<UploadFileIcon fontSize="small" />}
                onClick={() => fileInputRef.current?.click()}
              >
                Choose .vcf
              </Button>
              <Button
                size="small"
                variant="contained"
                disabled={!importPreview}
                onClick={() => {
                  void commitImport(importMode);
                }}
              >
                Commit Import
              </Button>
            </Stack>
            <input
              ref={fileInputRef}
              type="file"
              accept=".vcf,text/vcard,text/x-vcard"
              className="hidden"
              onChange={(event) => {
                void handleImportFile(event);
              }}
            />

            {importPreview ? (
              <Typography variant="caption" color="text.secondary">
                preview rows: {importPreview.total_rows} valid:{" "}
                {importPreview.valid_rows} duplicates:{" "}
                {importPreview.potential_duplicates}
              </Typography>
            ) : null}
            {importCommitResult ? (
              <Typography variant="caption" color="text.secondary">
                import result: +{importCommitResult.created} ~{importCommitResult.updated} skip:{" "}
                {importCommitResult.skipped} fail: {importCommitResult.failed}
              </Typography>
            ) : null}
            {mergeUndoResult ? (
              <Typography variant="caption" color="text.secondary">
                merge undo restored: {mergeUndoResult.restored}
              </Typography>
            ) : null}
          </Stack>

          <Divider />

          <Typography variant="caption" color="text.secondary">
            contacts: {contacts.length}
          </Typography>

          <div className="min-h-0 flex-1 overflow-y-auto pr-1">
            <Stack spacing={0.6}>
              {contacts.map((contact) => (
                <Paper
                  key={contact.id}
                  variant={selectedContactId === contact.id ? "elevation" : "outlined"}
                  className="cursor-pointer p-2"
                  onClick={() => {
                    selectContact(contact.id);
                    if (selectionMode) {
                      toggleSelected(contact.id);
                    }
                  }}
                >
                  <Stack direction="row" spacing={1} alignItems="center">
                    {selectionMode ? (
                      <Checkbox
                        size="small"
                        checked={selectionSet.has(contact.id)}
                        onChange={() => toggleSelected(contact.id)}
                        onClick={(event) => event.stopPropagation()}
                      />
                    ) : null}
                    <Stack className="min-w-0 flex-1">
                      <Typography variant="subtitle2" className="truncate">
                        {contact.display_name || "Unnamed Contact"}
                      </Typography>
                      <Typography variant="caption" color="text.secondary" className="truncate">
                        {contact.emails[0]?.value ?? contact.phones[0]?.value ?? "No email/phone"}
                      </Typography>
                    </Stack>
                  </Stack>
                </Paper>
              ))}
            </Stack>
          </div>
        </Stack>
      </Paper>

      <Stack spacing={2} className="min-h-0">
        <Paper className="p-4">
          <Stack spacing={1.25}>
            <Typography variant="h6">Add / Edit Contact</Typography>
            <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
              <Button
                size="small"
                variant="outlined"
                onClick={loadDraftFromSelected}
                disabled={!selectedContact}
              >
                Load Selected Into Form
              </Button>
              <Button
                size="small"
                variant="outlined"
                onClick={resetDraft}
              >
                Reset Form
              </Button>
              <Button
                size="small"
                variant="contained"
                startIcon={<PersonAddIcon fontSize="small" />}
                onClick={() => {
                  void createFromDraft();
                }}
              >
                Create Contact
              </Button>
              <Button
                size="small"
                variant="outlined"
                disabled={!selectedContact}
                onClick={() => {
                  void updateSelectedFromDraft();
                }}
              >
                Update Selected
              </Button>
              <Button
                size="small"
                variant="outlined"
                color="error"
                disabled={!selectedContact}
                onClick={() => {
                  void removeSelectedContact();
                }}
              >
                Delete Selected
              </Button>
            </Stack>

            <TextField
              label="Display Name"
              value={draft.display_name ?? ""}
              onChange={(event) => updateDraftField("display_name", event.target.value)}
              size="small"
            />

            <Stack direction={{ xs: "column", md: "row" }} spacing={1}>
              <TextField
                label="Given Name"
                value={draft.given_name ?? ""}
                onChange={(event) => updateDraftField("given_name", event.target.value)}
                size="small"
                className="flex-1"
              />
              <TextField
                label="Family Name"
                value={draft.family_name ?? ""}
                onChange={(event) => updateDraftField("family_name", event.target.value)}
                size="small"
                className="flex-1"
              />
              <TextField
                label="Nickname"
                value={draft.nickname ?? ""}
                onChange={(event) => updateDraftField("nickname", event.target.value)}
                size="small"
                className="flex-1"
              />
            </Stack>

            <Stack direction={{ xs: "column", md: "row" }} spacing={1}>
              <TextField
                label="Organization"
                value={draft.organization ?? ""}
                onChange={(event) => updateDraftField("organization", event.target.value)}
                size="small"
                className="flex-1"
              />
              <TextField
                label="Title"
                value={draft.title ?? ""}
                onChange={(event) => updateDraftField("title", event.target.value)}
                size="small"
                className="flex-1"
              />
              <TextField
                label="Birthday"
                type="date"
                value={draft.birthday ?? ""}
                onChange={(event) => updateDraftField("birthday", event.target.value || null)}
                size="small"
                className="flex-1"
                InputLabelProps={{ shrink: true }}
              />
            </Stack>

            <TextField
              label="Notes"
              value={draft.notes ?? ""}
              onChange={(event) => updateDraftField("notes", event.target.value)}
              size="small"
              multiline
              minRows={3}
            />

            <Divider />

            <Stack spacing={0.8}>
              <Typography variant="subtitle2">Emails</Typography>
              {draft.emails.map((email, index) => (
                <Stack key={`email-${index}`} direction="row" spacing={1} alignItems="center">
                  <TextField
                    label="Email"
                    size="small"
                    value={email.value}
                    onChange={(event) => {
                      const next = [...draft.emails];
                      next[index] = { ...next[index], value: event.target.value };
                      updateDraftFieldArray("emails", next);
                    }}
                    className="flex-1"
                  />
                  <TextField
                    label="Type"
                    size="small"
                    value={email.kind}
                    onChange={(event) => {
                      const next = [...draft.emails];
                      next[index] = { ...next[index], kind: event.target.value };
                      updateDraftFieldArray("emails", next);
                    }}
                    className="w-32"
                  />
                  <Checkbox
                    checked={email.is_primary}
                    onChange={(event) => {
                      const next = draft.emails.map((entry, i) => ({ ...entry, is_primary: i === index ? event.target.checked : false }));
                      updateDraftFieldArray("emails", next);
                    }}
                  />
                  <IconButton
                    size="small"
                    onClick={() => {
                      const next = draft.emails.filter((_, i) => i !== index);
                      updateDraftFieldArray("emails", next.length > 0 ? next : [fieldTemplate("home")]);
                    }}
                  >
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </Stack>
              ))}
              <Button
                size="small"
                variant="outlined"
                onClick={() => {
                  updateDraftFieldArray("emails", [...draft.emails, fieldTemplate("home")]);
                }}
              >
                Add Email
              </Button>
            </Stack>

            <Stack spacing={0.8}>
              <Typography variant="subtitle2">Phones</Typography>
              {draft.phones.map((phone, index) => (
                <Stack key={`phone-${index}`} direction="row" spacing={1} alignItems="center">
                  <TextField
                    label="Phone"
                    size="small"
                    value={phone.value}
                    onChange={(event) => {
                      const next = [...draft.phones];
                      next[index] = { ...next[index], value: event.target.value };
                      updateDraftFieldArray("phones", next);
                    }}
                    className="flex-1"
                  />
                  <TextField
                    label="Type"
                    size="small"
                    value={phone.kind}
                    onChange={(event) => {
                      const next = [...draft.phones];
                      next[index] = { ...next[index], kind: event.target.value };
                      updateDraftFieldArray("phones", next);
                    }}
                    className="w-32"
                  />
                  <Checkbox
                    checked={phone.is_primary}
                    onChange={(event) => {
                      const next = draft.phones.map((entry, i) => ({ ...entry, is_primary: i === index ? event.target.checked : false }));
                      updateDraftFieldArray("phones", next);
                    }}
                  />
                  <IconButton
                    size="small"
                    onClick={() => {
                      const next = draft.phones.filter((_, i) => i !== index);
                      updateDraftFieldArray("phones", next.length > 0 ? next : [fieldTemplate("mobile")]);
                    }}
                  >
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </Stack>
              ))}
              <Button
                size="small"
                variant="outlined"
                onClick={() => {
                  updateDraftFieldArray("phones", [...draft.phones, fieldTemplate("mobile")]);
                }}
              >
                Add Phone
              </Button>
            </Stack>

            {selectedContact ? (
              <Stack direction="row" spacing={1}>
                <Button
                  size="small"
                  variant="outlined"
                  disabled={selectedContact.emails.length === 0}
                  onClick={() => {
                    void openAction({ id: selectedContact.id, action: "mailto", value: selectedContact.emails[0]?.value ?? null });
                  }}
                >
                  Email
                </Button>
                <Button
                  size="small"
                  variant="outlined"
                  disabled={selectedContact.phones.length === 0}
                  onClick={() => {
                    void openAction({ id: selectedContact.id, action: "tel", value: selectedContact.phones[0]?.value ?? null });
                  }}
                >
                  Call
                </Button>
              </Stack>
            ) : null}
          </Stack>
        </Paper>

        <Paper className="min-h-0 p-4">
          <Stack spacing={1}>
            <Typography variant="h6">Dedup Center</Typography>
            {dedupe && dedupe.groups.length > 0 ? (
              <Stack spacing={1} className="max-h-[280px] overflow-y-auto pr-1">
                {dedupe.groups.slice(0, 20).map((group) => (
                  <Paper key={group.group_id} variant="outlined" className="p-2">
                    <Stack spacing={0.6}>
                      <Typography variant="subtitle2">{group.reason}</Typography>
                      <Typography variant="caption" color="text.secondary">
                        score: {group.score} contacts: {group.contacts.length}
                      </Typography>
                      <Typography variant="caption" color="text.secondary" className="truncate">
                        {group.contacts.map((contact) => contact.display_name).join(" • ")}
                      </Typography>
                      <Stack direction="row" spacing={1}>
                        <Button
                          size="small"
                          variant="outlined"
                          onClick={() => {
                            setSelectionIds(group.contacts.map((contact) => contact.id));
                            selectContact(group.contacts[0]?.id ?? null);
                          }}
                        >
                          Select Group
                        </Button>
                        <Button
                          size="small"
                          variant="contained"
                          onClick={() => {
                            setSelectionIds(group.contacts.map((contact) => contact.id));
                            selectContact(group.contacts[0]?.id ?? null);
                            void mergeSelected();
                          }}
                        >
                          Merge Group
                        </Button>
                      </Stack>
                    </Stack>
                  </Paper>
                ))}
              </Stack>
            ) : (
              <Typography variant="body2" color="text.secondary">
                No duplicate groups detected.
              </Typography>
            )}
          </Stack>
        </Paper>

        {error ? <Alert severity="error">{error}</Alert> : null}
        {loading ? <Alert severity="info">Working…</Alert> : null}
      </Stack>
    </div>
  );
}
