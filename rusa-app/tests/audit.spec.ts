import { test, expect, type Page } from '@playwright/test';

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/**
 * Assert that no standard error states are visible and that any loading
 * spinner has already disappeared.  Call this after every action that
 * triggers a network request.
 *
 * TODO: add `data-testid="loading-spinner"` to your global spinner component
 *       so Playwright can target it precisely.
 */
async function assertNoErrors(page: Page) {
  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
}

/**
 * Log in to RUSA IMS with the given credentials.
 *
 * Adjust the locators below if your login form uses different labels or IDs.
 * The current implementation targets the semantic <label> elements rendered by
 * the login page (`src/routes/+page.svelte`).
 */
async function login(page: Page, username: string, password: string) {
  await page.goto('/');

  // TODO: adjust if your Username / Password labels differ
  await page.getByLabel('Username').fill(username);
  await page.getByLabel('Password').fill(password);

  // TODO: adjust if the login button text changes (currently "ACCESS SYSTEM")
  await page.getByRole('button', { name: /access system/i }).click();

  // Wait for navigation away from the login page
  await page.waitForURL('**/dashboard', { timeout: 15_000 });

  await assertNoErrors(page);
}

/**
 * Navigate to a top-level module via its sidebar / nav link.
 *
 * TODO: adjust the link name to match the exact text in your nav component
 *       (`src/routes/+layout.svelte`).
 */
async function navigateTo(page: Page, moduleName: string) {
  // TODO: update the role / name if the nav uses a different element type
  await page.getByRole('link', { name: new RegExp(moduleName, 'i') }).click();
  await assertNoErrors(page);
}

// ---------------------------------------------------------------------------
// 1. Sanitary Department — head_of_sanitary
// ---------------------------------------------------------------------------
test('Sanitary Department: assign task, view inventory, update shift', async ({ page }) => {
  await login(page, 'head_of_sanitary', 'password');

  // Navigate to the Sanitary module
  // TODO: match the exact nav-link label used in your sidebar
  await navigateTo(page, 'Sanitary');
  await expect(page).toHaveURL(/sanitary/);

  // ── CREATE: Assign a new task to the clean-up crew ──────────────────────
  // TODO: adjust to match the actual button / heading text in sanitary/+page.svelte
  await page.getByRole('button', { name: /assign task/i }).click();

  // Fill in task details
  // TODO: update field labels to match your form components
  await page.getByLabel(/task name/i).fill('Clean Sector 7 waste bins');
  await page.getByLabel(/assigned to/i).fill('Clean-Up Crew');

  // TODO: adjust submit button name if needed
  await page.getByRole('button', { name: /submit|save|confirm/i }).click();

  await assertNoErrors(page);
  // TODO: adjust the success text to what your toast/notification shows
  await expect(page.getByText(/task.*assigned|assigned.*task|success/i)).toBeVisible();

  // ── READ: View the sanitary inventory list ──────────────────────────────
  // TODO: adjust tab/link name to the actual inventory section label
  await page.getByRole('tab', { name: /inventory/i }).click();

  await assertNoErrors(page);
  // Assert the inventory table/list has rendered at least one row
  // TODO: adjust the selector to match your inventory table component
  await expect(page.locator('table tbody tr, [data-testid="inventory-item"]').first()).toBeVisible();

  // ── UPDATE: Modify a staff member's shift / schedule ────────────────────
  // TODO: adjust tab/link name to the actual shift section label
  await page.getByRole('tab', { name: /shift|schedule/i }).click();

  await assertNoErrors(page);

  // Open the edit flow for the first shift entry
  // TODO: adjust to the actual edit button / icon in your shift component
  await page.locator('[data-testid="edit-shift"], button:has-text("Edit")').first().click();

  // Change the shift time
  // TODO: update field labels to match your shift form
  await page.getByLabel(/start time|shift start/i).fill('08:00');

  await page.getByRole('button', { name: /update|save|confirm/i }).click();

  await assertNoErrors(page);
  // TODO: adjust to your toast/notification success message
  await expect(page.getByText(/updated|saved|success/i)).toBeVisible();
});

