# Android LAN Share Selection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Android-first LAN photo sharing where one tokenized local link lets one guest mark shared project photos with `favorite`, `marked`, or `reject`.

**Architecture:** Core owns share sessions, token validation, saved asset queries, and guest marks. FFI exposes those operations to Android. Android owns the local HTTP server, image streaming, and UI controls for starting/stopping a share and showing guest result badges.

**Tech Stack:** Rust core with rusqlite/serde_json, Rust JNI FFI envelope, Android Kotlin/Compose, coroutine-backed `ServerSocket` HTTP adapter, existing Android preview bitmap loader.

---

## File Structure

- Create `core/src/lan_share.rs`: share session structs, `GuestMark` enum, validation, token generation helpers, and query serialization helpers.
- Modify `core/src/lib.rs`: re-export LAN share types.
- Modify `core/src/storage/mod.rs`: SQLite schema, store methods for sessions and guest marks, and asset read-model decoration.
- Modify `core/src/service.rs`: app-facing share methods that combine store operations with existing `AssetGroupQuery` behavior.
- Modify `core/tests/storage_store_tests.rs`: storage-level session and mark tests.
- Modify `core/tests/service_tests.rs`: service-level share asset query and guest mark isolation tests.
- Modify `core-ffi/src/lib.rs`: mobile JSON structs and exported methods for creating/stopping shares, querying share assets, and setting guest marks.
- Modify `core-ffi/tests/mobile_core_tests.rs`: JSON round-trip tests.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`: Android models and gateway methods.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeMobileCore.kt`: JNI wrappers and JSON helpers.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`: map `guest_mark` onto `ProjectAsset`.
- Modify `apps/android/app/src/test/java/com/cameraconnector/app/core/NativeDashboardMappingTest.kt`: Android mapping tests.
- Create `apps/android/app/src/main/java/com/cameraconnector/app/share/LanShareHttpServer.kt`: minimal local HTTP server and router.
- Create `apps/android/app/src/test/java/com/cameraconnector/app/share/LanShareHttpServerTest.kt`: route and guest mark tests using fake dependencies.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectUiModels.kt`: guest mark UI helpers.
- Modify `apps/android/app/src/test/java/com/cameraconnector/app/ui/ProjectUiModelsTest.kt`: badge/no-badge tests.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/ui/AssetUiModels.kt`: include guest result badge on tiles.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectAssetsScreen.kt`: start/stop/share URL UI.

---

### Task 1: Core LAN Share Types

**Files:**
- Create: `core/src/lan_share.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/service_tests.rs`

- [ ] **Step 1: Write the failing enum validation test**

Add this test to `core/tests/service_tests.rs`:

```rust
use camera_connector_core::GuestMark;

#[test]
fn guest_mark_accepts_only_share_selection_values() {
    assert_eq!(GuestMark::from_wire("favorite").unwrap(), GuestMark::Favorite);
    assert_eq!(GuestMark::from_wire("marked").unwrap(), GuestMark::Marked);
    assert_eq!(GuestMark::from_wire("reject").unwrap(), GuestMark::Reject);
    assert!(GuestMark::from_wire("delete").is_none());
    assert!(GuestMark::from_wire("").is_none());
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p camera_connector_core guest_mark_accepts_only_share_selection_values`

Expected: FAIL because `GuestMark` does not exist.

- [ ] **Step 3: Implement minimal types**

Create `core/src/lan_share.rs` with:

```rust
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use crate::AssetGroupQuery;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuestMark {
    Favorite,
    Marked,
    Reject,
}

impl GuestMark {
    pub fn from_wire(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "favorite" => Some(Self::Favorite),
            "marked" => Some(Self::Marked),
            "reject" => Some(Self::Reject),
            _ => None,
        }
    }

    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Favorite => "favorite",
            Self::Marked => "marked",
            Self::Reject => "reject",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanShareSession {
    pub share_id: String,
    pub project_id: String,
    pub token: String,
    pub query: AssetGroupQuery,
    pub title: Option<String>,
    pub active: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub stopped_at_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanShareGuestMark {
    pub share_id: String,
    pub project_id: String,
    pub asset_group_id: String,
    pub guest_mark: GuestMark,
    pub updated_at_ms: i64,
}

pub fn generate_lan_share_token() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
```

Modify `core/src/lib.rs`:

```rust
pub mod lan_share;

pub use lan_share::{
    generate_lan_share_token, GuestMark, LanShareGuestMark, LanShareSession,
};
```

- [ ] **Step 4: Verify GREEN**

