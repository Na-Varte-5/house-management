# Common messages for the House Management System Frontend

## General
app-name = House Management System
welcome = Welcome to the House Management System
loading = Loading...
no-data = No data available

## Navigation
nav-home = Home
nav-dashboard = Dashboard
nav-properties = Properties
nav-users = Users
nav-settings = Settings
nav-logout = Logout

## Navigation Extended
nav-buildings = Buildings
nav-health = Health
nav-admin = Admin
nav-manage = Manage
nav-login = Login
nav-maintenance = Maintenance
nav-voting = Voting
nav-my-properties = My Properties
nav-meters = Meters
nav-navigation = Navigation

## User interface
ui-language = Language
ui-language-en = English
ui-language-cs = Czech
ui-theme = Theme
ui-theme-light = Light
ui-theme-dark = Dark

## Buttons
button-save = Save
button-cancel = Cancel
button-delete = Delete
button-edit = Edit
button-add = Add
button-search = Search
button-back = Back
button-submit = Submit
button-create = Create
button-close = Close
button-confirm = Confirm

## Buttons Extended
button-publish = Publish
button-update = Update
button-pin = Pin
button-unpin = Unpin

## Form labels
label-email = Email
label-name = Name
label-password = Password
label-confirm-password = Confirm Password
label-full-name = Full Name
label-description = Description
label-title = Title
label-status = Status
label-priority = Priority
label-type = Type
label-apartment = Apartment
label-building = Building
label-search = Search
label-unit = Unit

## Messages
message-saved = Changes saved successfully
message-deleted = Item deleted successfully
message-error = An error occurred
message-success = Operation completed successfully

## Errors
error-delete-failed = Delete failed
error-restore-failed = Restore failed
error-purge-failed = Purge failed
error-post-failed = Failed to post
error-network = Network error
error-load-failed = Load failed
error-permission-denied = Permission denied
error-access-denied = Access denied

## Roles
role-admin = Admin
role-manager = Manager
role-homeowner = Homeowner
role-renter = Renter
role-hoa-member = HOA Member

## None Option
none-option = -- none --

## Previews
preview-empty = Nothing to preview...
preview-rendered = Rendered

## Sidebar
sidebar-management = Management
sidebar-user-management = User management
sidebar-admin-announcements = Announcements
sidebar-admin-properties = Properties
sidebar-meters = Meters

## Pagination
pagination-items-total = { $count } items total
pagination-previous = Previous
pagination-next = Next

# ============================================================
# Login / Auth
# ============================================================

login-title = House Management
login-sign-in = Sign In
login-register = Register
login-sign-in-to-continue = Sign in to continue
login-create-account = Create your account
login-already-signed-in = You are already signed in
login-welcome-user = Welcome, { $name }!
login-failed = Login failed
login-register-failed = Registration failed
login-register-then-login-failed = Registered, but login failed
login-invalid-credentials = Invalid email or password
login-failed-decode-token = Failed to decode token
login-create-account-button = Create Account

# ============================================================
# Dashboard / Home
# ============================================================

dashboard-title = Dashboard
dashboard-open-maintenance = Open Maintenance
dashboard-active-proposals = Active Proposals
dashboard-my-properties = My Properties
dashboard-pending-votes = Pending Votes
dashboard-view-requests = View Requests
dashboard-view-proposals = View Proposals
dashboard-view-properties = View Properties
dashboard-vote-now = Vote Now
dashboard-recent-announcements = Recent Announcements
dashboard-meter-calibration-due = Meter Calibration Due
dashboard-meters-need-calibration = { $count } meter(s) need calibration within 30 days
dashboard-getting-started = Getting Started
dashboard-no-properties = You don't have any properties assigned yet. Contact your building administrator to get access, or check for pending invitations.

## Home page (unauthenticated)
home-hero-title = House Management
home-hero-subtitle = Manage your property, maintenance, and community — all in one place.
home-sign-in-button = Sign In to Get Started

# ============================================================
# Announcements
# ============================================================

