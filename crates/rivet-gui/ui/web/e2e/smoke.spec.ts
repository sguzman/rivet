import { expect, test } from "@playwright/test";

const nodeEnv = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env ?? {};
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

  await expect(page.getByText("Kanban Smoke Task")).toBeVisible();

  const laneCard = page.getByTestId(/kanban-card-.+/).first();
  const targetLane = page.getByTestId("kanban-lane-finished");
  await laneCard.dragTo(targetLane);
  await expect(page.getByText("kanban:finished").first()).toBeVisible();
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