Run: `cargo test -p camera_connector_core guest_mark_accepts_only_share_selection_values`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add core/src/lan_share.rs core/src/lib.rs core/tests/service_tests.rs
git commit -m "feat(core): add lan share types"
```

---

### Task 2: Core Storage For Sessions And Guest Marks

**Files:**
- Modify: `core/src/storage/mod.rs`
- Test: `core/tests/storage_store_tests.rs`

- [ ] **Step 1: Write failing storage tests**

Add tests to `core/tests/storage_store_tests.rs`:

```rust
use camera_connector_core::{AssetGroupQuery, AssetGroupSort, GuestMark};

#[test]
fn sqlite_store_creates_lan_share_session_with_query() {
    let temp = tempfile::tempdir().unwrap();
    let store = test_store(temp.path());
    let project = store.create_project("LAN Share").unwrap();

    let session = store
        .create_lan_share_session(
            &project.project_id,
            AssetGroupQuery {
                collection: Some("favorites".to_string()),
                sort: AssetGroupSort::ModelScore,
                ..AssetGroupQuery::default()
            },
            Some("Client selects".to_string()),
            1_000,
        )
        .unwrap();

    assert_eq!(session.project_id, project.project_id);
    assert_eq!(session.query.collection.as_deref(), Some("favorites"));
    assert_eq!(session.query.sort, AssetGroupSort::ModelScore);
    assert!(session.active);
    assert_eq!(session.token.len(), 32);
}

#[test]
fn sqlite_store_sets_and_clears_lan_share_guest_mark_without_user_marks() {
    let temp = tempfile::tempdir().unwrap();
    let store = test_store(temp.path());
    let project = store.create_project("Guest Marks").unwrap();
    let group_id = seed_project_jpeg_group(&store, &project.project_id, "IMG_9001.JPG");
    let session = store
        .create_lan_share_session(&project.project_id, AssetGroupQuery::default(), None, 1_000)
        .unwrap();

    let mark = store
        .set_lan_share_guest_mark(
            &session.share_id,
            &project.project_id,
            &group_id,
            Some(GuestMark::Reject),
            2_000,
        )
        .unwrap();
    assert_eq!(mark.unwrap().guest_mark, GuestMark::Reject);

    let page = store
        .project_asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .unwrap();
    assert_eq!(page.groups[0].guest_mark, Some(GuestMark::Reject));
    assert!(!page.groups[0].user_marks.favorite);
    assert!(!page.groups[0].user_marks.marked);

    let cleared = store
        .set_lan_share_guest_mark(&session.share_id, &project.project_id, &group_id, None, 3_000)
        .unwrap();
    assert!(cleared.is_none());
}
```

Use existing helper style in the file; if `test_store` or asset seed helpers have different local names, reuse the closest existing helpers in that test file.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p camera_connector_core sqlite_store_creates_lan_share_session_with_query sqlite_store_sets_and_clears_lan_share_guest_mark_without_user_marks`

Expected: FAIL because storage methods and `guest_mark` read-model field do not exist.

- [ ] **Step 3: Implement schema and store methods**

In `core/src/model/asset_group.rs`, add to `ReceivedAssetGroup`:

```rust
pub guest_mark: Option<GuestMark>,
```

Initialize it to `None` in `group_received_assets`.

In `core/src/storage/mod.rs` schema setup, add:

```sql
CREATE TABLE IF NOT EXISTS lan_share_sessions (
    share_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    query_json TEXT NOT NULL,
    title TEXT,
    active INTEGER NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    stopped_at_ms INTEGER
);

CREATE TABLE IF NOT EXISTS lan_share_guest_marks (
    share_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    asset_group_id TEXT NOT NULL,
    guest_mark TEXT NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    PRIMARY KEY (share_id, asset_group_id)
);

CREATE INDEX IF NOT EXISTS idx_lan_share_sessions_token ON lan_share_sessions(token);
CREATE INDEX IF NOT EXISTS idx_lan_share_guest_marks_project ON lan_share_guest_marks(project_id, asset_group_id);
```

Add store methods:

```rust
pub fn create_lan_share_session(
    &self,
    project_id: &str,
    query: AssetGroupQuery,
    title: Option<String>,
    now_ms: i64,
) -> Result<LanShareSession>

pub fn lan_share_session_by_token(&self, token: &str) -> Result<Option<LanShareSession>>

pub fn stop_lan_share_session(&self, share_id: &str, now_ms: i64) -> Result<Option<LanShareSession>>

pub fn set_lan_share_guest_mark(
    &self,
    share_id: &str,
    project_id: &str,
    asset_group_id: &str,
    guest_mark: Option<GuestMark>,
    now_ms: i64,
) -> Result<Option<LanShareGuestMark>>
```