announcements-title = Announcements
announcements-manage-subtitle = Create, edit, and manage building or community announcements.
announcement-title-label = Title
announcement-body-label = Body (Markdown)
announcement-preview-tab = Preview
announcement-edit-tab = Edit
announcement-publish-at-label = Publish At
announcement-expire-at-label = Expire At
announcement-pin-label = Pin
announcement-comments-enabled-label = Enable Comments
announcement-roles-label = Roles
announcement-public-label = Public
announcement-building-label = Building
announcement-apartment-label = Apartment
announcement-audience-heading = Audience
announcement-publish-now = Publish Now
announcement-status-draft = Draft
announcement-status-scheduled = Scheduled
announcement-status-expired = Expired
announcement-status-pinned = Pinned
announcement-no-expiry = No expiry
announcement-published-now = Published: now
announcement-show-comments = Show Comments
announcement-hide-comments = Hide Comments
announcement-no-items = No announcements available.
announcement-soft-deleted-badge = Deleted
announcement-restore = Restore
announcement-purge = Purge
announcement-roles-help = Select one or more roles; leave empty for all (public depends on Public checkbox).
announcement-options-heading = Options
announcement-private-label = Private
announcement-new-button = New announcement
announcement-show-deleted-toggle = Show deleted

## Announcement Prefixes
announcement-by-prefix = By
announcement-building-prefix = Bldg:
announcement-apartment-prefix = Apt:
announcement-published-prefix = Published:
announcement-expires-prefix = Expires:

## Comments
comments-heading = Comments
comments-disabled = Comments disabled.
comments-empty = No comments yet.
comments-login-to-comment = Login to comment.
comment-add-placeholder = Add a comment...
comment-post-button = Post
comment-deleted-badge = Deleted
comment-delete-button = Del
comment-restore-button = Restore
comment-purge-button = Purge
comment-show-deleted-toggle = Show deleted

## Comment Errors
comment-delete-failed = Delete failed
comment-restore-failed = Restore failed
comment-purge-failed = Purge failed
comment-post-failed = Failed to post

# ============================================================
# Maintenance
# ============================================================

maintenance-title = Maintenance Requests
maintenance-new-request = + New Request
maintenance-search-placeholder = Search requests...
maintenance-no-requests = No maintenance requests found. Create your first request above.
maintenance-no-requests-status = No requests with this status.
maintenance-back-to-list = ← Back to List
maintenance-request-not-found = Request not found

## Maintenance New
maintenance-new-title = New Maintenance Request
maintenance-new-breadcrumb = New Request
maintenance-request-details = Request Details
maintenance-request-details-desc = Provide information about the maintenance issue
maintenance-loading-apartments = Loading apartments...
maintenance-select-apartment = -- Select Apartment --
maintenance-select-apartment-help = Select the apartment with the maintenance issue
maintenance-request-type-help = Category of the maintenance issue
maintenance-priority-help = How urgent is this issue?
maintenance-title-placeholder = Brief description of the issue
maintenance-description-placeholder = Detailed description of the maintenance issue
maintenance-description-help = Include any relevant details that will help us address the issue
maintenance-created-redirect = Request created successfully! Redirecting...
maintenance-no-permission-create = You don't have permission to create requests
maintenance-select-apartment-error = Please select an apartment
maintenance-title-required = Title is required
maintenance-description-required = Description is required
maintenance-invalid-apartment = Invalid apartment selected

## Maintenance Detail
maintenance-detail-description = Description
maintenance-detail-apartment = Apartment:
maintenance-detail-created-by = Created by:
maintenance-detail-assigned-to = Assigned to:
maintenance-detail-created = Created:
maintenance-detail-unassigned = Unassigned

## Maintenance Types
maintenance-type-general = General
maintenance-type-plumbing = Plumbing
maintenance-type-electrical = Electrical
maintenance-type-hvac = HVAC
maintenance-type-appliance = Appliance
maintenance-type-structural = Structural
maintenance-type-other = Other

## Maintenance Statuses
maintenance-status-all = All
maintenance-status-open = Open
maintenance-status-in-progress = In Progress
maintenance-status-resolved = Resolved

## Maintenance Priorities
maintenance-priority-low = Low
maintenance-priority-medium = Medium
maintenance-priority-high = High
maintenance-priority-urgent = Urgent

