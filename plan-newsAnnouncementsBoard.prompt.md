## Plan: News / Announcements Board (Scheduling & Comments)

TL;DR: Announcements system with markdown-based posts supporting public (no auth) and role/house/apartment–scoped visibility, optional pinning, soft-delete, scheduling (publish_at, expire_at), and optional comments (comments_enabled toggle) restricted to authenticated users. Backend adds migrations, announcements & comments handlers with RBAC and scheduling filters, markdown-to-sanitized-HTML rendering. Frontend replaces Home health display with a public announcements list, adds manager editor (markdown + preview) plus visibility, scheduling and comments toggle, and per-announcement comments UI.

### Steps
1. Add Diesel migrations: `migrations/<timestamp>_create_announcements/` with table `announcements` (id PK, title, body_md, body_html, author_id FK users, public bool, pinned bool, roles_csv TEXT nullable, building_id nullable FK, apartment_id nullable FK, comments_enabled BOOL DEFAULT 0, publish_at TIMESTAMP NULL, expire_at TIMESTAMP NULL, is_deleted BOOL default 0, created_at/updated_at TIMESTAMP).
2. Backend module: `api/src/announcements/mod.rs` with handlers (GET /announcements/public, GET /announcements, GET /announcements/{id}, POST/PUT/DELETE/restore/pin) + comments endpoints (GET /announcements/{id}/comments, POST comment, DELETE/restore comment) and visibility + scheduling filtering; register in `api/src/main.rs` via `.configure(announcements::configure)`.
3. Implement markdown render + sanitize (pulldown_cmark + ammonia) for announcements & comments on create/update; add tests in `api/tests/announcements_rbac.rs` for scheduling, public access, role filtering, comments RBAC, soft-delete/restore, pin ordering.
4. Extend schema/models: update `api/src/models.rs` & `api/src/schema.rs` with Announcement and AnnouncementComment structs; ensure `author_id` & `user_id` from JWT; enforce RBAC (Admin/Manager create/update/delete/restore/pin; author can edit own non-deleted; comment delete author or Admin/Manager; comment restore Admin/Manager only).
5. Frontend: replace `frontend/src/pages/home.rs` content with public announcements list component (`AnnouncementList`) using `/api/v1/announcements/public` (fallback to health if empty). Add manager section or route `/manage/announcements` with `AnnouncementEditor` (dual-pane markdown + live preview). Add comments components (`CommentList`, `CommentEditor`) shown only if logged in and comments_enabled.
6. Visibility & Scheduling UI: multi-select roles (Admin/Manager/Owner/Renter/HOA Member), optional building/apartment pickers, public checkbox, pin toggle, comments_enabled toggle, publish_at & expire_at datetime inputs. Serialize JSON accordingly.
7. Soft-delete & restore buttons (only managers/admins) for announcements and comments; hide deleted by default; reuse existing patterns. Maintain ordering: pinned first (created_at desc within pinned) then others (created_at desc) excluding future publish (for public) and expired.
8. Introduce translation keys for announcements & comments (list headers, buttons, scheduling labels, comments disabled message) in `frontend/locales/*/frontend.ftl` and backend messages; update i18n docs.
9. Update tracking file `docs/announcements_todo.md` (phased tasks including scheduling + comments) and link from `README.md`.

### Scheduling Rules
- Public list: exclude rows where publish_at > now; exclude rows where expire_at IS NOT NULL AND expire_at <= now; exclude is_deleted.
- Authenticated managers/admins list: include drafts (publish_at NULL or publish_at > now) and expired optionally via query flag (initially default hide expired); allow filter parameters later.
- Derived status: draft if publish_at > now; expired if expire_at <= now; published otherwise. No explicit status column initially.

### Comments Feature
- Table `announcements_comments`: id, announcement_id, user_id, body_md, body_html, created_at, is_deleted.
- Only visible if announcement.comments_enabled == true.
- Soft-delete comment: mark is_deleted; restore possible by Admin/Manager.
- Markdown sanitized same pipeline as announcements.
- Pagination deferred; initial fetch returns all non-deleted ordered by created_at asc.