Decorate `ReceivedAssetGroup.guest_mark` while building project asset pages by joining the latest active share guest mark for that project/group. Keep photographer `user_marks` unchanged.

- [ ] **Step 4: Verify GREEN**

Run: `cargo test -p camera_connector_core lan_share`

Expected: PASS for the new storage tests.

- [ ] **Step 5: Commit**

```powershell
git add core/src/model/asset_group.rs core/src/storage/mod.rs core/tests/storage_store_tests.rs
git commit -m "feat(core): persist lan share guest marks"
```

---

### Task 3: Core Service API

**Files:**
- Modify: `core/src/service.rs`
- Test: `core/tests/service_tests.rs`

- [ ] **Step 1: Write failing service tests**

Add:

```rust
use camera_connector_core::{AssetGroupQuery, AssetGroupSort, GuestMark};

#[test]
fn service_lan_share_asset_page_uses_saved_query() {
    let temp = tempfile::tempdir().unwrap();
    let service = CameraConnectorService::new(Some(temp.path().join("config.json")));
    let project = service.create_project("LAN Query").unwrap();
    let keep = record_jpeg_group(&service, &project.project_id, "KEEP_0001.JPG");
    let skip = record_jpeg_group(&service, &project.project_id, "SKIP_0001.JPG");
    service
        .set_asset_group_user_marks(&project.project_id, &keep, Some(true), None)
        .unwrap();

    let session = service
        .create_lan_share_session(
            &project.project_id,
            AssetGroupQuery {
                favorite: Some(true),
                sort: AssetGroupSort::Filename,
                ..AssetGroupQuery::default()
            },
            Some("Favorites".to_string()),
        )
        .unwrap();

    let page = service.lan_share_asset_group_page(&session.token, 0, 25).unwrap();

    assert_eq!(page.total_groups, 1);
    assert_eq!(page.groups[0].group_id.as_deref(), Some(keep.as_str()));
    assert_ne!(page.groups[0].group_id.as_deref(), Some(skip.as_str()));
}

#[test]
fn service_guest_reject_mark_does_not_delete_or_mutate_user_marks() {
    let temp = tempfile::tempdir().unwrap();
    let service = CameraConnectorService::new(Some(temp.path().join("config.json")));
    let project = service.create_project("LAN Reject").unwrap();
    let group_id = record_jpeg_group(&service, &project.project_id, "IMG_4040.JPG");
    let session = service
        .create_lan_share_session(&project.project_id, AssetGroupQuery::default(), None)
        .unwrap();

    service
        .set_lan_share_guest_mark(&session.token, &group_id, Some(GuestMark::Reject))
        .unwrap();

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .unwrap();
    assert_eq!(page.total_groups, 1);
    assert_eq!(page.groups[0].guest_mark, Some(GuestMark::Reject));
    assert!(!page.groups[0].user_marks.favorite);
    assert!(!page.groups[0].user_marks.marked);
}
```

Use local helper names already present in `service_tests.rs`; create small helpers if needed.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p camera_connector_core service_lan_share_asset_page_uses_saved_query service_guest_reject_mark_does_not_delete_or_mutate_user_marks`

Expected: FAIL because service methods do not exist.

- [ ] **Step 3: Implement service methods**

Add to `impl CameraConnectorService`:

```rust
pub fn create_lan_share_session(
    &self,
    project_id: &str,
    query: AssetGroupQuery,
    title: Option<String>,
) -> Result<LanShareSession>

pub fn stop_lan_share_session(&self, share_id: &str) -> Result<Option<LanShareSession>>

pub fn lan_share_asset_group_page(
    &self,
    token: &str,
    offset: usize,
    limit: usize,
) -> Result<AssetGroupPage>

pub fn set_lan_share_guest_mark(
    &self,
    token: &str,
    asset_group_id: &str,
    guest_mark: Option<GuestMark>,
) -> Result<Option<LanShareGuestMark>>
```

Use existing time helper style in `service.rs`. Token lookup must require `active == true`; inactive or unknown token should return a normal error.

- [ ] **Step 4: Verify GREEN**

Run: `cargo test -p camera_connector_core lan_share`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add core/src/service.rs core/tests/service_tests.rs
git commit -m "feat(core): expose lan share service api"
```

---