## Maintenance Management Panel
maintenance-management = Management
maintenance-update-status = Update Status
maintenance-update-priority = Update Priority
maintenance-assign-to = Assign To
maintenance-unassigned-option = -- Unassigned --
maintenance-update-status-btn = Update Status
maintenance-update-priority-btn = Update Priority
maintenance-assign-request-btn = Assign Request
maintenance-status-updated = Status updated successfully
maintenance-priority-updated = Priority updated successfully
maintenance-assigned-success = Request assigned successfully
maintenance-no-permission-update = You don't have permission to update requests
maintenance-no-permission-assign = You don't have permission to assign requests

## Maintenance History
maintenance-history = History
maintenance-no-history = No history available
maintenance-changed-status = Changed status from
maintenance-status-to = to
maintenance-status-none = (none)

## Maintenance Comments
maintenance-comments = Comments
maintenance-loading-comments = Loading comments...
maintenance-no-comments = No comments yet. Be the first to comment!
maintenance-add-comment = Add a Comment
maintenance-comment-placeholder = Write your comment here...
maintenance-post-comment = Post Comment
maintenance-comment-posted = Comment posted successfully
maintenance-comment-deleted = Comment deleted successfully

## Maintenance Attachments
maintenance-attachments = Attachments
maintenance-no-attachments = No attachments
maintenance-upload-file = Upload File
maintenance-uploading = Uploading...
maintenance-upload-success = File uploaded successfully
maintenance-download = Download

## Maintenance Escalation
maintenance-escalate = Escalate Request
maintenance-escalate-desc = Need help from a building manager? Escalate this request for faster resolution.
maintenance-escalate-to-manager = Escalate to Manager
maintenance-select-manager = Select a manager
maintenance-select-manager-option = -- Select Manager --
maintenance-escalating = Escalating...
maintenance-confirm-escalation = Confirm Escalation
maintenance-no-managers = No managers assigned to this building. Please contact administration.
maintenance-loading-managers = Loading managers...

## Maintenance Error Messages
maintenance-failed-load = Failed to load request: {$error}
maintenance-failed-post-comment = Failed to post comment: {$error}
maintenance-failed-delete-comment = Failed to delete comment: {$error}
maintenance-failed-upload = Failed to upload file: {$error}
maintenance-failed-update-status = Failed to update status: {$error}
maintenance-failed-update-priority = Failed to update priority: {$error}
maintenance-failed-assign = Failed to assign request: {$error}
maintenance-failed-escalate = Failed to escalate: {$error}
maintenance-cancel = Cancel
maintenance-posting = Posting...
maintenance-delete-comment-confirm = Are you sure you want to delete this comment?

# ============================================================
# Voting
# ============================================================

voting-title = Proposals
voting-new-proposal = + New Proposal
voting-search-placeholder = Search proposals...
voting-no-proposals = No proposals found.
voting-loading = Loading proposals...

## Voting Statuses
voting-status-scheduled = Scheduled
voting-status-open = Open
voting-status-closed = Closed
voting-status-tallied = Tallied

## Voting Methods
voting-method-simple = Simple Majority
voting-method-weighted = Weighted by Area
voting-method-per-seat = One Vote Per Seat
voting-method-consensus = Consensus Required