### RBAC Summary
- Announcement create/update/delete/restore/pin: roles Admin or Manager; author allowed update if still not deleted.
- Announcement view public: anyone (with publish/expire filters). Private view: must have intersecting role OR be Admin/Manager.
- Comment list/post: authenticated AND comments_enabled AND has visibility to parent announcement.
- Comment delete: author OR Admin/Manager. Restore: Admin/Manager only.

### Data Model (Initial Columns)
Announcements: id BIGINT PK, title VARCHAR(255), body_md TEXT, body_html TEXT, author_id BIGINT FK users, public BOOL, pinned BOOL, roles_csv TEXT NULL, building_id BIGINT NULL FK buildings, apartment_id BIGINT NULL FK apartments, comments_enabled BOOL DEFAULT 0, publish_at TIMESTAMP NULL, expire_at TIMESTAMP NULL, is_deleted BOOL DEFAULT 0, created_at TIMESTAMP DEFAULT NOW, updated_at TIMESTAMP DEFAULT NOW ON UPDATE.
Comments: id BIGINT PK, announcement_id BIGINT FK announcements, user_id BIGINT FK users, body_md TEXT, body_html TEXT, is_deleted BOOL DEFAULT 0, created_at TIMESTAMP DEFAULT NOW.

### Indexes
- announcements: (public, pinned, publish_at, expire_at, is_deleted), (building_id), (apartment_id)
- announcements_comments: (announcement_id, is_deleted, created_at)

### Testing Plan (Backend)
- create_requires_role
- public_list_excludes_future_publish
- public_list_excludes_expired
- manager_list_includes_future_publish
- role_filtered_list_includes_matching
- soft_delete_and_restore_announcement
- pin_ordering
- comments_enabled_gate
- comment_create_requires_auth
- comment_delete_author_or_manager
- comment_restore_manager_only
- comment_soft_delete_visibility

### Frontend Components
- AnnouncementList: displays public list; fallback health message if empty.
- AnnouncementEditor: form (title, markdown textarea, preview pane, visibility controls, scheduling fields, pin & comments_enabled toggles, save/update actions, soft-delete/restore).
- CommentList & CommentEditor: show comments under announcement detail (for logged-in users only when comments_enabled).
- Role chips / multi-select; building/apartment selectors reused; datetime inputs (HTML5 datetime-local) converted to UTC.

### i18n Keys (Examples)
- announcement-title, announcement-body, announcement-publish-at, announcement-expire-at, announcement-pin, announcement-comments-enabled, comment-add, comment-empty, comment-deleted, announcement-draft-badge, announcement-expired-badge.

### Security
- Sanitize HTML via ammonia; disallow script/style/iframe. Only allow basic formatting (links, lists, emphasis, code).
- Rate limit comment posting (future enhancement). For now ensure length bounds.

### Performance & Future
- CSV roles adequate short-term; migration path: create join table `announcement_roles` later.
- Future enhancements (attachments, websocket notifications, pagination, edit history, reactions) tracked separately.

### Acceptance (Phase 2)
- Public homepage shows only published & not expired announcements; pinned first.
- Managers/Admins can create scheduled announcements, toggle comments & pin, soft-delete & restore.
- Private announcements visible only to appropriate roles (or Admin/Manager).
- Comments appear only when enabled and sanitized.
- Tests for scheduling, visibility, comments RBAC pass.

### Deferred Items
- WebSocket push updates.
- Attachments on announcements.
- Pagination & search.
- Edit history & status column.
- Roles normalization.
- Comment moderation & reactions.

### Refinements (Ongoing Phase 2)
- Implement role multi-select UI (checkboxes: Admin, Manager, Homeowner, Renter, HOA Member) replacing raw CSV entry.
- Add building & apartment selectors (cascading: choose building -> load apartments) in editor; include in visibility rules.
- Show audience line on each card/detail: `Public` or `Roles: X` plus optional `Building: #` / `Apartment: #`.
- Show badge for deleted comments when Show Deleted is enabled (e.g. outline-danger 'Deleted').
- Add purge confirmation modal to prevent accidental permanent deletion.
- Enhance markdown rendering style: apply Bootstrap classes to headings, lists, code blocks; optional syntax highlighting deferred.
- Editor visibility hint: summary box showing who will see the announcement based on current settings.
- Scheduled/Expired badges (Implemented) – verify with tests.
