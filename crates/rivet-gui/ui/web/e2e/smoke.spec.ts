import { expect, test } from "@playwright/test";

const nodeEnv = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env ?? {};
const nodeBufferFactory = (globalThis as unknown as {
  Buffer?: {
    from: (value: string, encoding?: string) => Uint8Array;
  };
}).Buffer;
const e2eTheme = String(nodeEnv.RIVET_E2E_THEME ?? "day").trim().toLowerCase();
const e2eTaskCount = Number.parseInt(String(nodeEnv.RIVET_E2E_TASK_COUNT ?? "0"), 10);

test.beforeEach(async ({ page }) => {
  await page.addInitScript(({ theme, taskCount }) => {
    if (window.sessionStorage.getItem("rivet.e2e.seeded") === "1") {
      return;
    }

    window.localStorage.clear();
    if (theme === "night" || theme === "dark") {
      window.localStorage.setItem("rivet.theme", "night");
    }

    if (taskCount > 0) {
      const now = Date.now();
      const tasks = Array.from({ length: taskCount }).map((_, index) => {
        const due = new Date(now + (index % 180) * 86_400_000).toISOString();
        return {
          uuid: `seed-task-${index + 1}`,
          id: null,
          title: `Seed Task ${index + 1}`,
          description: `Seeded dataset item ${index + 1}`,
          status: index % 7 === 0 ? "Waiting" : "Pending",
          project: index % 4 === 0 ? "seed-alpha" : "seed-beta",
          tags: [`kanban:${index % 3 === 0 ? "todo" : (index % 3 === 1 ? "working" : "finished")}`],
          priority: index % 5 === 0 ? "High" : null,
          due,
          wait: null,
          scheduled: null,
          created: due,
          modified: due
        };
      });
      window.localStorage.setItem("rivet.mock.tasks", JSON.stringify(tasks));
    }
    window.sessionStorage.setItem("rivet.e2e.seeded", "1");
  }, { theme: e2eTheme, taskCount: Number.isFinite(e2eTaskCount) ? Math.max(0, e2eTaskCount) : 0 });
  await page.goto("/", { waitUntil: "domcontentloaded", timeout: 120_000 });
  await expect(page.getByText("Rivet")).toBeVisible();
});

test("tasks smoke: add, complete, delete", async ({ page }) => {
  await page.getByRole("button", { name: "Add Task" }).click();
  await page.getByLabel("Title").fill("Smoke Task");
  await page.getByRole("button", { name: "Save" }).click();

  const taskRow = page.getByRole("button", { name: /Smoke Task/i });
  await expect(taskRow).toBeVisible();
  await taskRow.click();

  await page.getByRole("button", { name: "Done" }).click();
  await expect(page.getByText("Completed").first()).toBeVisible();

  await page.getByRole("button", { name: /^Delete$/ }).click();
  await expect(page.getByText("Smoke Task")).toHaveCount(0);
});

test("kanban smoke: create board, add card, move lane", async ({ page }) => {
  await page.getByRole("tab", { name: "Kanban" }).click();
  await page.getByRole("button", { name: "New" }).click();
  await page.getByLabel("Board Name").fill("Smoke Board");
  await page.getByRole("button", { name: "Create" }).click();

  await page.getByRole("button", { name: "Add Task To Board" }).click();
  await page.getByLabel("Title").fill("Kanban Smoke Task");
  await page.getByRole("button", { name: "Save" }).click();

  await expect(
    page
      .getByTestId(/kanban-card-.+/)
      .filter({ hasText: "Kanban Smoke Task" })
      .first()
  ).toBeVisible();

  const laneCard = page.getByTestId(/kanban-card-.+/).first();
  const targetLane = page.getByTestId("kanban-lane-finished");
  await laneCard.dragTo(targetLane);
  await expect(targetLane.getByText("Kanban Smoke Task")).toBeVisible();
});