// ---------------------------------------------------------------------------
// 2. Medical Department — head_of_medicine
// ---------------------------------------------------------------------------
test('Medical Department: submit budget request, open patient log, update inventory', async ({ page }) => {
  await login(page, 'head_of_medicine', 'password');

  // Navigate to the Medical module
  // TODO: match the exact nav-link label used in your sidebar
  await navigateTo(page, 'Medical');
  await expect(page).toHaveURL(/medical/);

  // ── CREATE: Submit a budget request ─────────────────────────────────────
  // TODO: adjust button label to match medical/+page.svelte
  await page.getByRole('button', { name: /budget request|submit budget/i }).click();

  // Fill in budget request form
  // TODO: update field labels to match your form components
  await page.getByLabel(/amount|budget amount/i).fill('5000');
  await page.getByLabel(/reason|description/i).fill('Medical supplies restock for Q3');

  await page.getByRole('button', { name: /submit|send|confirm/i }).click();

  await assertNoErrors(page);
  // TODO: adjust to your success notification text
  await expect(page.getByText(/request submitted|budget.*submitted|success/i)).toBeVisible();

  // ── READ: Open a patient log ─────────────────────────────────────────────
  // TODO: adjust tab/section label for the patient log view
  await page.getByRole('tab', { name: /patient log|patients/i }).click();

  await assertNoErrors(page);

  // Open the first patient record
  // TODO: adjust to the actual row/card click target in your patient list
  await page.locator('[data-testid="patient-row"], tr[role="row"]').first().click();

  await assertNoErrors(page);
  // Assert the patient detail panel / modal appeared
  // TODO: adjust heading text to match your patient detail component
  await expect(page.getByRole('heading', { name: /patient detail|patient log|record/i })).toBeVisible();

  // ── UPDATE: Update medical inventory ────────────────────────────────────
  // Close modal if present
  // TODO: adjust close button selector if needed
  const closeBtn = page.getByRole('button', { name: /close|dismiss/i });
  if (await closeBtn.isVisible()) await closeBtn.click();

  // TODO: adjust tab/section label for the inventory view
  await page.getByRole('tab', { name: /inventory/i }).click();

  await assertNoErrors(page);

  // Edit the first inventory item
  // TODO: adjust edit button selector
  await page.locator('[data-testid="edit-inventory"], button:has-text("Edit")').first().click();

  // TODO: update field labels to match your inventory edit form
  await page.getByLabel(/quantity|stock/i).fill('100');

  await page.getByRole('button', { name: /update|save|confirm/i }).click();

  await assertNoErrors(page);
  // TODO: adjust to your success notification text
  await expect(page.getByText(/updated|saved|success/i)).toBeVisible();
});

// ---------------------------------------------------------------------------
// 3. Security Teams — head_of_earth_security
// ---------------------------------------------------------------------------
test('Security Teams: submit incident report and request broadcast', async ({ page }) => {
  await login(page, 'head_of_earth_security', 'password');

  // Navigate to the Security module
  // TODO: match the exact nav-link label used in your sidebar
  await navigateTo(page, 'Security');
  await expect(page).toHaveURL(/security/);

  // ── CREATE: Submit an incident report ────────────────────────────────────
  // TODO: adjust button label to match security/+page.svelte
  await page.getByRole('button', { name: /incident report|report incident/i }).click();

  // Fill in incident details
  // TODO: update field labels to match your incident form components
  await page.getByLabel(/title|incident title/i).fill('Perimeter breach — North Gate');
  await page.getByLabel(/description|details/i).fill('Unauthorised entry detected at north perimeter gate. Crew dispatched.');

  // TODO: replace 'High' with the exact option label used in your severity select
  await page.getByLabel(/severity|priority/i).selectOption({ label: 'High' });

  await page.getByRole('button', { name: /submit|file report|confirm/i }).click();

  await assertNoErrors(page);
  // TODO: adjust to your success notification text
  await expect(page.getByText(/report.*submitted|incident.*filed|success/i)).toBeVisible();

  // ── CREATE: Request a broadcast ──────────────────────────────────────────
  // Broadcasts are routed to The Guardian; the button may live in a separate tab
  // TODO: adjust tab/button label to match the broadcast section in security/+page.svelte
  await page.getByRole('button', { name: /request broadcast|broadcast/i }).click();

  // Fill in broadcast details
  // TODO: update field labels to match your broadcast request form
  await page.getByLabel(/message|broadcast message/i).fill('Security alert: all non-essential personnel evacuate Sector 4 immediately.');

  // TODO: replace 'Urgent' with the exact option label used in your priority select
  await page.getByLabel(/priority|urgency/i).selectOption({ label: 'Urgent' });

  await page.getByRole('button', { name: /submit|send broadcast|confirm/i }).click();

  await assertNoErrors(page);
  // TODO: adjust to your success notification text
  await expect(page.getByText(/broadcast.*requested|broadcast.*sent|success/i)).toBeVisible();
});

