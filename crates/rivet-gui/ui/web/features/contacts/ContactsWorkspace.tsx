import { useEffect, useMemo, useRef, useState } from "react";
import type { ChangeEvent } from "react";

import { useVirtualizer } from "@tanstack/react-virtual";
import DeleteIcon from "@mui/icons-material/Delete";
import LinkIcon from "@mui/icons-material/Link";
import PersonAddIcon from "@mui/icons-material/PersonAdd";
import RedoIcon from "@mui/icons-material/Redo";
import UploadFileIcon from "@mui/icons-material/UploadFile";
import Alert from "@mui/material/Alert";
import Avatar from "@mui/material/Avatar";
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
import type { ContactDto, ContactFieldValue } from "../../types/core";

function fieldTemplate(kind: string): ContactFieldValue {
  return {
    value: "",
    kind,
    is_primary: false
  };
}

const COUNTRY_OPTIONS = [
  "United States",
  "Canada",
  "United Kingdom",
  "Germany",
  "France",
  "India",
  "Japan",
  "Australia",
  "Other"
];

function contactInitials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return "?";
  }
  if (parts.length === 1) {
    return parts[0].slice(0, 2).toUpperCase();
  }
  return `${parts[0][0] ?? ""}${parts[1][0] ?? ""}`.toUpperCase();
}

function normalizeToken(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .trim();
}

function possibleDuplicateText(draftName: string, draftEmails: string[], draftPhones: string[], contacts: ContactDto[], selectedId: string | null): string | null {
  const name = normalizeToken(draftName);
  const emails = new Set(draftEmails.map(normalizeToken).filter(Boolean));
  const phones = new Set(draftPhones.map((phone) => phone.replace(/[^0-9+]/g, "")).filter(Boolean));

  const match = contacts.find((contact) => {
    if (contact.id === selectedId) {
      return false;
    }
    const contactName = normalizeToken(contact.display_name);
    if (name && contactName && name === contactName) {
      return true;
    }
    const contactEmails = contact.emails.map((item) => normalizeToken(item.value));
    if (contactEmails.some((email) => emails.has(email))) {
      return true;
    }
    const contactPhones = contact.phones.map((item) => item.value.replace(/[^0-9+]/g, ""));
    return contactPhones.some((phone) => phones.has(phone));
  });

  if (!match) {
    return null;
  }

  return `Possible duplicate: ${match.display_name || "Unnamed Contact"}`;
}

function mergePreview(contacts: ContactDto[], preferredTargetId: string | null): ContactDto | null {
  if (contacts.length < 2) {
    return null;
  }

  const byRecency = [...contacts].sort((left, right) => right.updated_at.localeCompare(left.updated_at));
  const target = contacts.find((contact) => contact.id === preferredTargetId) ?? byRecency[0];

  const mergeFields = (current: ContactFieldValue[], incoming: ContactFieldValue[]): ContactFieldValue[] => {
    const seen = new Set(current.map((item) => `${item.kind}:${item.value}`.toLowerCase()));
    const next = [...current];
    for (const item of incoming) {
      const key = `${item.kind}:${item.value}`.toLowerCase();
      if (!seen.has(key) && item.value.trim()) {
        next.push({ ...item, is_primary: false });
        seen.add(key);
      }
    }
    const primaryIndex = next.findIndex((item) => item.is_primary);
    if (primaryIndex === -1 && next.length > 0) {
      next[0] = { ...next[0], is_primary: true };
    } else {
      for (let index = 0; index < next.length; index += 1) {
        next[index] = { ...next[index], is_primary: index === primaryIndex };
      }
    }
    return next;
  };

  const merged: ContactDto = {
    ...target,
    phones: [...target.phones],
    emails: [...target.emails],
    websites: [...target.websites]
  };

  for (const contact of contacts) {
    if (contact.id === target.id) {
      continue;
    }
    if (!merged.display_name.trim() && contact.display_name.trim()) {
      merged.display_name = contact.display_name;
    }
    merged.given_name = merged.given_name || contact.given_name;
    merged.family_name = merged.family_name || contact.family_name;
    merged.nickname = merged.nickname || contact.nickname;
    merged.notes = merged.notes || contact.notes;
    merged.organization = merged.organization || contact.organization;
    merged.title = merged.title || contact.title;
    merged.birthday = merged.birthday || contact.birthday;
    merged.avatar_data_url = merged.avatar_data_url || contact.avatar_data_url;
    merged.import_batch_id = merged.import_batch_id || contact.import_batch_id;
    merged.source_file_name = merged.source_file_name || contact.source_file_name;
    if (merged.addresses.length === 0 && contact.addresses.length > 0) {
      merged.addresses = contact.addresses;
    }
    merged.phones = mergeFields(merged.phones, contact.phones);
    merged.emails = mergeFields(merged.emails, contact.emails);
    merged.websites = mergeFields(merged.websites, contact.websites);
  }

  return merged;
}