## Voting New
voting-new-title = Create New Proposal
voting-new-breadcrumb = New Proposal
voting-basic-info = Basic Information
voting-basic-info-desc = Enter the proposal details
voting-proposal-title = Proposal Title
voting-proposal-title-placeholder = Enter proposal title
voting-proposal-description = Description
voting-proposal-description-placeholder = Describe the proposal in detail
voting-proposal-description-help = Provide context for the proposal
voting-voting-settings = Voting Settings
voting-voting-settings-desc = Configure how voting will work
voting-voting-method = Voting Method
voting-voting-method-help = Choose how votes will be counted
voting-start-time = Start Time
voting-start-time-help = When should voting begin?
voting-end-time = End Time
voting-end-time-help = When should voting close?
voting-eligible-roles = Eligible Roles
voting-eligible-roles-desc = Select which roles can vote on this proposal
voting-create-proposal = Create Proposal
voting-creating = Creating...
voting-created-redirect = Proposal created! Redirecting...
voting-proposal-title-required = Title is required
voting-proposal-description-required = Description is required
voting-start-time-required = Start time is required
voting-end-time-required = End time is required
voting-at-least-one-role = At least one role must be eligible
voting-no-permission-create = Only Admins and Managers can create proposals.
voting-no-permission-tally = You don't have permission to tally results
voting-not-eligible-vote = You are not eligible to vote on this proposal
voting-proposal-created = Proposal created successfully
voting-results-tallied = Results tallied successfully
voting-failed-load = Failed to load proposals: {$error}
voting-failed-load-proposal = Failed to load proposal: {$error}
voting-failed-create = Failed to create proposal: {$error}
voting-failed-vote = Failed to cast vote: {$error}
voting-failed-tally = Failed to tally results: {$error}
voting-vote-cast-choice = Vote cast: {$choice}
voting-no-proposals-check-later = No proposals found. Check back later for voting opportunities.
voting-no-proposals-match = No proposals match your search.
voting-can-vote = Can Vote
voting-view-only = View Only
voting-voting-label = Voting:
voting-eligible-label = Eligible:
voting-breadcrumb-detail = Proposal Details
voting-voting-method-label = Voting Method:
voting-eligible-voters-label = Eligible Voters:
voting-start-time-label = Start Time:
voting-end-time-label = End Time:
voting-vote-counts = Vote Counts
voting-total-votes-label = Total votes: {$count}
voting-your-current-vote = Your current vote: {$vote}
voting-final-results = Final Results
voting-proposal-passed = Proposal PASSED
voting-proposal-failed = Proposal FAILED
voting-yes-weight = Yes Weight:
voting-no-weight = No Weight:
voting-abstain-weight = Abstain Weight:
voting-total-weight-label = Total Weight:
voting-tallied-at = Tallied at:
voting-can-change-vote = You can change your vote at any time before voting closes.
voting-info = Voting Information
voting-not-eligible-msg = Not Eligible
voting-not-eligible-desc = You are not eligible to vote on this proposal. Only members with the following roles can vote: {$roles}
voting-closed-tally-desc = Voting is closed. Tally the results to finalize the outcome.
voting-scheduled-msg = Voting hasn't started yet. It will begin on {$date}.
voting-closed-awaiting = Voting is closed. Results are being processed.
voting-building-scope = Building Scope
voting-building-scope-help = Leave as Global to make this proposal visible to all users, or select a building to restrict visibility
voting-global-scope = Global (visible to all buildings)
voting-method-simple-desc = Simple Majority (1 person = 1 vote)
voting-method-weighted-desc = Weighted by Area (vote weight = apartment size)
voting-method-per-seat-desc = Per Seat (1 apartment = 1 vote)
voting-method-consensus-desc = Consensus (no 'No' votes allowed)
voting-role-admin = Admins
voting-role-manager = Managers
voting-role-homeowner = Homeowners
voting-role-renter = Renters
voting-role-hoa = HOA Members

## Voting Detail
voting-results = Results
voting-total-votes = Total Votes
voting-total-weight = Total Weight
voting-yes-votes = Yes
voting-no-votes = No
voting-abstain-votes = Abstain
voting-cast-vote = Cast Your Vote
voting-vote-yes = Vote Yes
voting-vote-no = Vote No
voting-vote-abstain = Abstain
voting-tally-results = Tally Results
voting-tallying = Tallying...
voting-vote-cast = Vote cast
voting-passed = Passed
voting-failed = Failed
voting-pending = Pending
voting-your-vote = Your Vote
voting-not-voted = Not voted
voting-status = Status
voting-method = Voting Method
voting-start = Start
voting-end = End
voting-not-started-yet = Voting has not started yet.
voting-has-closed = Voting has closed.
voting-results-tallied-msg = Results have been tallied.
voting-not-available = Voting is not available.
voting-proposal-not-found = Proposal not found
voting-management = Management

# ============================================================
# Meters
# ============================================================