// ---------------------------------------------------------------------------
// 4. Settlers — settler_commander
// ---------------------------------------------------------------------------
test('Settlers: submit anomaly report, view inventory and personnel', async ({ page }) => {
  await login(page, 'settler_commander', 'password');

  // Navigate to the Settlement module
  // TODO: match the exact nav-link label used in your sidebar
  await navigateTo(page, 'Settlement');
  await expect(page).toHaveURL(/settlement/);

  // ── CREATE: Submit an anomaly report ────────────────────────────────────
  // TODO: adjust button label to match settlement/+page.svelte
  await page.getByRole('button', { name: /anomaly report|report anomaly/i }).click();

  // Fill in anomaly details
  // TODO: update field labels to match your anomaly report form
  await page.getByLabel(/title|anomaly title/i).fill('Unusual atmospheric reading — Dome 3');
  await page.getByLabel(/description|details/i).fill('Pressure sensors in Dome 3 reporting values 15% above normal baseline.');
  await page.getByLabel(/location|sector/i).fill('Dome 3');

  await page.getByRole('button', { name: /submit|file report|confirm/i }).click();

  await assertNoErrors(page);
  // TODO: adjust to your success notification text
  await expect(page.getByText(/report.*submitted|anomaly.*filed|success/i)).toBeVisible();

  // ── READ: View settlement inventory ─────────────────────────────────────
  // TODO: adjust tab/section label for the inventory view
  await page.getByRole('tab', { name: /inventory/i }).click();

  await assertNoErrors(page);
  // Assert the inventory list has rendered at least one item
  // TODO: adjust selector to match your inventory list component
  await expect(page.locator('table tbody tr, [data-testid="inventory-item"]').first()).toBeVisible();

  // ── READ: View personnel list ────────────────────────────────────────────
  // TODO: adjust tab/section label for the personnel view
  await page.getByRole('tab', { name: /personnel|crew/i }).click();

  await assertNoErrors(page);
  // Assert the personnel list has rendered at least one entry
  // TODO: adjust selector to match your personnel list component
  await expect(page.locator('table tbody tr, [data-testid="personnel-item"]').first()).toBeVisible();
});