export function ContactsWorkspace() {
  const {
    bootstrap,
    loading,
    error,
    query,
    sourceFilter,
    contacts,
    selectedContactId,
    selectionMode,
    selectionIds,
    total,
    nextCursor,
    dedupe,
    importPreview,
    importCommitResult,
    mergeUndoResult,
    formDraft,
    dirty,
    editorMode,
    setQuery,
    setSourceFilter,
    selectContact,
    toggleSelectionMode,
    toggleSelected,
    setSelectionIds,
    clearSelection,
    setDraft,
    resetDraft,
    beginAddNew,
    loadDraftFromSelected,
    createFromDraft,
    updateSelectedFromDraft,
    removeSelectedContact,
    removeBulkSelected,
    refreshDedupe,
    linkSelectedContacts,
    unlinkSelectedContacts,
    mergeSelected,
    undoLastMerge,
    decideDedupeGroup,
    previewImport,
    commitImport,
    openAction,
    loadMoreContacts
  } = useContactsStore();

  const rootRef = useRef<HTMLDivElement | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const avatarInputRef = useRef<HTMLInputElement | null>(null);
  const listContainerRef = useRef<HTMLDivElement | null>(null);

  const [searchInput, setSearchInput] = useState(query);
  const [importSource, setImportSource] = useState("gmail_export");
  const [importMode, setImportMode] = useState<"safe" | "upsert" | "review">("safe");
  const [selectedDedupeGroupId, setSelectedDedupeGroupId] = useState<string | null>(null);

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

  const selectedDedupeGroup = useMemo(
    () => dedupe?.groups.find((group) => group.group_id === selectedDedupeGroupId) ?? null,
    [dedupe, selectedDedupeGroupId]
  );

  const dedupePreviewContact = useMemo(() => {
    if (!selectedDedupeGroup) {
      return null;
    }
    return mergePreview(selectedDedupeGroup.contacts, selectedContactId);
  }, [selectedDedupeGroup, selectedContactId]);

  const duplicateWarning = useMemo(() => {
    return possibleDuplicateText(
      formDraft.display_name ?? "",
      formDraft.emails.map((item) => item.value),
      formDraft.phones.map((item) => item.value),
      contacts,
      selectedContactId
    );
  }, [contacts, formDraft.display_name, formDraft.emails, formDraft.phones, selectedContactId]);

  const sourceOptions = useMemo(() => {
    const out = new Map<string, number>();
    for (const contact of contacts) {
      const key = (contact.source_kind || contact.source_id || "local").trim() || "local";
      out.set(key, (out.get(key) ?? 0) + 1);
    }
    return [...out.entries()].sort(([left], [right]) => left.localeCompare(right));
  }, [contacts]);

  const rowVirtualizer = useVirtualizer({
    count: contacts.length,
    getScrollElement: () => listContainerRef.current,
    estimateSize: () => 74,
    overscan: 8
  });

  const updateDraftField = (key: keyof typeof formDraft, value: unknown) => {
    setDraft({
      ...formDraft,
      [key]: value
    });
  };

  const updateDraftFieldArray = (key: "phones" | "emails" | "websites", next: ContactFieldValue[]) => {
    setDraft({
      ...formDraft,
      [key]: next
    });
  };

  const updateDraftAddress = (field: "street" | "country", value: string) => {
    const first = formDraft.addresses[0] ?? {
      kind: "home",
      street: "",
      city: "",
      region: "",
      postal_code: "",
      country: ""
    };
    setDraft({
      ...formDraft,
      addresses: [{
        ...first,
        [field]: value
      }]
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

  const handleAvatarFile = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = typeof reader.result === "string" ? reader.result : null;
      updateDraftField("avatar_data_url", dataUrl);
    };
    reader.readAsDataURL(file);
    event.target.value = "";
  };

  const importErrorSummary = useMemo(() => {
    const lines = [
      ...(importPreview?.errors ?? []),
      ...(importCommitResult?.errors ?? [])
    ].filter((line) => line.trim().length > 0);
    if (lines.length === 0) {
      return null;
    }
    return lines.join("\n");
  }, [importCommitResult?.errors, importPreview?.errors]);

  const draftValidationError = useMemo(() => {
    const hasName = (formDraft.display_name ?? "").trim().length > 0;
    const hasEmail = formDraft.emails.some((item) => item.value.trim().length > 0);
    const hasPhone = formDraft.phones.some((item) => item.value.trim().length > 0);
    if (hasName || hasEmail || hasPhone) {
      return null;
    }
    return "At least one of display name, email, or phone is required.";
  }, [formDraft.display_name, formDraft.emails, formDraft.phones]);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      const root = rootRef.current;
      if (!root || root.offsetParent === null) {
        return;
      }

      const target = event.target as HTMLElement | null;
      const isEditable = Boolean(
        target
          && (target.tagName === "INPUT"
            || target.tagName === "TEXTAREA"
            || target.tagName === "SELECT"
            || target.isContentEditable)
      );
      const isMeta = event.metaKey || event.ctrlKey;
      const key = event.key.toLowerCase();

      if (isMeta && key === "n") {
        event.preventDefault();
        beginAddNew();
        return;
      }

      if (isEditable) {
        return;
      }

      const selectedIndex = contacts.findIndex((contact) => contact.id === selectedContactId);

      if (key === "arrowdown") {
        event.preventDefault();
        const nextIndex = selectedIndex < 0 ? 0 : Math.min(contacts.length - 1, selectedIndex + 1);
        const next = contacts[nextIndex];
        if (next) {
          selectContact(next.id);
          rowVirtualizer.scrollToIndex(nextIndex, { align: "auto" });
        }
        return;
      }

      if (key === "arrowup") {
        event.preventDefault();
        const nextIndex = selectedIndex < 0 ? 0 : Math.max(0, selectedIndex - 1);
        const next = contacts[nextIndex];
        if (next) {
          selectContact(next.id);
          rowVirtualizer.scrollToIndex(nextIndex, { align: "auto" });
        }
        return;
      }

      if (key === "enter" && selectedContactId) {
        event.preventDefault();
        loadDraftFromSelected();
        return;
      }

      if ((key === "delete" || key === "backspace") && selectionMode && selectionIds.length > 0) {
        event.preventDefault();
        void removeBulkSelected();
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [
    beginAddNew,
    contacts,
    loadDraftFromSelected,
    removeBulkSelected,
    rowVirtualizer,
    selectContact,
    selectedContactId,
    selectionIds.length,
    selectionMode
  ]);

  return (
    <div ref={rootRef} className="grid h-full min-h-0 grid-cols-[360px_minmax(0,1fr)] gap-3 p-3">
      <Paper className="min-h-0 p-3">
        <Stack spacing={1.25} className="h-full min-h-0">
          <Typography variant="h6">Contacts</Typography>

          <TextField
            label="Search contacts"
            value={searchInput}
            onChange={(event) => setSearchInput(event.target.value)}
            size="small"
          />

          <TextField
            select
            size="small"
            label="Source"
            value={sourceFilter ?? ""}
            onChange={(event) => {
              const value = event.target.value.trim();
              void setSourceFilter(value.length > 0 ? value : null);
            }}
          >
            <MenuItem value="">All Sources</MenuItem>
            {sourceOptions.map(([value, count]) => (
              <MenuItem key={value} value={value}>
                {value} ({count})
              </MenuItem>
            ))}
          </TextField>

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
                void linkSelectedContacts();
              }}
            >
              Link Selected
            </Button>
            <Button
              size="small"
              variant="outlined"
              disabled={selectionIds.length === 0 && !selectedContactId}
              onClick={() => {
                void unlinkSelectedContacts();
              }}
            >
              Unlink
            </Button>
            <Button
              size="small"
              variant="outlined"
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
            <Stack direction="row" spacing={1}>
              <Button
                size="small"
                variant="outlined"
                startIcon={<UploadFileIcon fontSize="small" />}
                onClick={() => fileInputRef.current?.click()}
              >
                Import Contacts
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
            <TextField
              select
              size="small"
              label="Source Preset"
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
                preview rows: {importPreview.total_rows} valid: {importPreview.valid_rows} duplicates: {importPreview.potential_duplicates}
              </Typography>
            ) : null}
            {importCommitResult ? (
              <Typography variant="caption" color="text.secondary">
                import result: +{importCommitResult.created} ~{importCommitResult.updated} skip: {importCommitResult.skipped} fail: {importCommitResult.failed}
              </Typography>
            ) : null}
            {importErrorSummary ? (
              <Button
                size="small"
                variant="outlined"
                onClick={() => {
                  void navigator.clipboard.writeText(importErrorSummary);
                }}
              >
                Copy Import Errors
              </Button>
            ) : null}
            {mergeUndoResult ? (
              <Typography variant="caption" color="text.secondary">
                merge undo restored: {mergeUndoResult.restored}
              </Typography>
            ) : null}
          </Stack>

          <Divider />

          <Stack direction="row" alignItems="center" justifyContent="space-between">
            <Typography variant="caption" color="text.secondary">
              contacts: {contacts.length}/{total}
            </Typography>
            <Button
              size="small"
              variant="outlined"
              disabled={!nextCursor}
              onClick={() => {
                void loadMoreContacts();
              }}
            >
              Load More
            </Button>
          </Stack>

          <div ref={listContainerRef} className="min-h-0 flex-1 overflow-y-auto pr-1">
            {contacts.length === 0 ? (
              <Typography variant="body2" color="text.secondary">
                {query.trim().length > 0 ? "No contacts match your search." : "No contacts yet. Add one from the form on the right."}
              </Typography>
            ) : (
              <div
                style={{
                  height: `${rowVirtualizer.getTotalSize()}px`,
                  position: "relative"
                }}
              >
                {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                  const contact = contacts[virtualRow.index];
                  return (
                    <div
                      key={contact.id}
                      style={{
                        position: "absolute",
                        top: 0,
                        left: 0,
                        width: "100%",
                        transform: `translateY(${virtualRow.start}px)`
                      }}
                    >
                      <Paper
                        variant={selectedContactId === contact.id ? "elevation" : "outlined"}
                        className="mb-1 cursor-pointer p-2"
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
                          <Avatar
                            src={contact.avatar_data_url ?? undefined}
                            sx={{ width: 32, height: 32, fontSize: 13 }}
                          >
                            {contactInitials(contact.display_name || "Unnamed Contact")}
                          </Avatar>
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
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </Stack>
      </Paper>

      <Stack spacing={2} className="min-h-0">
        <Paper className="p-4">
          <Stack spacing={1.25}>
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Typography variant="h6">{editorMode === "edit" && selectedContact ? "Contact Details / Edit" : "Add Contact"}</Typography>
              <Button size="small" variant="outlined" onClick={beginAddNew}>
                Add New
              </Button>
            </Stack>

            <Stack direction="row" spacing={1.5} alignItems="center">
              <Avatar
                src={formDraft.avatar_data_url ?? undefined}
                sx={{ width: 56, height: 56 }}
              >
                {contactInitials(formDraft.display_name ?? "")}
              </Avatar>
              <Stack direction="row" spacing={1}>
                <Button size="small" variant="outlined" onClick={() => avatarInputRef.current?.click()}>
                  Set Avatar
                </Button>
                <Button
                  size="small"
                  variant="outlined"
                  onClick={() => updateDraftField("avatar_data_url", null)}
                  disabled={!formDraft.avatar_data_url}
                >
                  Remove Avatar
                </Button>
              </Stack>
              <input
                ref={avatarInputRef}
                type="file"
                accept="image/*"
                className="hidden"
                onChange={handleAvatarFile}
              />
            </Stack>

            {dirty ? <Alert severity="info">Unsaved changes</Alert> : null}
            {duplicateWarning ? <Alert severity="warning">{duplicateWarning}</Alert> : null}
            {draftValidationError ? <Alert severity="warning">{draftValidationError}</Alert> : null}

            <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
              <Button
                size="small"
                variant="outlined"
                onClick={loadDraftFromSelected}
                disabled={!selectedContact}
              >
                Reload Selected
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
                  if (!draftValidationError) {
                    void createFromDraft();
                  }
                }}
              >
                Create Contact
              </Button>
              <Button
                size="small"
                variant="outlined"
                disabled={!selectedContact}
                onClick={() => {
                  if (!draftValidationError) {
                    void updateSelectedFromDraft();
                  }
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
              value={formDraft.display_name ?? ""}
              onChange={(event) => updateDraftField("display_name", event.target.value)}
              size="small"
            />

            <Stack direction={{ xs: "column", md: "row" }} spacing={1}>
              <TextField
                label="Given Name"
                value={formDraft.given_name ?? ""}
                onChange={(event) => updateDraftField("given_name", event.target.value)}
                size="small"
                className="flex-1"
              />
              <TextField
                label="Family Name"
                value={formDraft.family_name ?? ""}
                onChange={(event) => updateDraftField("family_name", event.target.value)}
                size="small"
                className="flex-1"
              />
              <TextField
                label="Nickname"
                value={formDraft.nickname ?? ""}
                onChange={(event) => updateDraftField("nickname", event.target.value)}
                size="small"
                className="flex-1"
              />
            </Stack>

            <Stack direction={{ xs: "column", md: "row" }} spacing={1}>
              <TextField
                label="Organization"
                value={formDraft.organization ?? ""}
                onChange={(event) => updateDraftField("organization", event.target.value)}
                size="small"
                className="flex-1"
              />
              <TextField
                label="Title"
                value={formDraft.title ?? ""}
                onChange={(event) => updateDraftField("title", event.target.value)}
                size="small"
                className="flex-1"
              />
              <TextField
                label="Birthday"
                type="date"
                value={formDraft.birthday ?? ""}
                onChange={(event) => updateDraftField("birthday", event.target.value || null)}
                size="small"
                className="flex-1"
                InputLabelProps={{ shrink: true }}
              />
            </Stack>

            <TextField
              label="Notes"
              value={formDraft.notes ?? ""}
              onChange={(event) => updateDraftField("notes", event.target.value)}
              size="small"
              multiline
              minRows={3}
            />

            <Stack direction={{ xs: "column", md: "row" }} spacing={1}>
              <TextField
                label="Address"
                value={formDraft.addresses[0]?.street ?? ""}
                onChange={(event) => updateDraftAddress("street", event.target.value)}
                size="small"
                className="flex-1"
              />
              <TextField
                select
                label="Country"
                value={formDraft.addresses[0]?.country ?? ""}
                onChange={(event) => updateDraftAddress("country", event.target.value)}
                size="small"
                className="flex-1"
              >
                {COUNTRY_OPTIONS.map((country) => (
                  <MenuItem key={country} value={country}>
                    {country}
                  </MenuItem>
                ))}
              </TextField>
            </Stack>

            <Divider />

            <Stack spacing={0.8}>
              <Typography variant="subtitle2">Emails</Typography>
              {formDraft.emails.map((email, index) => (
                <Stack key={`email-${index}`} direction="row" spacing={1} alignItems="center">
                  <TextField
                    label="Email"
                    size="small"
                    value={email.value}
                    onChange={(event) => {
                      const next = [...formDraft.emails];
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
                      const next = [...formDraft.emails];
                      next[index] = { ...next[index], kind: event.target.value };
                      updateDraftFieldArray("emails", next);
                    }}
                    className="w-32"
                  />
                  <Checkbox
                    checked={email.is_primary}
                    onChange={(event) => {
                      const next = formDraft.emails.map((entry, i) => ({ ...entry, is_primary: i === index ? event.target.checked : false }));
                      updateDraftFieldArray("emails", next);
                    }}
                  />
                  <IconButton
                    size="small"
                    onClick={() => {
                      const next = formDraft.emails.filter((_, i) => i !== index);
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
                  updateDraftFieldArray("emails", [...formDraft.emails, fieldTemplate("home")]);
                }}
              >
                Add Email
              </Button>
            </Stack>

            <Stack spacing={0.8}>
              <Typography variant="subtitle2">Phones</Typography>
              {formDraft.phones.map((phone, index) => (
                <Stack key={`phone-${index}`} direction="row" spacing={1} alignItems="center">
                  <TextField
                    label="Phone"
                    size="small"
                    value={phone.value}
                    onChange={(event) => {
                      const next = [...formDraft.phones];
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
                      const next = [...formDraft.phones];
                      next[index] = { ...next[index], kind: event.target.value };
                      updateDraftFieldArray("phones", next);
                    }}
                    className="w-32"
                  />
                  <Checkbox
                    checked={phone.is_primary}
                    onChange={(event) => {
                      const next = formDraft.phones.map((entry, i) => ({ ...entry, is_primary: i === index ? event.target.checked : false }));
                      updateDraftFieldArray("phones", next);
                    }}
                  />
                  <IconButton
                    size="small"
                    onClick={() => {
                      const next = formDraft.phones.filter((_, i) => i !== index);
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
                  updateDraftFieldArray("phones", [...formDraft.phones, fieldTemplate("mobile")]);
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
                      <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
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
                          variant="outlined"
                          onClick={() => {
                            setSelectedDedupeGroupId(group.group_id);
                            setSelectionIds(group.contacts.map((contact) => contact.id));
                          }}
                        >
                          Preview Merge
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
                        <Button
                          size="small"
                          variant="outlined"
                          color="warning"
                          onClick={() => {
                            void decideDedupeGroup(group.group_id, "separate");
                          }}
                        >
                          Keep Separate
                        </Button>
                        <Button
                          size="small"
                          variant="outlined"
                          color="secondary"
                          onClick={() => {
                            void decideDedupeGroup(group.group_id, "ignored");
                          }}
                        >
                          Ignore
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

            {selectedDedupeGroup ? (
              <Paper variant="outlined" className="p-3">
                <Stack spacing={1}>
                  <Typography variant="subtitle2">Duplicate Group Details</Typography>
                  <Stack direction={{ xs: "column", md: "row" }} spacing={1}>
                    {selectedDedupeGroup.contacts.slice(0, 2).map((contact) => (
                      <Paper key={contact.id} variant="outlined" className="flex-1 p-2">
                        <Stack spacing={0.5}>
                          <Typography variant="subtitle2">{contact.display_name || "Unnamed Contact"}</Typography>
                          <Typography variant="caption" color="text.secondary">email: {contact.emails[0]?.value ?? "-"}</Typography>
                          <Typography variant="caption" color="text.secondary">phone: {contact.phones[0]?.value ?? "-"}</Typography>
                          <Typography variant="caption" color="text.secondary">org: {contact.organization ?? "-"}</Typography>
                          <Button
                            size="small"
                            variant={selectedContactId === contact.id ? "contained" : "outlined"}
                            onClick={() => {
                              setSelectionIds(selectedDedupeGroup.contacts.map((item) => item.id));
                              selectContact(contact.id);
                            }}
                          >
                            Use As Merge Target
                          </Button>
                        </Stack>
                      </Paper>
                    ))}
                  </Stack>
                  {dedupePreviewContact ? (
                    <Alert severity="info">
                      Merge preview: {dedupePreviewContact.display_name || "Unnamed Contact"} · emails {dedupePreviewContact.emails.length} · phones {dedupePreviewContact.phones.length}
                    </Alert>
                  ) : null}
                </Stack>
              </Paper>
            ) : null}
          </Stack>
        </Paper>

        {error ? <Alert severity="error">{error}</Alert> : null}
        {loading ? <Alert severity="info">Working…</Alert> : null}
      </Stack>
    </div>
  );
}
