package com.cameraconnector.app.ui

import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetBurst
import com.cameraconnector.app.core.InboxAssetQuality
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.core.ReviewQueueCount
import com.cameraconnector.app.core.ReviewQueueSummary
import com.cameraconnector.app.core.StrategyProfileUi
import com.cameraconnector.app.core.StrategyWeightsUi
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
    fun selectedPhotoRefreshUsesVisibleAssetWithSameStableId() {
        val stale = inboxAsset(id = "group-1").copy(
            quality = InboxAssetQuality(
                overall = 0.52,
                analysisStatus = "ready",
                scorerVersion = "local-v1",
                primaryReason = "old",
                analyzedAtMs = 1,
            ),
        )
        val fresh = stale.copy(
            quality = stale.quality?.copy(overall = 0.91, primaryReason = "fresh", analyzedAtMs = 2),
        )

        assertEquals(
            fresh,
            refreshedSelectedPhoto(
                selectedPhoto = stale,
                visibleAssets = listOf(inboxAsset(id = "group-2"), fresh),
            ),
        )
    }

    @Test
    fun selectedPhotoRefreshClosesWhenAssetLeavesVisibleList() {
        val selected = inboxAsset(id = "group-1")

        assertNull(
            refreshedSelectedPhoto(
                selectedPhoto = selected,
                visibleAssets = listOf(inboxAsset(id = "group-2")),
            ),
        )
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
    fun projectPhotoContentDoesNotRequireRunningReceiver() {
        assertTrue(projectPhotoContentVisible(receiverRunning = false, reviewModeActive = false))
        assertTrue(projectPhotoContentVisible(receiverRunning = true, reviewModeActive = false))
        assertTrue(projectPhotoContentVisible(receiverRunning = false, reviewModeActive = true))
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

    @Test
    fun selectedStrategyProfileFallsBackToGeneralThenFirstProfile() {
        val general = strategyProfile(profileId = "general", name = "General")
        val portrait = strategyProfile(profileId = "portrait", name = "Portrait")

        assertEquals(portrait, selectedStrategyProfile(listOf(general, portrait), "portrait"))
        assertEquals(general, selectedStrategyProfile(listOf(general, portrait), "missing"))
        assertEquals(portrait, selectedStrategyProfile(listOf(portrait), "missing"))
        assertNull(selectedStrategyProfile(emptyList(), "general"))
    }

    @Test
    fun strategyWeightUpdateClampsCompositionToSupportedMaximum() {
        val updated = strategyProfile()
            .withStrategyWeight(StrategyWeightField.Composition, 0.42)

        assertEquals(0.12, updated.weights.composition, 0.0001)
    }

    @Test
    fun builtInStrategyProfileSavesAsStableCustomCopy() {
        val custom = strategyProfile(profileId = "general", name = "General", builtIn = true)
            .asSavableCustomStrategyProfile(nowMs = 1234)

        assertEquals("custom-general", custom.profileId)
        assertEquals("自定义 General", custom.name)
        assertFalse(custom.builtIn)
        assertEquals(1234, custom.updatedAtMs)
    }

    @Test
    fun customStrategyProfileKeepsIdWhenSavedAgain() {
        val custom = strategyProfile(profileId = "custom-sharp", name = "Sharp", builtIn = false)
            .asSavableCustomStrategyProfile(nowMs = 5678)

        assertEquals("custom-sharp", custom.profileId)
        assertEquals("Sharp", custom.name)
        assertFalse(custom.builtIn)
        assertEquals(5678, custom.updatedAtMs)
    }

    @Test
    fun strategyWeightDisplayUsesPercentForPositiveAndNegativeWeights() {
        assertEquals("40%", strategyWeightDisplayText(0.4))
        assertEquals("-14%", strategyWeightDisplayText(-0.14))
    }

    @Test
    fun reviewQueueEntryIsHiddenWhenProjectHasNoReviewUnits() {
        assertNull(reviewQueueSummary(totalUnits = 0).reviewQueueEntryUi())
    }

    @Test
    fun reviewQueueEntryIsHiddenWhenNoActionableReviewUnitsRemain() {
        assertNull(reviewQueueSummary(totalUnits = 6).reviewQueueEntryUi())
    }

    @Test
    fun reviewQueueEntryPrioritizesUnconfirmedBestForBrushMode() {
        val entry = reviewQueueSummary(
            totalUnits = 8,
            pendingCount = 2,
            unconfirmedBestCount = 3,
            needsReviewCount = 1,
            lowScoreCandidateCount = 4,
        ).reviewQueueEntryUi()

        requireNotNull(entry)
        assertEquals("待确认优选", entry.primaryLabel)
        assertEquals(3, entry.primaryCount)
        assertEquals("3 组待确认", entry.primaryText)
        assertEquals("连拍/单张 8 组 · 待评分 2 · 需复核 1 · 低分 4", entry.subtitle)
        assertEquals("ready", entry.recommendationState)
        assertNull(entry.analysisStatus)
    }

    @Test
    fun reviewQueueEntryFallsBackToPendingAnalysis() {
        val entry = reviewQueueSummary(totalUnits = 5, pendingCount = 4).reviewQueueEntryUi()

        requireNotNull(entry)
        assertEquals("等待分析", entry.primaryLabel)
        assertEquals(4, entry.primaryCount)
        assertEquals("pending", entry.analysisStatus)
        assertNull(entry.recommendationState)
    }

    @Test
    fun reviewQueueEntryFallsBackToNearDuplicateQueue() {
        val entry = reviewQueueSummary(totalUnits = 5, nearDuplicateCount = 2).reviewQueueEntryUi()

        requireNotNull(entry)
        assertEquals("近重复", entry.primaryLabel)
        assertEquals("2 组近重复", entry.primaryText)
        assertEquals("near_duplicates", entry.queue)
        assertNull(entry.recommendationState)
        assertNull(entry.analysisStatus)
    }

    @Test
    fun reviewQueueEntryFallsBackToUnsupportedQueue() {
        val entry = reviewQueueSummary(totalUnits = 5, unsupportedCount = 1).reviewQueueEntryUi()

        requireNotNull(entry)
        assertEquals("不支持评分", entry.primaryLabel)
        assertEquals("1 组需复核", entry.primaryText)
        assertEquals("unsupported", entry.queue)
    }

    @Test
    fun reviewQueueEntryPrefersUnsupportedWhenNeedsReviewAlsoIncludesUnsupported() {
        val entry = reviewQueueSummary(
            totalUnits = 5,
            needsReviewCount = 1,
            unsupportedCount = 1,
        ).reviewQueueEntryUi()

        requireNotNull(entry)
        assertEquals("不支持评分", entry.primaryLabel)
        assertEquals("unsupported", entry.queue)
    }

    @Test
    fun reviewQueueEntryCanSurfaceUserOverriddenQueue() {
        val entry = reviewQueueSummary(totalUnits = 5, userOverriddenCount = 2).reviewQueueEntryUi()

        requireNotNull(entry)
        assertEquals("手动调整", entry.primaryLabel)
        assertEquals("2 组已调整", entry.primaryText)
        assertEquals("user_overridden", entry.queue)
    }

    @Test
    fun reviewQueueEntriesExposeSwitchableActionQueuesInPriorityOrder() {
        val entries = reviewQueueSummary(
            totalUnits = 12,
            pendingCount = 2,
            unconfirmedBestCount = 3,
            needsReviewCount = 1,
            lowScoreCandidateCount = 4,
            nearDuplicateCount = 5,
            unsupportedCount = 6,
            userOverriddenCount = 7,
        ).reviewQueueEntriesUi()

        assertEquals(
            listOf(
                "unconfirmed_best",
                "unsupported",
                "needs_review",
                "low_score_candidates",
                "near_duplicates",
                "user_overridden",
                "pending",
            ),
            entries.map { it.queue },
        )
        assertEquals(entries.first(), entries.selectedReviewQueueEntry(selectedQueue = null))
        assertEquals("near_duplicates", entries.selectedReviewQueueEntry("near_duplicates")?.queue)
        assertEquals("unconfirmed_best", entries.selectedReviewQueueEntry("missing")?.queue)
    }

    @Test
    fun reviewQueueEntryCanApplyQueryForBrushMode() {
        val entry = reviewQueueSummary(totalUnits = 4, unconfirmedBestCount = 2).reviewQueueEntryUi()
        val query = entry?.assetQuery(selectedAccount = "camera01", strategyProfileId = "portrait")

        requireNotNull(query)
        assertEquals("camera01", query.username)
        assertEquals("ready", query.recommendationState)
        assertEquals(com.cameraconnector.app.core.PhotoSortMode.GroupBestScore, query.sort)
        assertEquals("unconfirmed_best", query.reviewQueue)
        assertEquals("portrait", query.strategyProfileId)
    }

    @Test
    fun assetListQueryAppliesScoreFilterAndPromotesBestScoreSort() {
        val query = assetListQuery(
            selectedAccount = "camera01",
            selectedFilter = InboxFilter.Raw,
            selectedSort = PhotoSortMode.LatestReceived,
            selectedScoreFilter = ScoreFilter.Excellent,
        )

        assertEquals("camera01", query.username)
        assertEquals(com.cameraconnector.app.core.InboxAssetRole.Raw, query.role)
        assertEquals(80.0, query.scoreMin ?: 0.0, 0.0001)
        assertEquals(PhotoSortMode.GroupBestScore, query.sort)
    }

    @Test
    fun projectPhotoCollectionFiltersAndSortsSelectsLocally() {
        val selectedSharp = inboxAsset(id = "selected-sharp").copy(
            username = "camera01",
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "accepted",
                bestAssetGroupId = "selected-sharp",
                bestScore = 0.91,
            ),
        )
        val selectedSoft = inboxAsset(id = "selected-soft").copy(
            username = "camera01",
            burst = InboxAssetBurst(
                burstGroupId = "burst-2",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "accepted",
                bestAssetGroupId = "selected-soft",
                bestScore = 0.62,
            ),
        )
        val otherAccount = inboxAsset(id = "selected-other").copy(username = "camera02")

        val filtered = projectPhotoCollectionAssets(
            assets = listOf(selectedSoft, otherAccount, selectedSharp),
            selectedAccount = "camera01",
            selectedFilter = InboxFilter.All,
            selectedSort = PhotoSortMode.GroupBestScore,
            selectedScoreFilter = ScoreFilter.Usable,
        )

        assertEquals(listOf("selected-sharp", "selected-soft"), filtered.map { it.id })
    }

    @Test
    fun tileSmartMetaIncludesBurstBestScoreWhenItExplainsScoreFilterMatch() {
        val asset = inboxAsset(id = "group-soft").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
            quality = InboxAssetQuality(
                overall = 0.55,
                analysisStatus = "ready",
                scorerVersion = "local-v1",
                primaryReason = null,
                analyzedAtMs = 10_000,
            ),
        )

        val meta = asset.tileSmartMeta().orEmpty()
        assertTrue(meta.contains("55"))
        assertTrue(meta.contains("93"))
        assertFalse(meta.contains("连拍"))
        assertFalse(meta.contains("2/2"))
    }

    @Test
    fun burstBadgesUseMinimalCountOnGridAndPositionInDetail() {
        val asset = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 5,
                memberRank = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )

        assertEquals("5", asset.burstCountBadgeText())
        assertEquals("2/5", asset.burstPositionBadgeText())
        assertEquals("2/5", asset.burstBadgeText())
    }

    @Test
    fun smartReasonTextLocalizesKnownScoringReasons() {
        assertEquals("技术表现均衡", smartReasonText("balanced technical score"))
        assertEquals("锐度偏低", smartReasonText("low sharpness"))
        assertEquals("高光溢出", smartReasonText("highlight clipping"))
        assertEquals("阴影过暗", smartReasonText("shadow clipping"))
        assertEquals("曝光偏弱", smartReasonText("weak exposure"))
        assertEquals("需要复核预览", smartReasonText("unsupported preview sample"))
        assertEquals("自定义原因", smartReasonText("自定义原因"))
    }

    @Test
    fun tileSmartMetaIncludesLocalizedQualityReason() {
        val asset = inboxAsset(id = "group-scored").copy(
            quality = InboxAssetQuality(
                overall = 0.82,
                analysisStatus = "ready",
                scorerVersion = "local-v1",
                primaryReason = "low sharpness",
                analyzedAtMs = 10_000,
            ),
        )

        val meta = asset.tileSmartMeta().orEmpty()

        assertTrue(meta.contains("82"))
        assertTrue(meta.contains("锐度偏低"))
        assertFalse(meta.contains("low sharpness"))
    }

    @Test
    fun recommendationStatusLabelsUseReviewVocabulary() {
        assertEquals("已精选", recommendationStatusLabel("accepted"))
        assertEquals("需要复核", recommendationStatusLabel("needs_review"))
        assertEquals("手动调整", recommendationStatusLabel("user_overridden"))
        assertEquals("更新中", recommendationStatusLabel("stale"))
    }

    @Test
    fun qualitySignalRowsExposeReadableScoreBreakdown() {
        val asset = inboxAsset(id = "group-scored").copy(
            quality = InboxAssetQuality(
                overall = 0.82,
                analysisStatus = "ready",
                scorerVersion = "local-v1",
                primaryReason = "balanced",
                analyzedAtMs = 10_000,
                sharpness = 0.73,
                exposure = 0.66,
                highlightClippingPenalty = 0.08,
                shadowClippingPenalty = 0.12,
                composition = 0.58,
                compositionConfidence = 0.71,
            ),
        )

        val rows = asset.qualitySignalRows()

        assertEquals(
            listOf("锐度", "曝光", "构图", "高光", "阴影", "构图置信"),
            rows.map { it.label },
        )
        assertEquals(listOf("73", "66", "58", "8", "12", "71"), rows.map { it.value })
    }

    @Test
    fun reviewModeSignalRowsKeepCardSummaryFocusedOnPrimarySignals() {
        val asset = inboxAsset(id = "group-scored").copy(
            quality = InboxAssetQuality(
                overall = 0.82,
                analysisStatus = "ready",
                scorerVersion = "local-v1",
                primaryReason = "balanced",
                analyzedAtMs = 10_000,
                sharpness = 0.73,
                exposure = 0.66,
                highlightClippingPenalty = 0.08,
                shadowClippingPenalty = 0.12,
                composition = 0.58,
                compositionConfidence = 0.71,
            ),
        )

        val rows = asset.reviewModeSignalRows()

        assertEquals(listOf("锐度", "曝光", "构图"), rows.map { it.label })
        assertEquals(listOf("73", "66", "58"), rows.map { it.value })
    }

    @Test
    fun reviewModeProgressCoercesIndexAndFormatsPosition() {
        assertEquals(
            ReviewModeProgressUi(currentIndex = 0, totalCount = 0, text = "0/0"),
            reviewModeProgress(currentIndex = 4, totalCount = 0),
        )
        assertEquals(
            ReviewModeProgressUi(currentIndex = 2, totalCount = 3, text = "3/3"),
            reviewModeProgress(currentIndex = 8, totalCount = 3),
        )
    }

    @Test
    fun reviewModeNavigationStaysWithinVisibleAssets() {
        assertEquals(0, previousReviewIndex(currentIndex = 0))
        assertEquals(1, previousReviewIndex(currentIndex = 2))
        assertEquals(1, nextReviewIndex(currentIndex = 0, totalCount = 2))
        assertEquals(1, nextReviewIndex(currentIndex = 1, totalCount = 2))
        assertEquals(0, nextReviewIndex(currentIndex = 0, totalCount = 0))
    }

    @Test
    fun reviewModeSummaryOpensAfterActingOnLastVisibleCard() {
        assertFalse(reviewModeShouldSummarizeAfterAction(currentIndex = 0, totalCount = 2))
        assertTrue(reviewModeShouldSummarizeAfterAction(currentIndex = 1, totalCount = 2))
        assertTrue(reviewModeShouldSummarizeAfterAction(currentIndex = 0, totalCount = 1))
        assertTrue(reviewModeShouldSummarizeAfterAction(currentIndex = 0, totalCount = 0))
    }

    @Test
    fun reviewModeDragActionClassifiesCardLikeGestures() {
        assertEquals(ReviewModeDragAction.Next, reviewModeDragAction(deltaX = -140f, deltaY = 18f))
        assertEquals(ReviewModeDragAction.Previous, reviewModeDragAction(deltaX = 140f, deltaY = 18f))
        assertEquals(
            ReviewModeDragAction.AcceptRecommendedBest,
            reviewModeDragAction(deltaX = 12f, deltaY = -140f),
        )
        assertEquals(ReviewModeDragAction.MarkNeedsReview, reviewModeDragAction(deltaX = 12f, deltaY = 140f))
        assertNull(reviewModeDragAction(deltaX = 40f, deltaY = 30f))
        assertNull(reviewModeDragAction(deltaX = 96f, deltaY = 96f, threshold = 120f))
    }

    @Test
    fun reviewModeShortcutHintsExposeOnlyUsableCardGestures() {
        val best = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )

        val enabledHints = reviewModeShortcutHints(
            asset = best,
            currentIndex = 1,
            totalCount = 3,
            actionsEnabled = true,
        )

        assertEquals(
            listOf(
                "右滑: 上一张",
                "左滑: 下一张",
                "上滑: 接受推荐",
                "下滑: 标记复核",
            ),
            enabledHints.map { "${it.gestureLabel}: ${it.actionLabel}" },
        )
        assertTrue(enabledHints.all { it.enabled })

        val alternate = best.copy(
            id = "group-alt",
            burst = best.burst?.copy(
                memberRank = 2,
                bestAssetGroupId = "group-best",
            ),
        )
        val disabledHints = reviewModeShortcutHints(
            asset = alternate,
            currentIndex = 0,
            totalCount = 1,
            actionsEnabled = false,
        )

        assertEquals(
            listOf("右滑", "左滑", "上滑", "下滑"),
            disabledHints.map { it.gestureLabel },
        )
        assertEquals(
            listOf(false, false, false, false),
            disabledHints.map { it.enabled },
        )
    }

    @Test
    fun reviewModeSessionCountsAcceptedAndNeedsReviewDecisions() {
        val session = ReviewModeSessionUi()
            .record(ReviewSessionDecision.AcceptRecommendedBest)
            .record(ReviewSessionDecision.MarkNeedsReview)
            .record(ReviewSessionDecision.AcceptRecommendedBest)

        assertEquals(3, session.processedGroupCount)
        assertEquals(2, session.acceptedRecommendationCount)
        assertEquals(1, session.markedNeedsReviewCount)
        assertTrue(session.hasActivity)
        assertEquals("本轮 3 组 · 接受 2 · 复核 1", session.compactText)
    }

    @Test
    fun reviewModeSessionTracksLatestUndoableDecision() {
        val session = ReviewModeSessionUi()
            .record(ReviewSessionDecision.AcceptRecommendedBest, burstGroupId = "burst-1")

        assertEquals("burst-1", session.undoBurstGroupId)
        assertEquals("撤销接受推荐", session.undoLabel)
    }

    @Test
    fun reviewModeSessionCountsManualBestOverridesAndCanUndo() {
        val session = ReviewModeSessionUi()
            .record(ReviewSessionDecision.OverrideRecommendedBest, burstGroupId = "burst-1")

        assertEquals(1, session.processedGroupCount)
        assertEquals(1, session.manualOverrideCount)
        assertEquals("burst-1", session.undoBurstGroupId)
        assertEquals("\u64a4\u9500\u624b\u52a8\u4f18\u9009", session.undoLabel)
        assertEquals("\u672c\u8f6e 1 \u7ec4 \u00b7 \u624b\u52a8 1", session.compactText)

        val undone = session.undoLatestDecision()

        assertEquals(0, undone.processedGroupCount)
        assertEquals(0, undone.manualOverrideCount)
        assertNull(undone.undoBurstGroupId)
        assertNull(undone.undoLabel)
    }

    @Test
    fun reviewModeSessionCountsRestoreAutomaticWithoutUndo() {
        val session = ReviewModeSessionUi()
            .record(ReviewSessionDecision.RestoreAutomaticRecommendation, burstGroupId = "burst-1")

        assertEquals(1, session.processedGroupCount)
        assertEquals(1, session.restoredAutomaticCount)
        assertNull(session.undoBurstGroupId)
        assertNull(session.undoLabel)
        assertEquals("本轮 1 组 · 恢复 1", session.compactText)
    }

    @Test
    fun reviewModeSessionCountsSkippedCardsWithoutUndo() {
        val session = ReviewModeSessionUi()
            .record(ReviewSessionDecision.SkipCurrent)

        assertEquals(1, session.processedGroupCount)
        assertEquals(1, session.skippedCount)
        assertNull(session.undoBurstGroupId)
        assertNull(session.undoLabel)
        assertEquals("本轮 1 组 · 跳过 1", session.compactText)
        assertEquals(
            "已处理 1 组 · 接受推荐 0 · 标记复核 0 · 跳过 1 · 当前队列剩余 3 · 低分候选 2",
            reviewModeSessionExitSummaryText(
                session = session,
                remainingReviewGroupCount = 3,
                lowScoreCandidateCount = 2,
            ),
        )
    }

    @Test
    fun reviewModeSessionCountsExtendedReviewDecisionsWithoutUndo() {
        val session = ReviewModeSessionUi()
            .record(ReviewSessionDecision.ClearRecommendation, burstGroupId = "burst-1")
            .record(ReviewSessionDecision.KeepAllCandidates, burstGroupId = "burst-2")
            .record(ReviewSessionDecision.HideLowScoreCandidates, burstGroupId = "burst-3")

        assertEquals(3, session.processedGroupCount)
        assertEquals(1, session.clearedRecommendationCount)
        assertEquals(1, session.keptAllCandidatesCount)
        assertEquals(1, session.hiddenLowScoreCount)
        assertNull(session.undoBurstGroupId)
        assertEquals("本轮 3 组 · 清除 1 · 保留 1 · 隐藏低分 1", session.compactText)
    }

    @Test
    fun reviewModeSessionUndoLatestDecisionRevertsCountsAndClearsUndo() {
        val session = ReviewModeSessionUi()
            .record(ReviewSessionDecision.AcceptRecommendedBest, burstGroupId = "burst-1")
            .record(ReviewSessionDecision.MarkNeedsReview, burstGroupId = "burst-2")
            .undoLatestDecision()

        assertEquals(1, session.processedGroupCount)
        assertEquals(1, session.acceptedRecommendationCount)
        assertEquals(0, session.markedNeedsReviewCount)
        assertNull(session.undoBurstGroupId)
        assertNull(session.undoLabel)
    }

    @Test
    fun reviewModeSessionHidesCompactTextBeforeAnyDecision() {
        val session = ReviewModeSessionUi()

        assertFalse(session.hasActivity)
        assertNull(session.compactText)
    }

    @Test
    fun reviewModeSessionExitSummaryIncludesRemainingAndLowScoreContext() {
        val session = ReviewModeSessionUi()
            .record(ReviewSessionDecision.AcceptRecommendedBest)
            .record(ReviewSessionDecision.MarkNeedsReview)

        assertEquals(
            "已处理 2 组 · 接受推荐 1 · 标记复核 1 · 当前队列剩余 4 · 低分候选 2",
            reviewModeSessionExitSummaryText(
                session = session,
                remainingReviewGroupCount = 4,
                lowScoreCandidateCount = 2,
            ),
        )
    }

    @Test
    fun reviewDecisionTargetsOnlyValidBurstActions() {
        val best = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )
        val alternate = inboxAsset(id = "group-alt").copy(
            burst = best.burst?.copy(bestAssetGroupId = "group-best"),
        )
        val single = inboxAsset(id = "single")

        assertEquals(
            "burst-1",
            reviewDecisionBurstGroupId(best, ReviewDecisionAction.AcceptRecommendedBest),
        )
        assertNull(reviewDecisionBurstGroupId(alternate, ReviewDecisionAction.AcceptRecommendedBest))
        assertEquals(
            "burst-1",
            reviewDecisionBurstGroupId(alternate, ReviewDecisionAction.MarkNeedsReview),
        )
        assertNull(reviewDecisionBurstGroupId(single, ReviewDecisionAction.MarkNeedsReview))
    }

    @Test
    fun restoreAutomaticTargetsOnlyUserOverriddenBurstGroups() {
        val overridden = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "user_overridden",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )
        val ready = overridden.copy(
            burst = overridden.burst?.copy(recommendationStatus = "ready"),
        )

        assertEquals(
            "burst-1",
            reviewDecisionBurstGroupId(overridden, ReviewDecisionAction.RestoreAutomaticRecommendation),
        )
        assertNull(reviewDecisionBurstGroupId(ready, ReviewDecisionAction.RestoreAutomaticRecommendation))
        assertNull(
            reviewDecisionBurstGroupId(
                inboxAsset(id = "single"),
                ReviewDecisionAction.RestoreAutomaticRecommendation,
            ),
        )
    }

    @Test
    fun manualBestOverrideTargetOnlyUsesNonBestBurstMembers() {
        val best = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )
        val alternate = inboxAsset(id = "group-alt").copy(
            burst = best.burst?.copy(memberRank = 2),
        )

        assertEquals(
            ManualBestOverrideTarget(
                burstGroupId = "burst-1",
                bestAssetGroupId = "group-alt",
            ),
            manualBestOverrideTarget(alternate),
        )
        assertNull(manualBestOverrideTarget(best))
        assertNull(manualBestOverrideTarget(inboxAsset(id = "single")))
    }

    @Test
    fun manualBurstSplitTargetRequiresBurstMemberGroupId() {
        val burstMember = inboxAsset(id = "group-member").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 3,
                memberRank = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )

        assertEquals(
            ManualBurstSplitTarget(
                burstGroupId = "burst-1",
                memberGroupId = "group-member",
            ),
            manualBurstSplitTarget(burstMember),
        )
        assertNull(manualBurstSplitTarget(inboxAsset(id = "single")))
        assertNull(manualBurstSplitTarget(burstMember.copy(id = "")))
    }

    @Test
    fun reviewModeManualSplitTargetRequiresEnabledBurstMember() {
        val burstMember = inboxAsset(id = "group-member").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 3,
                memberRank = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )

        assertEquals(
            ManualBurstSplitTarget(
                burstGroupId = "burst-1",
                memberGroupId = "group-member",
            ),
            reviewModeManualSplitTarget(burstMember, actionsEnabled = true),
        )
        assertNull(reviewModeManualSplitTarget(burstMember, actionsEnabled = false))
        assertNull(reviewModeManualSplitTarget(inboxAsset(id = "single"), actionsEnabled = true))
        assertNull(reviewModeManualSplitTarget(null, actionsEnabled = true))
    }

    @Test
    fun reviewModePrimaryActionKeepsDefaultDecisionToOneObviousChoice() {
        val recommended = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 3,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )
        val alternate = inboxAsset(id = "group-alt").copy(
            burst = recommended.burst?.copy(
                memberRank = 2,
                bestAssetGroupId = "group-best",
            ),
        )

        assertEquals(
            ReviewModePrimaryActionUi(
                action = ReviewModePrimaryAction.AcceptRecommendedBest,
                label = "接受推荐",
                enabled = true,
            ),
            reviewModePrimaryAction(recommended, actionsEnabled = true),
        )
        assertEquals(
            ReviewModePrimaryActionUi(
                action = ReviewModePrimaryAction.OverrideRecommendedBest,
                label = "设为优选",
                enabled = true,
            ),
            reviewModePrimaryAction(alternate, actionsEnabled = true),
        )
        assertNull(reviewModePrimaryAction(inboxAsset(id = "single"), actionsEnabled = true))
        assertEquals(false, reviewModePrimaryAction(recommended, actionsEnabled = false)?.enabled)
    }

    @Test
    fun projectPhotoGridItemsCollapseBurstMembersIntoOneBestCoverCard() {
        val alternate = inboxAsset(id = "group-alt").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.91,
            ),
            quality = qualityScore(0.67),
        )
        val best = inboxAsset(id = "group-best").copy(
            burst = alternate.burst?.copy(memberRank = 1),
            quality = qualityScore(0.91),
        )
        val single = inboxAsset(id = "single")

        val items = projectPhotoGridItems(listOf(alternate, single, best))

        assertEquals(listOf("burst:burst-1", "asset:single"), items.map { it.key })
        assertTrue(items.first().isBurstGroup)
        assertEquals("group-best", items.first().coverAsset.id)
        assertEquals(listOf("group-best", "group-alt"), items.first().members.map { it.id })
        assertFalse(items.last().isBurstGroup)
        assertEquals("single", items.last().coverAsset.id)
    }

    @Test
    fun manualBurstMergeTargetUsesFirstSelectedBurstAsTargetAndNextDifferentGroupAsSource() {
        val target = inboxAsset(id = "group-target").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-target",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-target",
                bestScore = 0.93,
            ),
        )
        val sameBurstMember = inboxAsset(id = "group-same").copy(
            burst = target.burst?.copy(memberRank = 2, bestAssetGroupId = "group-target"),
        )
        val source = inboxAsset(id = "group-source").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-source",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-source",
                bestScore = 0.82,
            ),
        )

        assertEquals(
            ManualBurstMergeTarget(
                targetBurstGroupId = "burst-target",
                memberGroupId = "group-source",
            ),
            manualBurstMergeTarget(listOf(target, sameBurstMember, source)),
        )
        assertNull(manualBurstMergeTarget(listOf(inboxAsset(id = "single"), source)))
        assertNull(manualBurstMergeTarget(listOf(target, sameBurstMember)))
    }

    @Test
    fun burstMemberFilmstripOrdersMembersByRankAndHighlightsCurrentAndBest() {
        val best = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 3,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.91,
            ),
            quality = qualityScore(0.91),
        )
        val alternate = inboxAsset(id = "group-alt").copy(
            burst = best.burst?.copy(memberRank = 2),
            quality = qualityScore(0.76),
        )
        val low = inboxAsset(id = "group-low").copy(
            burst = best.burst?.copy(memberRank = 3),
            quality = qualityScore(0.31),
        )

        val filmstrip = burstMemberFilmstrip(
            currentAsset = alternate,
            allProjectAssets = listOf(low, alternate, best, inboxAsset(id = "single")),
        )

        assertEquals(listOf("group-best", "group-alt", "group-low"), filmstrip.map { it.asset.id })
        assertEquals(listOf("最佳", "当前", "低分"), filmstrip.map { it.badgeText })
        assertEquals(listOf(91, 76, 31), filmstrip.map { it.scoreText?.toInt() })
    }

    @Test
    fun burstMemberFilmstripIsEmptyForSingleAssets() {
        assertTrue(
            burstMemberFilmstrip(
                currentAsset = inboxAsset(id = "single"),
                allProjectAssets = listOf(inboxAsset(id = "single")),
            ).isEmpty(),
        )
    }

    @Test
    fun adjacentBurstMemberAssetNavigatesRankedBurstMembers() {
        val first = inboxAsset(id = "group-first").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 3,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-first",
                bestScore = 0.91,
            ),
            quality = qualityScore(0.91),
        )
        val second = inboxAsset(id = "group-second").copy(
            burst = first.burst?.copy(memberRank = 2),
            quality = qualityScore(0.76),
        )
        val third = inboxAsset(id = "group-third").copy(
            burst = first.burst?.copy(memberRank = 3),
            quality = qualityScore(0.55),
        )

        val assets = listOf(third, second, first, inboxAsset(id = "single"))

        assertEquals(
            "group-second",
            adjacentBurstMemberAsset(first, assets, DetailNavigationDirection.Next)?.id,
        )
        assertEquals(
            "group-first",
            adjacentBurstMemberAsset(second, assets, DetailNavigationDirection.Previous)?.id,
        )
        assertNull(adjacentBurstMemberAsset(first, assets, DetailNavigationDirection.Previous))
        assertNull(adjacentBurstMemberAsset(third, assets, DetailNavigationDirection.Next))
    }

    @Test
    fun adjacentProjectGridAssetNavigatesAggregatedPhotoGroups() {
        val burstBest = inboxAsset(id = "burst-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "burst-best",
                bestScore = 0.91,
            ),
            quality = qualityScore(0.91),
        )
        val burstOther = inboxAsset(id = "burst-other").copy(
            burst = burstBest.burst?.copy(memberRank = 2),
            quality = qualityScore(0.66),
        )
        val single = inboxAsset(id = "single")
        val nextBurstBest = inboxAsset(id = "next-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-2",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "next-best",
                bestScore = 0.82,
            ),
            quality = qualityScore(0.82),
        )
        val visibleAssets = listOf(burstOther, burstBest, single, nextBurstBest)

        assertEquals(
            "single",
            adjacentProjectGridAsset(burstOther, visibleAssets, DetailNavigationDirection.Next)?.id,
        )
        assertEquals(
            "burst-best",
            adjacentProjectGridAsset(single, visibleAssets, DetailNavigationDirection.Previous)?.id,
        )
        assertEquals(
            "next-best",
            adjacentProjectGridAsset(single, visibleAssets, DetailNavigationDirection.Next)?.id,
        )
        assertNull(adjacentProjectGridAsset(burstBest, visibleAssets, DetailNavigationDirection.Previous))
    }

    @Test
    fun burstComparisonItemsPrioritizeCurrentBestAndHighestScoredAlternative() {
        val best = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 4,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.91,
            ),
            quality = qualityScore(0.91),
        )
        val current = inboxAsset(id = "group-current").copy(
            burst = best.burst?.copy(memberRank = 3),
            quality = qualityScore(0.55),
        )
        val highAlternative = inboxAsset(id = "group-alt").copy(
            burst = best.burst?.copy(memberRank = 2),
            quality = qualityScore(0.84),
        )
        val low = inboxAsset(id = "group-low").copy(
            burst = best.burst?.copy(memberRank = 4),
            quality = qualityScore(0.31),
        )

        val comparisonItems = burstComparisonItems(
            currentAsset = current,
            allProjectAssets = listOf(low, highAlternative, current, best),
        )

        assertEquals(
            listOf("group-current", "group-best", "group-alt"),
            comparisonItems.map { it.asset.id },
        )
        assertEquals(listOf("当前", "最佳", "备选"), comparisonItems.map { it.badgeText })
    }

    @Test
    fun photoDetailDecisionUiEnablesActionsForRecommendedBest() {
        val best = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )

        val decision = photoDetailDecisionUi(best, actionsEnabled = true)

        assertEquals("burst-1", decision.acceptRecommendedBestBurstGroupId)
        assertEquals("burst-1", decision.markNeedsReviewBurstGroupId)
        assertTrue(decision.acceptRecommendedBestEnabled)
        assertTrue(decision.markNeedsReviewEnabled)
        assertTrue(decision.hasAnyAction)
        assertNull(decision.disabledReason)
    }

    @Test
    fun photoDetailDecisionUiKeepsReviewActionForNonBestBurstMember() {
        val alternate = inboxAsset(id = "group-alt").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )

        val decision = photoDetailDecisionUi(alternate, actionsEnabled = true)

        assertNull(decision.acceptRecommendedBestBurstGroupId)
        assertEquals("burst-1", decision.markNeedsReviewBurstGroupId)
        assertFalse(decision.acceptRecommendedBestEnabled)
        assertTrue(decision.markNeedsReviewEnabled)
        assertEquals("当前照片不是推荐优选", decision.disabledReason)
    }

    @Test
    fun photoDetailDecisionUiOffersManualBestOverrideForNonBestBurstMember() {
        val alternate = inboxAsset(id = "group-alt").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )

        val decision = photoDetailDecisionUi(alternate, actionsEnabled = true)

        assertEquals(
            ManualBestOverrideTarget(
                burstGroupId = "burst-1",
                bestAssetGroupId = "group-alt",
            ),
            decision.overrideRecommendedBestTarget,
        )
        assertTrue(decision.overrideRecommendedBestEnabled)
        assertTrue(decision.hasAnyAction)
    }

    @Test
    fun photoDetailDecisionUiEnablesRestoreAutomaticForUserOverriddenBurst() {
        val overridden = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "user_overridden",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )

        val decision = photoDetailDecisionUi(overridden, actionsEnabled = true)

        assertEquals("burst-1", decision.restoreAutomaticBurstGroupId)
        assertTrue(decision.restoreAutomaticEnabled)
        assertTrue(decision.hasAnyAction)
        assertNull(decision.disabledReason)
    }

    @Test
    fun photoDetailDecisionUiDisablesActionsWhileAnotherActionRuns() {
        val best = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                memberRank = 1,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )

        val decision = photoDetailDecisionUi(best, actionsEnabled = false)

        assertEquals("burst-1", decision.acceptRecommendedBestBurstGroupId)
        assertEquals("burst-1", decision.markNeedsReviewBurstGroupId)
        assertFalse(decision.acceptRecommendedBestEnabled)
        assertFalse(decision.markNeedsReviewEnabled)
        assertEquals("正在处理上一项操作", decision.disabledReason)
    }

    @Test
    fun photoDetailDecisionUiHidesActionsForSingleAssets() {
        val decision = photoDetailDecisionUi(inboxAsset(id = "single"), actionsEnabled = true)

        assertFalse(decision.hasAnyAction)
        assertNull(decision.acceptRecommendedBestBurstGroupId)
        assertNull(decision.markNeedsReviewBurstGroupId)
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

    private fun qualityScore(overall: Double): InboxAssetQuality =
        InboxAssetQuality(
            overall = overall,
            analysisStatus = "ready",
            scorerVersion = "local-v1",
            primaryReason = null,
            analyzedAtMs = 0,
        )

    private fun strategyProfile(
        profileId: String = "general",
        name: String = "General",
        builtIn: Boolean = true,
    ): StrategyProfileUi =
        StrategyProfileUi(
            profileId = profileId,
            name = name,
            builtIn = builtIn,
            strategyVersion = "strategy-v1",
            burstWindowMs = 1200,
            minGroupSize = 2,
            weights = StrategyWeightsUi(
                sharpness = 0.40,
                exposure = 0.22,
                composition = 0.12,
                highlightClippingPenalty = -0.14,
                shadowClippingPenalty = -0.08,
                diversity = 0.04,
            ),
            rejectIfSharpnessBelow = 0.25,
            flagIfOverallBelow = 0.40,
            nearDuplicateSimilarityAbove = 0.92,
            autoHideLowScore = false,
            llmEnabled = false,
        )

    private fun reviewQueueSummary(
        totalUnits: Int,
        pendingCount: Int = 0,
        unconfirmedBestCount: Int = 0,
        needsReviewCount: Int = 0,
        lowScoreCandidateCount: Int = 0,
        nearDuplicateCount: Int = 0,
        unsupportedCount: Int = 0,
        userOverriddenCount: Int = 0,
    ): ReviewQueueSummary =
        ReviewQueueSummary(
            projectId = "project-1",
            strategyProfileId = "general",
            totalUnits = totalUnits,
            pendingCount = pendingCount,
            unconfirmedBestCount = unconfirmedBestCount,
            needsReviewCount = needsReviewCount,
            lowScoreCandidateCount = lowScoreCandidateCount,
            nearDuplicateCount = nearDuplicateCount,
            unsupportedCount = unsupportedCount,
            userOverriddenCount = userOverriddenCount,
            queues = listOf(
                ReviewQueueCount("pending", pendingCount),
                ReviewQueueCount("unconfirmed_best", unconfirmedBestCount),
                ReviewQueueCount("needs_review", needsReviewCount),
                ReviewQueueCount("low_score_candidates", lowScoreCandidateCount),
                ReviewQueueCount("near_duplicates", nearDuplicateCount),
                ReviewQueueCount("unsupported", unsupportedCount),
                ReviewQueueCount("user_overridden", userOverriddenCount),
            ),
        )
}
