# Announcements / News Board TODO (Updated: Scheduling & Comments Included)

Status Legend: [ ] pending, [~] in progress, [x] done, [>] deferred

## Phase 1: Data & Backend Core
[x] Migration: create `announcements` table (columns: id, title, body_md TEXT, body_html TEXT, author_id FK users.id, public BOOL, pinned BOOL, roles_csv TEXT NULL, building_id FK buildings.id NULL, apartment_id FK apartments.id NULL, comments_enabled BOOL DEFAULT 0, publish_at TIMESTAMP NULL, expire_at TIMESTAMP NULL, is_deleted BOOL DEFAULT 0, created_at TIMESTAMP DEFAULT NOW, updated_at TIMESTAMP DEFAULT NOW ON UPDATE))
[x] Migration: create `announcements_comments` table (id, announcement_id FK announcements.id, user_id FK users.id, body_md TEXT, body_html TEXT, is_deleted BOOL DEFAULT 0, created_at TIMESTAMP DEFAULT NOW)
[x] Add to `schema.rs` and verify mapping
[x] Models: Add Announcement/NewAnnouncement + AnnouncementComment/NewAnnouncementComment in `models.rs`
[x] Markdown rendering pipeline (pulldown_cmark + ammonia) for announcements & comments
[x] RBAC: announcements (Admin/Manager; author can edit own non-deleted), comments (post/list if authenticated & comments_enabled; delete author or Admin/Manager; restore Admin/Manager)
[x] Scheduling filter helpers (public vs manager scope)
[x] Query helpers: visibility by roles_csv, building/apartment, scheduling, is_deleted (implemented inline in handlers)
[x] Handlers:
    - [x] GET /api/v1/announcements/public
    - [x] GET /api/v1/announcements
    - [x] GET /api/v1/announcements/{id}
    - [x] POST /api/v1/announcements
    - [x] PUT /api/v1/announcements/{id}
    - [x] DELETE /api/v1/announcements/{id} (soft)
    - [x] POST /api/v1/announcements/{id}/restore
    - [x] POST /api/v1/announcements/{id}/pin (toggle)
    - [x] GET /api/v1/announcements/{id}/comments
    - [x] POST /api/v1/announcements/{id}/comments
    - [x] DELETE /api/v1/announcements/comments/{id} (soft)
    - [x] POST /api/v1/announcements/comments/{id}/restore
[x] Ordering: pinned first (created_at desc), then others (created_at desc)
[x] Error types additions (NotPublished, Expired, CommentsDisabled)
[x] Indices: announcements (public, pinned, publish_at, expire_at, is_deleted), building_id, apartment_id; comments (announcement_id, is_deleted, created_at)
[x] Wire module in `main.rs`

## Phase 2: Frontend Integration
[x] Replace health display in `home.rs` with AnnouncementList
[x] Component: `components/announcement_list.rs`
[x] Component: `components/announcement_editor.rs` (markdown + preview + scheduling + visibility + comments toggle)
[x] Components: `components/comment_list.rs` (basic), `components/comment_editor.rs` (merged in list for now)
[x] Manager page integration (section in existing manage page)
[x] Visibility UI: role multi-select (checkboxes; replaces CSV input)
[x] Building/apartment selectors (editor cascading dropdowns)
[x] Audience line (Public / Roles / Building / Apartment) on cards & detail
[x] Comment delete/restore controls (UI implemented with show deleted toggle and deleted badge)
[x] Scheduling inputs (publish_at, expire_at) using datetime-local -> UTC conversion
[x] Pin & comments_enabled toggles
[x] CRUD flows with spinners (basic) & refresh; optimistic updates deferred
[x] Soft-deleted announcements panel & restore
[x] Empty states (no announcements / no comments)
[x] Public announcements list: expandable comments (Show/Hide Comments) for announcements with comments_enabled
[x] Anonymous public comment viewing (list without auth; only authenticated can post; managers see deleted)

## Phase 3: i18n & UX Polish
[~] Add translation keys (announcement-title, publish-at, expire-at, comments-enabled, add-comment, draft, expired, etc.) -- core keys + editor + error messages added; remaining: role descriptions, options help text.
[x] Language persistence (store selected language in localStorage)
[x] Translate announcement editor form & error messages
[~] Localize date/time formatting (basic EN/CZ implemented; full ICU deferred)
[x] Remaining static texts localized (private label, preview empty/rendered, none-option, role labels)
[ ] Accessibility (aria-labels for editor and comment form)
[ ] Markdown styling (Bootstrap classes) & sanitized display wrapper
[ ] Display badges (Draft / Expired / Pinned)

## Phase 4: Testing & Quality
[ ] Backend tests `api/tests/announcements_rbac.rs`:
    - [ ] create_requires_role
    - [ ] public_list_excludes_future_publish
    - [ ] public_list_excludes_expired
    - [ ] manager_list_includes_future_publish
    - [ ] role_filtered_list_includes_matching
    - [ ] soft_delete_and_restore_announcement
    - [ ] purge_permanently_removes_announcement_and_comments
    - [ ] pin_ordering
    - [ ] comments_enabled_gate
    - [ ] comment_create_requires_auth
    - [ ] comment_delete_author_or_manager
    - [ ] comment_restore_manager_only
    - [ ] comment_soft_delete_visibility
    - [ ] publish_now_draft_to_published
    - [ ] comment_delete_restore_flow_ui (frontend integration test placeholder)
    - [ ] scheduled_and_expired_badges_render
[ ] Security review: sanitized HTML (no scripts)
[ ] Performance: confirm indices effective; consider limit/pagination for comments (deferred)
[ ] Frontend tests (if infra) for comment enabled gating & draft badge rendering

## Phase 5: Deferred / Future Enhancements
[>] Attachments on announcements
[>] WebSocket push for new announcements & live comments
[>] Pagination & search (title/body, comments)
[>] Edit history & versioning
[>] Roles normalization (join table)
[>] Comment reactions + moderation workflow
[>] Rate limiting for comment spam

## Cross-Cutting
[ ] README: link to plan and summarize usage & API
[ ] Logging: include publish_at/expire_at in create/update logs
[ ] Metrics counters (announcements_created, comments_created, announcements_pinned)
[ ] Consistent JSON error bodies

## Acceptance Criteria (Phase 2 complete)
- Public homepage shows only published & unexpired announcements (pinned first)
- Managers/Admins can schedule, pin, toggle comments, soft-delete & restore
- Private announcements visible only to authorized roles (or Admin/Manager)
- Comments appear only when enabled & user authenticated; RBAC enforced
- Sanitized HTML prevents XSS
- Tests pass for scheduling, visibility, pinning, comments RBAC

## References
- Plan file: `plan-newsAnnouncementsBoard.prompt.md`