// ---------------------------------------------------------------------------
// 1. Sanitary Department — head_of_sanitary
// ---------------------------------------------------------------------------
test('Sanitary Department: assign task, view inventory, update shift', async ({ page }) => {
  await login(page, 'head_of_sanitary', 'password');

  // Navigate to the Sanitary module
  // TODO: match the exact nav-link label used in your sidebar
  await navigateTo(page, 'Sanitary');
  await expect(page).toHaveURL(/sanitary/);

  // ── CREATE: Assign a new task to the clean-up crew ──────────────────────
  // TODO: adjust to match the actual button / heading text in sanitary/+page.svelte
  await page.getByRole('button', { name: /assign task/i }).click();

  // Fill in task details
  // TODO: update field labels to match your form components
  await page.getByLabel(/task name/i).fill('Clean Sector 7 waste bins');
  await page.getByLabel(/assigned to/i).fill('Clean-Up Crew');

  // TODO: adjust submit button name if needed
  await page.getByRole('button', { name: /submit|save|confirm/i }).click();

  // Assert success — no errors, spinner gone, success message visible
  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // TODO: adjust the success text to what your toast/notification shows
  await expect(page.getByText(/task.*assigned|assigned.*task|success/i)).toBeVisible();

  // ── READ: View the sanitary inventory list ──────────────────────────────
  // TODO: adjust tab/link name to the actual inventory section label
  await page.getByRole('tab', { name: /inventory/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // Assert the inventory table/list has rendered at least one row
  // TODO: adjust the selector to match your inventory table component
  await expect(page.locator('table tbody tr, [data-testid="inventory-item"]').first()).toBeVisible();

  // ── UPDATE: Modify a staff member's shift / schedule ────────────────────
  // TODO: adjust tab/link name to the actual shift section label
  await page.getByRole('tab', { name: /shift|schedule/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();

  // Open the edit flow for the first shift entry
  // TODO: adjust to the actual edit button / icon in your shift component
  await page.locator('[data-testid="edit-shift"], button:has-text("Edit")').first().click();

  // Change the shift time
  // TODO: update field labels to match your shift form
  await page.getByLabel(/start time|shift start/i).fill('08:00');

  await page.getByRole('button', { name: /update|save|confirm/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // TODO: adjust to your toast/notification success message
  await expect(page.getByText(/updated|saved|success/i)).toBeVisible();
});

// ---------------------------------------------------------------------------
// 2. Medical Department — head_of_medicine
// ---------------------------------------------------------------------------
test('Medical Department: submit budget request, open patient log, update inventory', async ({ page }) => {
  await login(page, 'head_of_medicine', 'password');

  // Navigate to the Medical module
  // TODO: match the exact nav-link label used in your sidebar
  await navigateTo(page, 'Medical');
  await expect(page).toHaveURL(/medical/);

  // ── CREATE: Submit a budget request ─────────────────────────────────────
  // TODO: adjust button label to match medical/+page.svelte
  await page.getByRole('button', { name: /budget request|submit budget/i }).click();

  // Fill in budget request form
  // TODO: update field labels to match your form components
  await page.getByLabel(/amount|budget amount/i).fill('5000');
  await page.getByLabel(/reason|description/i).fill('Medical supplies restock for Q3');

  await page.getByRole('button', { name: /submit|send|confirm/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // TODO: adjust to your success notification text
  await expect(page.getByText(/request submitted|budget.*submitted|success/i)).toBeVisible();

  // ── READ: Open a patient log ─────────────────────────────────────────────
  // TODO: adjust tab/section label for the patient log view
  await page.getByRole('tab', { name: /patient log|patients/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();

  // Open the first patient record
  // TODO: adjust to the actual row/card click target in your patient list
  await page.locator('[data-testid="patient-row"], tr[role="row"]').first().click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // Assert the patient detail panel / modal appeared
  // TODO: adjust heading text to match your patient detail component
  await expect(page.getByRole('heading', { name: /patient detail|patient log|record/i })).toBeVisible();

  // ── UPDATE: Update medical inventory ────────────────────────────────────
  // Close modal if present
  // TODO: adjust close button selector if needed
  const closeBtn = page.getByRole('button', { name: /close|dismiss/i });
  if (await closeBtn.isVisible()) await closeBtn.click();

  // TODO: adjust tab/section label for the inventory view
  await page.getByRole('tab', { name: /inventory/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();

  // Edit the first inventory item
  // TODO: adjust edit button selector
  await page.locator('[data-testid="edit-inventory"], button:has-text("Edit")').first().click();

  // TODO: update field labels to match your inventory edit form
  await page.getByLabel(/quantity|stock/i).fill('100');

  await page.getByRole('button', { name: /update|save|confirm/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // TODO: adjust to your success notification text
  await expect(page.getByText(/updated|saved|success/i)).toBeVisible();
});

// ---------------------------------------------------------------------------
// 3. Security Teams — head_of_earth_security
// ---------------------------------------------------------------------------
test('Security Teams: submit incident report and request broadcast', async ({ page }) => {
  await login(page, 'head_of_earth_security', 'password');

  // Navigate to the Security module
  // TODO: match the exact nav-link label used in your sidebar
  await navigateTo(page, 'Security');
  await expect(page).toHaveURL(/security/);

  // ── CREATE: Submit an incident report ────────────────────────────────────
  // TODO: adjust button label to match security/+page.svelte
  await page.getByRole('button', { name: /incident report|report incident/i }).click();

  // Fill in incident details
  // TODO: update field labels to match your incident form components
  await page.getByLabel(/title|incident title/i).fill('Perimeter breach — North Gate');
  await page.getByLabel(/description|details/i).fill('Unauthorised entry detected at north perimeter gate. Crew dispatched.');
  await page.getByLabel(/severity|priority/i).selectOption({ index: 1 });

  await page.getByRole('button', { name: /submit|file report|confirm/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // TODO: adjust to your success notification text
  await expect(page.getByText(/report.*submitted|incident.*filed|success/i)).toBeVisible();

  // ── CREATE: Request a broadcast ──────────────────────────────────────────
  // Broadcasts are routed to The Guardian; the button may live in a separate tab
  // TODO: adjust tab/button label to match the broadcast section in security/+page.svelte
  await page.getByRole('button', { name: /request broadcast|broadcast/i }).click();

  // Fill in broadcast details
  // TODO: update field labels to match your broadcast request form
  await page.getByLabel(/message|broadcast message/i).fill('Security alert: all non-essential personnel evacuate Sector 4 immediately.');
  await page.getByLabel(/priority|urgency/i).selectOption({ index: 0 });

  await page.getByRole('button', { name: /submit|send broadcast|confirm/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // TODO: adjust to your success notification text
  await expect(page.getByText(/broadcast.*requested|broadcast.*sent|success/i)).toBeVisible();
});

// ---------------------------------------------------------------------------
// 4. Settlers — settler_commander
// ---------------------------------------------------------------------------
test('Settlers: submit anomaly report, view inventory and personnel', async ({ page }) => {
  await login(page, 'settler_commander', 'password');

  // Navigate to the Settlement module
  // TODO: match the exact nav-link label used in your sidebar
  await navigateTo(page, 'Settlement');
  await expect(page).toHaveURL(/settlement/);

  // ── CREATE: Submit an anomaly report ────────────────────────────────────
  // TODO: adjust button label to match settlement/+page.svelte
  await page.getByRole('button', { name: /anomaly report|report anomaly/i }).click();

  // Fill in anomaly details
  // TODO: update field labels to match your anomaly report form
  await page.getByLabel(/title|anomaly title/i).fill('Unusual atmospheric reading — Dome 3');
  await page.getByLabel(/description|details/i).fill('Pressure sensors in Dome 3 reporting values 15% above normal baseline.');
  await page.getByLabel(/location|sector/i).fill('Dome 3');

  await page.getByRole('button', { name: /submit|file report|confirm/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // TODO: adjust to your success notification text
  await expect(page.getByText(/report.*submitted|anomaly.*filed|success/i)).toBeVisible();

  // ── READ: View settlement inventory ─────────────────────────────────────
  // TODO: adjust tab/section label for the inventory view
  await page.getByRole('tab', { name: /inventory/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // Assert the inventory list has rendered at least one item
  // TODO: adjust selector to match your inventory list component
  await expect(page.locator('table tbody tr, [data-testid="inventory-item"]').first()).toBeVisible();

  // ── READ: View personnel list ────────────────────────────────────────────
  // TODO: adjust tab/section label for the personnel view
  await page.getByRole('tab', { name: /personnel|crew/i }).click();

  await expect(page.getByText('500 Server Error')).toBeHidden();
  await expect(page.getByTestId('loading-spinner')).toBeHidden();
  // Assert the personnel list has rendered at least one entry
  // TODO: adjust selector to match your personnel list component
  await expect(page.locator('table tbody tr, [data-testid="personnel-item"]').first()).toBeVisible();
});
