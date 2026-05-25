package com.cameraconnector.app.ui

import com.cameraconnector.app.core.ProjectSummary
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ProjectUiModelsTest {
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
        assertTrue(ui.canRestore)
    }

    @Test
    fun systemInboxProjectCannotBeArchived() {
        val ui = projectLifecycleUi(
            project = project(id = "project-inbox", status = "Active"),
            selected = true,
            actionsEnabled = true,
        )

        assertEquals("当前项目", ui.statusLabel)
        assertFalse(ui.canSelect)
        assertFalse(ui.canArchive)
        assertFalse(ui.canRestore)
    }

    private fun project(id: String, status: String): ProjectSummary =
        ProjectSummary(
            id = id,
            name = "Project",
            slug = "project",
            status = status,
            createdAtMs = 0,
            updatedAtMs = 0,
        )
}