### Task 4: FFI JSON Surface

**Files:**
- Modify: `core-ffi/src/lib.rs`
- Modify: `core-ffi/tests/mobile_core_tests.rs`

- [ ] **Step 1: Write failing FFI tests**

Add:

```rust
#[test]
fn mobile_core_creates_lan_share_and_sets_guest_mark_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile LAN").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:lan", "DCIM/100/IMG_7001.JPG", 10),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let session: Value = serde_json::from_str(
        &core
            .create_lan_share_session_json(
                project.project_id.clone(),
                r#"{"collection":"all","sort":"latest_received"}"#.to_string(),
                "Client link".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(session["project_id"], project.project_id);
    let token = session["token"].as_str().unwrap();

    let page: Value = serde_json::from_str(
        &core
            .lan_share_asset_group_page_json(token.to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let group_id = page["groups"][0]["group_id"].as_str().unwrap();

    let mark: Value = serde_json::from_str(
        &core
            .set_lan_share_guest_mark_json(
                token.to_string(),
                group_id.to_string(),
                r#"{"guest_mark":"reject"}"#.to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(mark["guest_mark"], "reject");
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test -p camera_connector_ffi mobile_core_creates_lan_share_and_sets_guest_mark_json`

Expected: FAIL because FFI methods do not exist.

- [ ] **Step 3: Implement FFI methods**

In `MobileCore`, add Rust methods:

```rust
pub fn create_lan_share_session_json(
    &self,
    project_id: String,
    query_json: String,
    title: String,
) -> MobileCoreResult<String>

pub fn stop_lan_share_session_json(&self, share_id: String) -> MobileCoreResult<String>

pub fn lan_share_asset_group_page_json(
    &self,
    token: String,
    offset: usize,
    limit: usize,
) -> MobileCoreResult<String>

pub fn set_lan_share_guest_mark_json(
    &self,
    token: String,
    asset_group_id: String,
    patch_json: String,
) -> MobileCoreResult<String>
```

Add matching `extern "system"` JNI functions following the existing envelope style. Serialize `GuestMark` as wire strings and clear when JSON has `guest_mark: null`.

- [ ] **Step 4: Verify GREEN**

Run: `cargo test -p camera_connector_ffi lan_share`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add core-ffi/src/lib.rs core-ffi/tests/mobile_core_tests.rs
git commit -m "feat(ffi): expose lan share json api"
```

---

### Task 5: Android Core Mapping

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeMobileCore.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`
- Modify: `apps/android/app/src/test/java/com/cameraconnector/app/core/NativeDashboardMappingTest.kt`

- [ ] **Step 1: Write failing mapping tests**

Add:

```kotlin
@Test
fun projectAssetsMapGuestMarkFromNativeDashboard() {
    val assets = mapProjectAssets(
        JSONObject()
            .put(
                "groups",
                org.json.JSONArray()
                    .put(
                        JSONObject()
                            .put("group_id", "group-1")
                            .put("guest_mark", "reject")
                            .put(
                                "primary",
                                JSONObject()
                                    .put("id", "asset-jpg")
                                    .put("filename", "IMG_1001.JPG")
                                    .put("format", "Jpeg")
                                    .put("received_time_ms", 10),
                            ),
                    ),
            ),
    )

    assertEquals(GuestMark.Reject, assets.single().guestMark)
}
```

- [ ] **Step 2: Verify RED**

Run: `.\gradlew.bat -p apps\android testDebugUnitTest --tests com.cameraconnector.app.core.NativeDashboardMappingTest.projectAssetsMapGuestMarkFromNativeDashboard`

Expected: FAIL because `GuestMark` and `guestMark` do not exist.

- [ ] **Step 3: Implement Android models and mapping**

In `CoreGateway.kt` add:

```kotlin
enum class GuestMark(val wireName: String) {
    Favorite("favorite"),
    Marked("marked"),
    Reject("reject"),
}

data class LanShareSessionUi(
    val shareId: String,
    val projectId: String,
    val token: String,
    val title: String?,
    val active: Boolean,
)
```

Add `val guestMark: GuestMark? = null` to `ProjectAsset`.

Extend `CoreGateway`:

```kotlin
suspend fun createLanShareSession(
    projectId: String,
    query: ProjectAssetQuery,
    title: String?,
): LanShareSessionUi
suspend fun stopLanShareSession(shareId: String): LanShareSessionUi?
suspend fun loadLanShareAssets(token: String, offset: Int = 0, limit: Int = 2_000): List<ProjectAsset>
suspend fun setLanShareGuestMark(token: String, groupId: String, guestMark: GuestMark?): GuestMark?
```