meters-title = Water Meters
meters-management-title = Meter Management
meters-register-new = + Register New Meter
meters-no-meters = No meters registered for this apartment.
meters-no-meters-admin = Click 'Register New Meter' to add one.
meters-no-meters-found = No meters found. Register a meter to get started.

## Meters Table Headers
meters-building = Building
meters-apartment = Apartment
meters-type = Type
meters-serial-number = Serial Number
meters-last-reading = Last Reading
meters-installation-date = Installation Date
meters-calibration-due = Calibration Due
meters-calibration-status = Cal. Status
meters-no-readings = No readings

## Meter Types
meter-type-cold-water = Cold Water
meter-type-hot-water = Hot Water
meter-type-gas = Gas
meter-type-electricity = Electricity

## Meter Calibration Status
meter-calibration-overdue = Overdue by { $days } days
meter-calibration-overdue-short = Overdue ({ $days } days)
meter-calibration-due-soon = Due in { $days } days
meter-calibration-valid = Valid ({ $days } days)
meter-calibration-unknown = Unknown
meter-calibration-not-set = Not set

## Meter Filters
meters-filter-all-buildings = All Buildings
meters-filter-all-statuses = All Statuses
meters-filter-overdue = Overdue
meters-filter-due-soon = Due Soon (30 days)
meters-filter-valid = Valid
meters-filter-by-building = Filter by Building
meters-filter-search-placeholder = Search by serial, type, or apartment...

## Meter Register Form
meters-meter-location = Meter Location
meters-meter-location-desc = Select the building and apartment for this meter
meters-select-building = Select building...
meters-select-apartment = Select apartment...
meters-meter-details = Meter Details
meters-meter-details-desc = Enter the meter type and serial number
meters-serial-placeholder = Enter meter serial number
meters-dates-title = Dates (Optional)
meters-dates-desc = Set installation and calibration dates
meters-register-meter = Register Meter
meters-select-apartment-error = Please select an apartment
meters-serial-required = Serial number is required
meters-invalid-apartment = Invalid apartment selected

## Meter Detail
meters-detail-info = Meter Information
meters-detail-meter-type = Meter Type
meters-detail-serial = Serial Number
meters-detail-installation = Installation Date
meters-detail-calibration-due = Calibration Due
meters-detail-last-calibration = Last Calibration
meters-detail-active = Active
meters-detail-visible-renters = Visible to Renters
meters-detail-yes = Yes
meters-detail-no = No
meters-detail-not-set = Not set

## Meter Reading
meters-reading-history = Reading History
meters-export-csv = Export CSV
meters-no-readings-yet = No readings recorded yet.
meters-reading-timestamp = Timestamp
meters-reading-value = Reading Value
meters-reading-unit = Unit
meters-reading-source = Source

## Meter Manual Reading
meters-add-reading = Add Manual Reading
meters-reading-value-label = Reading Value
meters-reading-value-placeholder = 123.456
meters-reading-unit-label = Unit
meters-unit-m3 = m³ (cubic meters)
meters-unit-kwh = kWh (kilowatt-hours)
meters-unit-liters = L (liters)
meters-save-reading = Save Reading
meters-reading-required = Reading value is required
meters-reading-success = Reading recorded successfully

## Meter Edit
meters-edit-title = Edit/Replace Meter
meters-edit-desc = Update meter details or replace with a new serial number (e.g., after calibration)
meters-edit-serial-placeholder = Enter new serial number
meters-edit-serial-help = Update this when replacing the physical meter
meters-edit-calibration-help = Set new calibration due date after replacement
meters-visible-to-renters = Visible to Renters
meters-update-meter = Update Meter

## Meter Management Tabs
meters-tab-list = Meter List
meters-tab-register = Register New

## Meter Calibration
meters-calibration-title = Meter Calibration Due
meters-calibration-days-before = Days before due date
meters-calibration-check = Check
meters-calibration-no-meters = No meters found requiring calibration in this period.

# ============================================================
# Properties
# ============================================================