test("calendar smoke: add external calendar source", async ({ page }) => {
  await page.getByRole("tab", { name: "Calendar" }).click();
  await page.getByRole("button", { name: "Year" }).click();
  await page.getByRole("button", { name: "January" }).click();
  await expect(page.locator(".calendar-month-grid")).toBeVisible();

  await page.getByRole("button", { name: "Add Source" }).click();
  await expect(page.getByRole("heading", { name: "Add External Calendar" })).toBeVisible();

  await page.getByLabel("Calendar Name").fill("Smoke Source");
  await page.getByLabel("Location (ICS or webcal URL)").fill("webcal://example.com/smoke.ics");
  await page.getByRole("button", { name: "Save" }).click();

  await expect(page.getByText("Smoke Source")).toBeVisible();
  await page.getByLabel("Edit Smoke Source").click();
  await page.getByLabel("Refresh Calendar").click();
  await page.getByRole("option", { name: "Every 15 minutes" }).click();
  await page.getByRole("button", { name: "Save" }).click();
  await expect(page.getByText("refresh:15m")).toBeVisible();
});

test("theme smoke: toggle persists across reload", async ({ page }) => {
  const startsNight = e2eTheme === "night" || e2eTheme === "dark";
  const clickLabel = startsNight ? "Day" : "Night";
  const expectLabel = startsNight ? "Night" : "Day";

  await page.getByRole("button", { name: clickLabel }).click();
  await expect(page.getByRole("button", { name: expectLabel })).toBeVisible();
  await page.reload();
  await expect(page.getByRole("button", { name: expectLabel })).toBeVisible();
});

test("tasks parity: edit flow and bulk filtered actions", async ({ page }) => {
  for (const title of ["Bulk A", "Bulk B"]) {
    await page.getByRole("button", { name: "Add Task" }).click();
    await page.getByLabel("Title").fill(title);
    await page.getByRole("button", { name: "Save" }).click();
    await expect(page.getByRole("button", { name: new RegExp(title, "i") })).toBeVisible();
  }

  const taskRow = page.getByRole("button", { name: /Bulk A/i });
  await taskRow.click();
  await page.getByRole("button", { name: "Edit" }).click();
  await page.getByLabel("Title").fill("Bulk A Edited");
  await page.getByRole("button", { name: "Save" }).click();
  await expect(page.getByRole("button", { name: /Bulk A Edited/i })).toBeVisible();

  await page.getByRole("button", { name: /^Complete Filtered/ }).first().click();
  await expect(page.getByText("Completed").first()).toBeVisible();

  await page.getByLabel("Search").fill("Bulk");
  page.once("dialog", (dialog) => dialog.accept());
  await page.getByRole("button", { name: /Delete Filtered/i }).click();
  await expect(page.getByText("Bulk A Edited")).toHaveCount(0);
  await expect(page.getByText("Bulk B")).toHaveCount(0);
});

test("keyboard shortcuts: add task and settings", async ({ page }) => {
  await page.keyboard.press("Control+N");
  await expect(page.getByRole("heading", { name: "Add Task" })).toBeVisible();
  await page.getByRole("button", { name: "Cancel" }).click();

  await page.keyboard.press("Control+,");
  await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
  await page.getByLabel("Enable OS due notifications").click();
  await page.getByRole("button", { name: "Close" }).click();
  await expect(page.getByRole("heading", { name: "Settings" })).toHaveCount(0);
});

test("contacts smoke: create, search, update, and bulk delete", async ({ page }) => {
  await page.getByRole("tab", { name: "Contacts" }).click();
  await expect(page.getByRole("heading", { name: "Contacts" })).toBeVisible();

  await page.getByLabel("Display Name").fill("Smoke Contact");
  await page.getByLabel("Email").first().fill("smoke.contact@example.com");
  await page.getByRole("button", { name: "Create Contact" }).click();

  await expect(page.getByText("Smoke Contact")).toBeVisible();
  await page.getByLabel("Search contacts").fill("Smoke Contact");
  await page.getByText("Smoke Contact").first().click();
  await page.getByLabel("Notes").fill("Updated note");
  await page.getByRole("button", { name: "Update Selected" }).click();

  await page.getByRole("button", { name: /^Select$/ }).click();
  await page.getByText("Smoke Contact").first().click();
  await page.getByRole("button", { name: /^Delete Selected$/ }).first().click();
  await expect(page.getByText("Smoke Contact")).toHaveCount(0);
});