Map `guest_mark` in `NativeCoreGateway.mapProjectAssets`.

- [ ] **Step 4: Verify GREEN**

Run: `.\gradlew.bat -p apps\android testDebugUnitTest --tests com.cameraconnector.app.core.NativeDashboardMappingTest.projectAssetsMapGuestMarkFromNativeDashboard`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add apps/android/app/src/main/java/com/cameraconnector/app/core apps/android/app/src/test/java/com/cameraconnector/app/core/NativeDashboardMappingTest.kt
git commit -m "feat(android): map lan share guest marks"
```

---

### Task 6: Android LAN HTTP Server

**Files:**
- Create: `apps/android/app/src/main/java/com/cameraconnector/app/share/LanShareHttpServer.kt`
- Create: `apps/android/app/src/test/java/com/cameraconnector/app/share/LanShareHttpServerTest.kt`

- [ ] **Step 1: Write failing route tests**

Create tests that instantiate the router with fake dependencies:

```kotlin
@Test
fun guestMarkRejectRouteWritesGuestMarkWithoutDeleteCallback() = runTest {
    val gateway = RecordingShareGateway()
    val router = LanShareRouter(
        gateway = gateway,
        previewLoader = { _, _ -> ByteArray(0) },
    )

    val response = router.handle(
        LanShareRequest(
            method = "PUT",
            path = "/api/s/token-1/assets/group-1/guest-mark",
            body = """{"guest_mark":"reject"}""",
        ),
    )

    assertEquals(200, response.status)
    assertEquals(GuestMark.Reject, gateway.marks["group-1"])
    assertFalse(gateway.deleteCalled)
}
```

- [ ] **Step 2: Verify RED**

Run: `.\gradlew.bat -p apps\android testDebugUnitTest --tests com.cameraconnector.app.share.LanShareHttpServerTest`

Expected: FAIL because server/router classes do not exist.

- [ ] **Step 3: Implement minimal router and server**

Implement focused classes:

```kotlin
data class LanShareRequest(val method: String, val path: String, val body: String = "")
data class LanShareResponse(
    val status: Int,
    val contentType: String,
    val body: ByteArray,
)
```

`LanShareRouter` handles:

- `GET /s/{token}` -> HTML page.
- `GET /api/s/{token}/assets` -> JSON asset list.
- `PUT /api/s/{token}/assets/{groupId}/guest-mark` -> calls `setLanShareGuestMark`.
- `GET /api/s/{token}/preview/{groupId}` -> JPEG bytes or 404.

`LanShareHttpServer` uses `ServerSocket(0)` or a requested port, accepts sockets on `Dispatchers.IO`, parses simple HTTP/1.1 requests, and writes status/content headers plus bytes. Keep it intentionally small: no chunked uploads, no keep-alive.

- [ ] **Step 4: Verify GREEN**

Run: `.\gradlew.bat -p apps\android testDebugUnitTest --tests com.cameraconnector.app.share.LanShareHttpServerTest`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add apps/android/app/src/main/java/com/cameraconnector/app/share apps/android/app/src/test/java/com/cameraconnector/app/share
git commit -m "feat(android): add lan share http server"
```

---

### Task 7: Android UI Models And Badges

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectUiModels.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/AssetUiModels.kt`
- Modify: `apps/android/app/src/test/java/com/cameraconnector/app/ui/ProjectUiModelsTest.kt`

- [ ] **Step 1: Write failing UI model tests**

Add:

```kotlin
@Test
fun guestMarkBadgeIsHiddenWhenNoGuestMarkExists() {
    assertNull(projectAsset().copy(guestMark = null).guestMarkBadgeText())
}

@Test
fun guestMarkBadgeShowsRejectAsGuestDeleteSuggestion() {
    assertEquals("访客 删除", projectAsset().copy(guestMark = GuestMark.Reject).guestMarkBadgeText())
}
```

- [ ] **Step 2: Verify RED**

Run: `.\gradlew.bat -p apps\android testDebugUnitTest --tests com.cameraconnector.app.ui.ProjectUiModelsTest`

Expected: FAIL because `guestMarkBadgeText` does not exist.

- [ ] **Step 3: Implement UI helpers**

Add:

```kotlin
internal fun ProjectAsset.guestMarkBadgeText(): String? = when (guestMark) {
    GuestMark.Favorite -> "访客 收藏"
    GuestMark.Marked -> "访客 标记"
    GuestMark.Reject -> "访客 删除"
    null -> null
}
```

Include this badge in `tileAuxiliaryBadges()` before format badges, while preserving the existing maximum badge count behavior.

- [ ] **Step 4: Verify GREEN**

Run: `.\gradlew.bat -p apps\android testDebugUnitTest --tests com.cameraconnector.app.ui.ProjectUiModelsTest`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add apps/android/app/src/main/java/com/cameraconnector/app/ui apps/android/app/src/test/java/com/cameraconnector/app/ui/ProjectUiModelsTest.kt
git commit -m "feat(android): show guest mark badges"
```