properties-my-title = My Properties
properties-total = Total Properties
properties-active-maintenance = Active Maintenance Requests
properties-pending-votes = Pending Votes
properties-your-properties = Your Properties
properties-no-properties = You don't have any properties yet. Contact your building administrator to get access.
properties-click-to-view = Click to view details
properties-owner = Owner
properties-active-renter = Active Renter
properties-past-renter = Past Renter
properties-start-date = Start:
properties-end-date = End:
properties-bed = {$count} bed
properties-bath = {$count} bath
properties-failed-load = Failed to load properties: {$error}

## Buildings
buildings-title-all = All Buildings
buildings-title-my = My Buildings
buildings-search-placeholder = Search buildings...
buildings-no-buildings = No Buildings Found
buildings-no-buildings-desc = You don't have any apartments assigned yet. Contact your building administrator to get access.
buildings-no-match = No buildings match your search.
buildings-view-apartments = View apartments
buildings-built = Built {$year}

## Building Apartments
buildings-no-apartments = No apartments found in this building.
buildings-apartments-in = Apartments in {$address}
buildings-no-access = No apartments found in this building that you have access to.
buildings-id = ID
buildings-number = Number
buildings-size = Size (m²)
buildings-failed-load = Failed to load buildings: {$error}
buildings-failed-load-data = Failed to load data: {$error}

# ============================================================
# User Management
# ============================================================

users-title = User Management
users-admin-only = Only Admins can access user management.
users-no-users = No users found.
users-loading = Loading users...
users-id = ID
users-name = Name
users-email = Email
users-roles = Roles
users-add-role = Add Role
users-select-role = -- Select Role --
users-role-help = Remove a role by clicking the × on its badge. Add a role using the dropdown.
users-role-added = Role '{$role}' added successfully
users-role-removed = Role '{$role}' removed successfully
users-permission-denied = You don't have permission to update roles
users-failed-load = Failed to load users: {$error}
users-failed-reload = Failed to reload users: {$error}
users-failed-add-role = Failed to add role: {$error}
users-failed-remove-role = Failed to remove role: {$error}

# ============================================================
# Admin / Manage Dashboard
# ============================================================

admin-access-denied = Access denied
admin-need-permission = You need Admin or Manager permissions to access this page.
manage-dashboard-title = Dashboard
manage-dashboard-desc = Use the sidebar or the quick links below to manage users, announcements, and properties.
manage-user-management = User Management
manage-user-management-desc = View users and assign roles.
manage-go-to-users = Go to users
manage-announcements = Announcements
manage-announcements-desc = Create and manage building/community announcements.
manage-go-to-announcements = Go to announcements
manage-properties = Properties
manage-properties-desc = Manage buildings, apartments, and owners.
manage-go-to-properties = Go to properties
manage-properties-title = Properties Management

# ============================================================
# Health
# ============================================================

health-title = System Health
health-status = Status:
health-message = Message:

# ============================================================
# Building Apartments
# ============================================================

building-apartments-title = Apartments in {$building}
building-apartments-none = No apartments found in this building that you have access to.
building-apartments-id = ID
building-apartments-number = Number
building-apartments-size = Size (m²)

# ============================================================
# Meter Extra Keys
# ============================================================