test("app startup regression: diagnostics and tab switching remain responsive", async ({ page }) => {
  await expect(page.getByText("mode: dev")).toBeVisible();
  await expect(page.getByRole("button", { name: "Diagnostics" })).toBeVisible();

  await page.getByRole("tab", { name: "Tasks" }).click();
  await expect(page.getByRole("button", { name: "Add Task" })).toBeVisible();

  await page.getByRole("tab", { name: "Kanban" }).click();
  await expect(page.getByRole("button", { name: "New" })).toBeVisible();

  await page.getByRole("tab", { name: "Calendar" }).click();
  await expect(page.getByRole("button", { name: "Year" })).toBeVisible();

  await page.getByRole("tab", { name: "Contacts" }).click();
  await expect(page.getByRole("heading", { name: "Contacts" })).toBeVisible();

  await page.getByRole("button", { name: "Diagnostics" }).click();
  await expect(page.getByText("Diagnostics (last invoke failures)")).toBeVisible();
  await page.getByTitle("Close diagnostics").click();
  await expect(page.getByText("Diagnostics (last invoke failures)")).toHaveCount(0);
});

test("contacts e2e: import preview conflicts then merge and verify list/search", async ({ page }) => {
  await page.getByRole("tab", { name: "Contacts" }).click();

  await page.getByLabel("Display Name").fill("Mergeable Contact");
  await page.getByLabel("Email").first().fill("merge.a@example.com");
  await page.getByLabel("Phone").first().fill("+1-555-1000");
  await page.getByRole("button", { name: "Create Contact" }).click();
  await expect(page.getByText("Mergeable Contact")).toBeVisible();

  await page.getByLabel("Display Name").fill("Mergeable Contact");
  await page.getByLabel("Email").first().fill("merge.b@example.com");
  await page.getByLabel("Phone").first().fill("+1-555-1001");
  await page.getByRole("button", { name: "Create Contact" }).click();

  const vcard = [
    "BEGIN:VCARD",
    "VERSION:3.0",
    "FN:Mergeable Contact",
    "EMAIL;TYPE=HOME,PREF:merge.a@example.com",
    "TEL;TYPE=CELL:+1-555-1000",
    "END:VCARD",
    ""
  ].join("\n");
  const vcardBuffer = nodeBufferFactory?.from(vcard, "utf-8");
  if (!vcardBuffer) {
    throw new Error("node Buffer is unavailable for e2e file upload");
  }
  await page.locator('input[type="file"][accept*=".vcf"]').setInputFiles({
    name: "gmail-contacts.vcf",
    mimeType: "text/vcard",
    buffer: vcardBuffer as never
  });

  await expect(page.getByText(/preview rows:/i)).toBeVisible();
  await expect(page.getByText(/duplicates:\s*\d+/i)).toBeVisible();

  await page.getByLabel("Import Mode").click();
  await page.getByRole("option", { name: /Review \(preview first\)/i }).click();
  await page.getByRole("button", { name: "Commit Import" }).click();
  await expect(page.getByText(/import result:/i)).toBeVisible();

  const mergeButtons = page.getByRole("button", { name: /^Merge Group$/ });
  const mergeCountBefore = await mergeButtons.count();
  expect(mergeCountBefore).toBeGreaterThan(0);
  await mergeButtons.first().click();
  await expect.poll(async () => mergeButtons.count()).toBeLessThan(mergeCountBefore);

  await page.getByLabel("Search contacts").fill("Mergeable Contact");
  await page.getByText("Mergeable Contact").first().click();
  await expect(page.getByLabel("Display Name")).toHaveValue("Mergeable Contact");
});