---

### Task 8: Android Share Controls In Project Photos

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectAssetsScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/CameraConnectorApp.kt`

- [ ] **Step 1: Write failing UI behavior test if existing structure allows it**

If the existing unit tests have pure helpers for action state, add:

```kotlin
@Test
fun lanShareActionRequiresActiveProjectAndAssets() {
    assertFalse(lanShareActionUi(activeProjectId = null, assetCount = 3, running = false).enabled)
    assertFalse(lanShareActionUi(activeProjectId = "project-1", assetCount = 0, running = false).enabled)
    assertTrue(lanShareActionUi(activeProjectId = "project-1", assetCount = 3, running = false).enabled)
}
```

If no suitable pure helper exists, create `LanShareActionUi` in `ProjectUiModels.kt` and test it there before adding Compose UI.

- [ ] **Step 2: Verify RED**

Run: `.\gradlew.bat -p apps\android testDebugUnitTest --tests com.cameraconnector.app.ui.ProjectUiModelsTest`

Expected: FAIL before helper implementation.

- [ ] **Step 3: Implement Compose integration**

In `ProjectAssetsScreen`:

- Add a share button near filter controls.
- Use the current `assetQuery`, `projectState.activeProjectId`, and `filteredAssets`.
- On start, call `coreGateway.createLanShareSession(projectId, assetQuery, title)`.
- Start `LanShareHttpServer`.
- Show URL as selectable text: `http://<cameraConnectHost>:<port>/s/<token>`.
- Add stop action that stops the server and calls `stopLanShareSession`.

Keep first version activity-scoped; do not move to a new foreground service until the HTTP path is working.

- [ ] **Step 4: Verify GREEN**

Run: `.\gradlew.bat -p apps\android testDebugUnitTest`

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectAssetsScreen.kt apps/android/app/src/main/java/com/cameraconnector/app/ui/CameraConnectorApp.kt apps/android/app/src/test/java/com/cameraconnector/app/ui/ProjectUiModelsTest.kt
git commit -m "feat(android): add lan share controls"
```

---

### Task 9: End-To-End Verification

**Files:**
- Modify as needed only for fixes from verification.

- [ ] **Step 1: Run Rust tests**

Run: `cargo test`

Expected: all Rust workspace tests pass.

- [ ] **Step 2: Run Android unit tests**

Run: `.\gradlew.bat -p apps\android testDebugUnitTest`

Expected: all Android unit tests pass.

- [ ] **Step 3: Build Android debug APK with native core**

Run: `.\scripts\verify_android_build.ps1`

Expected: debug build succeeds with native core packaging.

- [ ] **Step 4: Manual LAN smoke test**

Install/run the Android app, open a project with photos, start a share, open the shown URL from another device on the same LAN, set `favorite`, `marked`, and `reject`, clear one mark, and confirm Android tiles update without changing photographer favorite/marked state.

- [ ] **Step 5: Commit verification fixes**

```powershell
git status --short
git add <only-files-changed-for-verification-fixes>
git commit -m "fix: stabilize android lan share selection"
```

Skip this commit if no verification fixes were needed.

---

## Self-Review

Spec coverage:

- Android-first LAN host is covered by Tasks 6 and 8.
- Single token and single guest operator are covered by Tasks 2, 3, 4, and 6.
- Current project plus saved query is covered by Tasks 2 and 3.
- Guest marks `favorite`, `marked`, `reject`, and clear are covered by Tasks 1, 2, 3, 4, and 6.
- `guest_mark` alongside `user_marks` is covered by Tasks 2, 5, and 7.
- Guest `reject` not deleting files is covered by Tasks 3, 6, and 7.
- Manual LAN validation is covered by Task 9.

Placeholder scan: no `TBD` or deferred implementation placeholders are intentionally left in the plan.

Type consistency: use `GuestMark` in Rust and Kotlin, wire field `guest_mark`, and Android data property `guestMark`.