meters-back = Back
meters-access-denied = Access denied. Only Admins and Managers can access meter management.
meters-detail-title = Meter Details
meters-detail-not-found = Meter not found
meters-edit-replace = Edit/Replace Meter
meters-detail-type = Type:
meters-detail-serial-label = Serial Number:
meters-detail-installation-label = Installation Date:
meters-detail-calibration-due-label = Calibration Due:
meters-detail-last-calibration-label = Last Calibration:
meters-detail-visible-renters-label = Visible to Renters:
meters-serial-label = Serial:
meters-last-reading-label = Last Reading:
meters-no-readings-short = No readings yet
meters-installed-label = Installed:
meters-calibration-dashboard = Calibration Dashboard
meters-show-due-within = Show meters due within:
meters-calibration-serial = Serial:
meters-calibration-due-label = Calibration Due:
meters-calibration-last = Last Calibrated:
meters-calibration-installed = Installed:
meters-calibration-click = Click to view meter details and record calibration
meters-calibration-legend = Legend:
meters-calibration-overdue-legend = Overdue / Due in ≤7 days
meters-calibration-warning-legend = Due in 8-30 days
meters-calibration-info-legend = Due in >30 days
meters-register-title = Register New Meter
meters-when-recalibrate = When the meter needs to be recalibrated/renewed
meters-register-cancel = Cancel
meters-list-meters = List Meters
meters-register-meter-tab = Register Meter
meters-apartment-prefix = Apartment
meters-overdue-by-days = Overdue by {$days} days
meters-due-in-days = Due in {$days} days
meters-valid-days = Valid ({$days} days)
meters-unknown = Unknown
meters-cold-water = Cold Water
meters-hot-water = Hot Water
meters-gas = Gas
meters-electricity = Electricity
meters-meter-registered = Meter registered successfully
meters-failed-load = Failed to load meters: {$error}
meters-failed-load-meter = Failed to load meter: {$error}
meters-failed-load-readings = Failed to load readings: {$error}
meters-failed-load-buildings = Failed to load buildings: {$error}
meters-failed-load-apartments = Failed to load apartments: {$error}
meters-failed-register = Failed to register meter: {$error}
meters-failed-export = Failed to export: {$error}
meters-permission-denied = Permission denied
meters-meter-updated = Meter updated successfully
meters-select-building-option = -- Select Building --
meters-select-apartment-option = -- Select Apartment --
meters-no-meters-in-period = No meters requiring calibration in the selected time period.
meters-meters-in-period = {$count} meter(s) requiring calibration within {$days} days
meters-overdue-label = OVERDUE by {$days} days
meters-my-properties = My Properties
meters-no-apartments-for-meters = No meters registered for this apartment.

# ============================================================
# Properties Components
# ============================================================

properties-select-building = Select a building to view its apartments
properties-no-apartments-create = No apartments in this building. Create one using the form below.
properties-create-building = Create New Building
properties-create-building-btn = Create Building
properties-no-buildings-create = No buildings found. Create one using the form below.
properties-select-apartment = Select an apartment to manage its owners
properties-current-owners = Current Owners
properties-no-owners = No owners assigned to this apartment
properties-assign-owner = Assign New Owner
properties-no-matching-users = No matching users found
properties-create-apartment = Create New Apartment
properties-select-building-create = Select a building to create apartments
properties-create-apartment-btn = Create Apartment
properties-back-to-properties = Back to My Properties
properties-apartment-label = Apartment
properties-manage-renters = Manage Renters
properties-address-placeholder = Address
properties-construction-year-placeholder = Construction Year (optional)
properties-apartment-number-placeholder = Apartment Number
properties-size-placeholder = Size (m²) - optional
properties-search-users = Search users...
properties-email-placeholder = Email address
properties-renter-assigned = Renter assigned successfully
properties-renter-removed = Renter removed successfully
properties-failed-assign = Failed to assign renter: {$error}
properties-failed-remove = Failed to remove renter: {$error}
properties-failed-load-apartment = Failed to load apartment: {$error}
properties-failed-load-permissions = Failed to load permissions: {$error}
properties-failed-load-meters = Failed to load meters: {$error}
properties-failed-load-renters = Failed to load renters: {$error}
properties-invitation-sent = Invitation sent successfully
properties-invitation-cancelled = Invitation cancelled
properties-failed-invite = Failed to send invitation: {$error}
properties-failed-cancel-invitation = Failed to cancel invitation: {$error}
properties-renter-marked = Renter marked as {$status}
properties-failed-update-renter = Failed to update renter: {$error}
properties-failed-load-history = Failed to load property history: {$error}
properties-unknown-date = Unknown date
properties-by-user = by {$name}
properties-not-set = Not set
properties-meters-tab = Meters
properties-renters-tab = Renters
properties-history-tab = History
properties-property-history = Property History
properties-no-history = No property history events yet.
properties-select-apartment-history = Select an apartment to view its property history

