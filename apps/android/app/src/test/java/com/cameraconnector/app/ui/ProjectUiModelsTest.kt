package com.cameraconnector.app.ui

import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.ProjectState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
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
    fun systemInboxProjectCannotBeArchived() {
        val ui = projectLifecycleUi(
            project = project(id = "project-inbox", status = "Active"),
            selected = true,
            actionsEnabled = true,
        )

        assertEquals("当前项目", ui.statusLabel)
        assertFalse(ui.canSelect)
        assertFalse(ui.canArchive)
        assertFalse(ui.canRename)
        assertFalse(ui.canRestore)
    }

    @Test
    fun groupMoveTargetsOnlyIncludeOtherActiveRegularProjects() {
        val state = ProjectState(
            projects = listOf(
                project(id = "project-active", status = "Active"),
                project(id = "project-target", status = "Active"),
                project(id = "project-archived", status = "Archived"),
                project(id = "project-inbox", status = "Active"),
            ),
            activeProjectId = "project-active",
        )

        val targets = state.groupMoveTargets(sourceProjectId = "project-active")

        assertEquals(listOf("project-target"), targets.map { it.id })
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
