import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.clear();
  });
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

  await page.getByRole("button", { name: "Delete" }).click();
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
  await page.getByRole("button", { name: "Working" }).first().click();
  await expect(page.getByText("kanban:working").first()).toBeVisible();

  const laneCard = page.getByTestId(/kanban-card-.+/).first();
  const targetLane = page.getByTestId("kanban-lane-finished");
  await laneCard.dragTo(targetLane);
  await expect(page.getByText("kanban:finished").first()).toBeVisible();
});

test("calendar smoke: add external calendar source", async ({ page }) => {
  await page.getByRole("tab", { name: "Calendar" }).click();
  await page.getByRole("button", { name: "Year" }).click();
  await page.getByRole("button", { name: "January" }).click();
  await expect(page.getByRole("button", { name: "Week of" }).first()).toBeVisible();

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
  await page.getByRole("button", { name: "Night" }).click();
  await expect(page.getByRole("button", { name: "Day" })).toBeVisible();
  await page.reload();
  await expect(page.getByRole("button", { name: "Day" })).toBeVisible();
});