## Renter Management
renters-select-apartment = Select an apartment to manage its renters
renters-current = Current Renters
renters-no-active = No active renters for this apartment
renters-active = Active
renters-start = Start:
renters-end = End:
renters-ongoing = Ongoing
renters-past = Past Renters
renters-inactive = Inactive
renters-assign-new = Assign New Renter
renters-start-date = Start Date
renters-end-date = End Date
renters-active-rental = Active rental
renters-no-matching = No matching users found
renters-invite-email = Invite by Email
renters-invite-desc = Send an invitation to someone who doesn't have an account yet.
renters-send-invitation = Send Invitation
renters-pending = Pending Invitations
renters-pending-badge = Pending
renters-invited-by = Invited by {$name}
renters-expires = Expires: {$date}

## Admin Properties
properties-buildings = Buildings
properties-apartments = Apartments
properties-owners-tab = Owners
properties-renters-tab-admin = Renters
properties-history-tab-admin = History
properties-address-required = Address required
properties-apt-number-required = Apartment number required
properties-building-created = Building created successfully
properties-apt-created = Apartment created successfully
properties-building-deleted = Building deleted
properties-apt-deleted = Apartment deleted
properties-owner-assigned = Owner assigned successfully
properties-owner-removed = Owner removed successfully
properties-failed-create-building = Failed to create building: {$error}
properties-failed-create-apartment = Failed to create apartment: {$error}
properties-failed-delete-building = Failed to delete building: {$error}
properties-failed-delete-apartment = Failed to delete apartment: {$error}
properties-failed-load-owners = Failed to load owners: {$error}
properties-failed-load-apartments = Failed to load apartments: {$error}
properties-failed-assign-owner = Failed to assign owner: {$error}
properties-failed-remove-owner = Failed to remove owner: {$error}
properties-invitation-processed = Invitation processed

## Meter Card List (in properties)
meters-no-meters-apartment = No meters registered for this apartment.

# ============================================================
# Announcement Components
# ============================================================

announcement-active = Active
announcement-deleted = Deleted
announcement-deleted-none = None
announcement-restore-btn = Restore
announcement-purge-btn = Purge
announcement-publish-now-btn = Publish Now
announcement-edit-btn = Edit
announcement-pin-btn = Pin
announcement-delete-btn = Delete
announcement-new-btn = New announcement
announcement-edit-label = Edit announcement
announcement-cancel-btn = Cancel
announcement-disable-comments = Disable Comments
announcement-enable-comments = Enable Comments
announcement-public-badge = Public
announcement-private-badge = Private

# ============================================================
# Admin Layout / Sidebar
# ============================================================

admin-layout-access-denied = Access denied
admin-layout-menu = Admin menu
nav-dashboard = Dashboard
nav-management = Management
nav-users = Users
nav-announcements = Announcements
nav-admin-meters = Meters

# Error Alert
error-prefix = Error:

# Meters - remaining strings
meters-empty-state-hint = Click 'Register New Meter' to add one.
meters-add-reading-btn = + Add Reading
meters-register-new-btn = + Register New Meter
meters-register-success = Meter registered successfully
meters-meter-updated-success = Meter updated successfully
breadcrumb-my-properties = My Properties
breadcrumb-apartment = Apartment #{$id}
breadcrumb-meter-details = Meter Details
breadcrumb-meters = Meters

# Calibration - time period options
calibration-7-days = 7 days
calibration-30-days = 30 days
calibration-60-days = 60 days
calibration-90-days = 90 days
calibration-1-year = 1 year
calibration-no-meters-period = No meters requiring calibration in the selected time period.
calibration-meters-count = {$count} meter(s) requiring calibration within {$days} days
calibration-action-immediate = - Immediate action required
calibration-action-schedule = - Schedule calibration soon
calibration-action-monitor = - Monitor
meters-not-set = Not set

# User Management
user-mgmt-loading = Loading users...
user-mgmt-no-users = No users found.
user-mgmt-id = ID
user-mgmt-name = Name
user-mgmt-email = Email
user-mgmt-roles = Roles
user-mgmt-add-role = Add Role
user-mgmt-select-role = -- Select Role --
user-mgmt-help = Remove a role by clicking the × on its badge. Add a role using the dropdown.

# Admin Properties
admin-properties-title = Properties

# App
page-not-found = Not found
