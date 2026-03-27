## ✅ Mandatory System-Wide Integration Audit (Post-Migration 018)

This checklist is the release gate for this fix PR and must be fully completed before marking as **Done**.

### 1) Migration Fix Verification

- [ ] Migration 018 updated to include missing role in backfill:
  - `biologist` added to Observer routing condition.
- [ ] Final backfill SQL used:

```sql
UPDATE help_requests hr
SET assigned_proxy_director = CASE
    WHEN r.name IN ('biologist', 'biological_engineer', 'agricultural_engineer', 'chemist') THEN 'the_observer'
    ELSE 'the_artificer'
END
FROM users u
JOIN roles r ON r.id = u.role_id
WHERE hr.requested_by = u.id;
```

- [ ] Migration applies cleanly on:
  - [ ] Fresh database
  - [ ] Existing/staging-like database with pre-existing rows
- [ ] No SQL errors during migrate up/down (if reversible in your setup).

---

### 2) Data Integrity Assertions (Help Requests)

- [ ] `assigned_proxy_director` exists and is non-null for all rows after migration.
- [ ] Rows where requester role is one of:
  - `biologist`
  - `biological_engineer`
  - `agricultural_engineer`
  - `chemist`

  are assigned to `the_observer`.
- [ ] All other requester roles route to `the_artificer`.
- [ ] `new_matter` constraint is present and enforced as intended.
- [ ] No orphaned references or invalid enum/domain values introduced.

---

### 3) Full CRUD Sweep Across Major Feature Tabs

> Execute **Create / Read / Update / Delete** for each module and verify no 500 Server Errors from schema mismatches.

#### Sanitary
- [ ] Create
- [ ] Read/List/Detail
- [ ] Update
- [ ] Delete
- [ ] No 500s (attach evidence)

#### Medical
- [ ] Create
- [ ] Read/List/Detail
- [ ] Update
- [ ] Delete
- [ ] No 500s (attach evidence)

#### Security
- [ ] Create
- [ ] Read/List/Detail
- [ ] Update
- [ ] Delete
- [ ] No 500s (attach evidence)

#### Settlers
- [ ] Create
- [ ] Read/List/Detail
- [ ] Update
- [ ] Delete
- [ ] No 500s (attach evidence)

#### Chemistry
- [ ] Create
- [ ] Read/List/Detail
- [ ] Update
- [ ] Delete
- [ ] No 500s (attach evidence)

#### Help Requests
- [ ] Create
- [ ] Read/List/Detail
- [ ] Update
- [ ] Delete
- [ ] Routing/assignment behavior correct
- [ ] No 500s (attach evidence)

#### Other Production Tabs/Modules (list all)
- [ ] Module: __________ (CRUD + no 500s)
- [ ] Module: __________ (CRUD + no 500s)
- [ ] Module: __________ (CRUD + no 500s)

---

### 4) API/Backend Contract Audit (Rust Struct ↔ SQL Schema)

- [ ] Reviewed all recently changed Rust structs/models against SQL schema.
- [ ] Confirmed all required columns exist for every serialized field used by handlers.
- [ ] Confirmed no removed/renamed SQL columns are still referenced in Rust.
- [ ] Confirmed inserts/updates match actual DB constraints/defaults.
- [ ] Confirmed no endpoint returns 500 due to missing column/constraint.

---

### 5) Evidence Attached to PR

- [ ] Test run logs/screenshots attached.
- [ ] If automated tests exist: integration/e2e output attached.
- [ ] If manual validation used: step-by-step notes attached.
- [ ] Error logs reviewed (backend + DB) and clean for tested flows.

---

## Final Confirmation Statement (Required)

**I confirm that:**
1. Migration 018 was corrected to include `biologist` in Observer routing backfill.
2. A full CRUD integration sweep was completed across all major feature tabs/modules.
3. No additional 500 Server Error schema mismatches were found outside Chemistry/Help Request modules.
4. Evidence is attached in this PR.

**Validated by:** @<your-username>  
**Date:** <YYYY-MM-DD>  
**Environment(s):** <local/staging/etc.>
