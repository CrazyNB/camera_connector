package com.cameraconnector.app.ui

import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ReceiverState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ProjectUiModelsTest {
    @Test
    fun globalDestinationsMatchFigmaTopLevelOrder() {
        assertEquals(
            listOf("项目", "账号", "设置"),
            GlobalDestination.entries.map { it.label },
        )
    }

    @Test
    fun projectDestinationsMatchProjectWorkspaceOrder() {
        assertEquals(
            listOf(ProjectDestination.Photos.label),
            ProjectDestination.entries.map { it.label },
        )
    }

    @Test
    fun projectWorkspaceDefaultsToPhotos() {
        assertEquals(ProjectDestination.Photos, defaultProjectDestination())
    }

    @Test
    fun projectAssetScreenUsesPhotoTitle() {
        assertEquals("项目照片", ProjectDestination.Photos.assetScreenTitle())
    }

    @Test
    fun assetSelectionIdPrefersGroupIdAndFallsBackToPath() {
        val grouped = inboxAsset(id = "group-1", displayPath = "DCIM/IMG_1001.JPG")
        val ungrouped = inboxAsset(id = "", displayPath = "DCIM/IMG_1002.JPG")

        assertEquals("group-1", grouped.assetSelectionId())
        assertEquals("DCIM/IMG_1002.JPG", ungrouped.assetSelectionId())
    }

    @Test
    fun togglingAssetSelectionAddsAndRemovesStableId() {
        val first = inboxAsset(id = "group-1")
        val second = inboxAsset(id = "group-2")

        val selected = toggleAssetSelection(
            selectedIds = toggleAssetSelection(emptyList(), first),
            asset = second,
        )

        assertEquals(listOf("group-1", "group-2"), selected)
        assertEquals(listOf("group-2"), toggleAssetSelection(selected, first))
    }

    @Test
    fun selectedAssetsResolveInVisibleAssetOrder() {
        val assets = listOf(
            inboxAsset(id = "group-1"),
            inboxAsset(id = "group-2"),
            inboxAsset(id = "group-3"),
        )

        val selected = selectedAssetsFromIds(assets, listOf("group-3", "group-1"))

        assertEquals(listOf("group-1", "group-3"), selected.map { it.id })
    }

    @Test
    fun emptyAssetSelectionDisablesSelectionMode() {
        assertFalse(isAssetSelectionMode(emptyList()))
        assertTrue(isAssetSelectionMode(listOf("group-1")))
    }

    @Test
    fun activeRegularProjectCanBeSelectedAndArchived() {
        val ui = projectLifecycleUi(
            project = project(id = "project-client", status = "Active"),
            selected = false,
            actionsEnabled = true,
        )

        assertEquals("活跃", ui.statusLabel)
        assertTrue(ui.canSelect)
        assertTrue(ui.canArchive)
        assertTrue(ui.canRename)
        assertFalse(ui.canRestore)
    }

    @Test
    fun archivedProjectCanOnlyBeRestored() {
        val ui = projectLifecycleUi(
            project = project(id = "project-client", status = "Archived"),
            selected = false,
            actionsEnabled = true,
        )

        assertEquals("已归档", ui.statusLabel)
        assertFalse(ui.canSelect)
        assertFalse(ui.canArchive)
        assertTrue(ui.canRename)
        assertTrue(ui.canRestore)
    }

    @Test
    fun selectedActiveProjectCanStillExposeLifecycleActions() {
        val ui = projectLifecycleUi(
            project = project(id = "project-client", status = "Active"),
            selected = true,
            actionsEnabled = true,
        )

        assertEquals("当前项目", ui.statusLabel)
        assertFalse(ui.canSelect)
        assertTrue(ui.canArchive)
        assertTrue(ui.canRename)
        assertFalse(ui.canRestore)
    }

    @Test
    fun groupMoveTargetsOnlyIncludeOtherActiveRegularProjects() {
        val state = ProjectState(
            projects = listOf(
                project(id = "project-active", status = "Active"),
                project(id = "project-target", status = "Active"),
                project(id = "project-archived", status = "Archived"),
                project(id = "project-extra", status = "Active"),
                project(
                    id = "project-policy-blocked",
                    status = "Active",
                    canAcceptMovedGroups = false,
                ),
            ),
            activeProjectId = "project-active",
        )

        val targets = state.groupMoveTargets(sourceProjectId = "project-active")

        assertEquals(listOf("project-target", "project-extra"), targets.map { it.id })
    }

    @Test
    fun groupMoveTargetsAreEmptyWithoutSourceProject() {
        val state = ProjectState(
            projects = listOf(project(id = "project-target", status = "Active")),
            activeProjectId = null,
        )

        assertEquals(emptyList<ProjectSummary>(), state.groupMoveTargets(sourceProjectId = null))
    }

    @Test
    fun activeProjectSummaryDoesNotFallbackToFirstProject() {
        val state = ProjectState(
            projects = listOf(project(id = "project-client", status = "Active")),
            activeProjectId = null,
        )

        assertNull(state.activeProjectSummary())
    }

    @Test
    fun activeProjectSummaryRequiresMatchingProjectId() {
        val state = ProjectState(
            projects = listOf(project(id = "project-client", status = "Active")),
            activeProjectId = "project-missing",
        )

        assertNull(state.activeProjectSummary())
    }

    @Test
    fun activeProjectSummaryReturnsSelectedProject() {
        val selected = project(id = "project-client", status = "Active")
        val state = ProjectState(
            projects = listOf(project(id = "project-other", status = "Active"), selected),
            activeProjectId = selected.id,
        )

        assertEquals(selected, state.activeProjectSummary())
    }

    @Test
    fun stoppedReceiverRequiresConfiguredAccountBeforeStart() {
        assertEquals(
            ReceiverStartBlockReason.MissingAccount,
            receiverStartBlockReason(
                running = false,
                actionsEnabled = true,
                notificationPermissionGranted = true,
                accountCount = 0,
            ),
        )
    }

    @Test
    fun runningReceiverCanAlwaysExposeStopAction() {
        assertNull(
            receiverStartBlockReason(
                running = true,
                actionsEnabled = true,
                notificationPermissionGranted = false,
                accountCount = 0,
            ),
        )
    }

    @Test
    fun stoppedReceiverRequiresNotificationPermissionAfterAccountExists() {
        assertEquals(
            ReceiverStartBlockReason.MissingNotificationPermission,
            receiverStartBlockReason(
                running = false,
                actionsEnabled = true,
                notificationPermissionGranted = false,
                accountCount = 1,
            ),
        )
    }

    @Test
    fun receiverEndpointLabelFallsBackToDefaultCameraConnectAddress() {
        val label = receiverEndpointLabel(
            ReceiverState(
                running = false,
                phase = "Stopped",
                protocol = "FTP",
                authMode = "Accounts",
                accountCount = 1,
                host = "0.0.0.0",
                port = 2121,
                outputLabel = "应用私有目录",
                message = null,
            ),
            connectHost = null,
        )

        assertEquals("FTP 192.168.50.1:2121", label)
    }

    @Test
    fun receiverEndpointLabelShowsResolvedCameraConnectAddress() {
        val label = receiverEndpointLabel(
            ReceiverState(
                running = false,
                phase = "Stopped",
                protocol = "FTP",
                authMode = "Accounts",
                accountCount = 1,
                host = "0.0.0.0",
                port = 2121,
                outputLabel = "应用私有目录",
                message = null,
            ),
            connectHost = "192.168.43.1",
        )

        assertEquals("FTP 192.168.43.1:2121", label)
    }

    @Test
    fun normalizeCameraConnectHostKeepsBindAllOutOfCameraAddress() {
        assertEquals("192.168.50.1", normalizeCameraConnectHost("0.0.0.0"))
        assertEquals("192.168.50.1", normalizeCameraConnectHost(""))
        assertEquals("192.168.43.1", normalizeCameraConnectHost(" 192.168.43.1 "))
    }

    private fun project(
        id: String,
        status: String,
        canAcceptMovedGroups: Boolean = status.equals("Active", ignoreCase = true),
    ): ProjectSummary =
        ProjectSummary(
            id = id,
            name = "Project",
            slug = "project",
            status = status,
            createdAtMs = 0,
            updatedAtMs = 0,
            canBeActiveProject = status.equals("Active", ignoreCase = true),
            canArchive = status.equals("Active", ignoreCase = true),
            canRename = true,
            canRestore = status.equals("Archived", ignoreCase = true),
            canAcceptMovedGroups = canAcceptMovedGroups,
        )

    private fun inboxAsset(
        id: String,
        displayPath: String = "$id.JPG",
    ): InboxAsset =
        InboxAsset(
            id = id,
            groupKey = id,
            displayPath = displayPath,
            format = "Jpeg",
            receivedAt = "0",
        )
}
