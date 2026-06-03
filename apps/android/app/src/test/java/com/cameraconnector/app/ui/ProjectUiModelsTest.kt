package com.cameraconnector.app.ui

import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetBurst
import com.cameraconnector.app.core.InboxAssetQuality
import com.cameraconnector.app.core.InboxAssetUserMarks
import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.PromptProfileUi
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.core.StrategyProfileUi
import com.cameraconnector.app.core.StrategyWeightsUi
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlinx.coroutines.runBlocking

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
        assertTrue(projectPhotoContentVisible(receiverRunning = false))
        assertTrue(projectPhotoContentVisible(receiverRunning = true))
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
    fun assetListQueryAppliesScoreFilterAndPromotesBestScoreSort() {
        val query = assetListQuery(
            selectedCollection = ProjectPhotoCollection.Favorites,
            selectedAccount = "camera01",
            selectedFilter = InboxFilter.Raw,
            selectedSort = PhotoSortMode.LatestReceived,
            selectedScoreFilter = ScoreFilter.Excellent,
        )

        assertEquals("camera01", query.username)
        assertEquals(com.cameraconnector.app.core.InboxAssetRole.Raw, query.role)
        assertEquals(true, query.favorite)
        assertEquals(null, query.marked)
        assertEquals(80.0, query.scoreMin ?: 0.0, 0.0001)
        assertEquals(PhotoSortMode.GroupBestScore, query.sort)
    }

    @Test
    fun assetListQueryMapsMarkedCollectionToUserMarkFilter() {
        val query = assetListQuery(
            selectedCollection = ProjectPhotoCollection.Marked,
            selectedAccount = null,
            selectedFilter = InboxFilter.All,
            selectedSort = PhotoSortMode.Filename,
            selectedScoreFilter = ScoreFilter.All,
        )

        assertEquals(null, query.favorite)
        assertEquals(true, query.marked)
        assertEquals(PhotoSortMode.Filename, query.sort)
    }

    @Test
    fun assetListQueryMapsModelAndRiskCollectionsToReviewQueues() {
        val modelSelects = assetListQuery(
            selectedCollection = ProjectPhotoCollection.ModelSelects,
            selectedAccount = null,
            selectedFilter = InboxFilter.All,
            selectedSort = PhotoSortMode.LatestReceived,
            selectedScoreFilter = ScoreFilter.All,
        )
        val qualityRisk = assetListQuery(
            selectedCollection = ProjectPhotoCollection.QualityRisk,
            selectedAccount = null,
            selectedFilter = InboxFilter.All,
            selectedSort = PhotoSortMode.LatestReceived,
            selectedScoreFilter = ScoreFilter.All,
        )
        val pending = assetListQuery(
            selectedCollection = ProjectPhotoCollection.PendingAnalysis,
            selectedAccount = null,
            selectedFilter = InboxFilter.All,
            selectedSort = PhotoSortMode.LatestReceived,
            selectedScoreFilter = ScoreFilter.All,
        )

        assertEquals("model_selects", modelSelects.reviewQueue)
        assertEquals("quality_risk", qualityRisk.reviewQueue)
        assertEquals("pending_analysis", pending.reviewQueue)
    }

    @Test
    fun tileSmartMetaOmitsNumericScoreFromProjectGrid() {
        val asset = inboxAsset(id = "group-soft").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
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
        assertFalse(meta.contains("55"))
        assertFalse(meta.contains("93"))
        assertFalse(meta.contains("连拍"))
        assertFalse(meta.contains("2/2"))
    }

    @Test
    fun burstBadgesUseMinimalCountOutsideDetail() {
        val asset = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 5,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )

        assertEquals("5", asset.burstCountBadgeText())
        assertEquals("5", asset.burstBadgeText())
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
            technicalGateStatus = "warn",
            quality = InboxAssetQuality(
                overall = 0.82,
                analysisStatus = "ready",
                scorerVersion = "local-v1",
                primaryReason = "low sharpness",
                analyzedAtMs = 10_000,
            ),
        )

        val meta = asset.tileSmartMeta().orEmpty()

        assertTrue(meta.contains("锐度偏低"))
        assertFalse(meta.contains("82"))
        assertFalse(meta.contains("low sharpness"))
    }

    @Test
    fun recommendationStatusLabelsUseReviewVocabulary() {
        assertEquals("已推荐", recommendationStatusLabel("accepted"))
        assertEquals("需要复核", recommendationStatusLabel("needs_review"))
        assertEquals("人工变更", recommendationStatusLabel("user_overridden"))
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
    fun manualBurstSplitTargetRequiresBurstMemberGroupId() {
        val burstMember = inboxAsset(id = "group-member").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 3,
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
    fun photoDetailBurstPositionFallsBackToMemberOrderWhenRankIsMissing() {
        val first = inboxAsset(id = "group-a").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 3,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-a",
                bestScore = 0.93,
            ),
        )
        val second = inboxAsset(id = "group-b").copy(
            burst = first.burst?.copy(bestAssetGroupId = "group-a"),
        )
        val third = inboxAsset(id = "group-c").copy(
            burst = first.burst?.copy(bestAssetGroupId = "group-a"),
        )

        assertEquals(
            "2/3",
            photoDetailBurstPositionText(
                asset = second,
                burstMembers = listOf(first, second, third),
            ),
        )
    }

    @Test
    fun projectPhotoGridItemsCollapseBurstMembersIntoOneBestCoverCard() {
        val alternate = inboxAsset(id = "group-alt").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.91,
            ),
            quality = qualityScore(0.67),
        )
        val best = inboxAsset(id = "group-best").copy(
            burst = alternate.burst,
            quality = qualityScore(0.91),
        )
        val single = inboxAsset(id = "single")

        val items = projectPhotoGridItems(listOf(alternate, single, best))

        assertEquals(listOf("burst:burst-1", "asset:single"), items.map { it.key })
        assertTrue(items.first().isBurstGroup)
        assertEquals("group-best", items.first().coverAsset.id)
        assertEquals(listOf("group-alt", "group-best"), items.first().members.map { it.id })
        assertFalse(items.last().isBurstGroup)
        assertEquals("single", items.last().coverAsset.id)
    }

    @Test
    fun manualBurstMergeTargetUsesFirstSelectedBurstAsTargetAndNextDifferentGroupAsSource() {
        val target = inboxAsset(id = "group-target").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-target",
                memberCount = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-target",
                bestScore = 0.93,
            ),
        )
        val sameBurstMember = inboxAsset(id = "group-same").copy(
            burst = target.burst?.copy(bestAssetGroupId = "group-target"),
        )
        val source = inboxAsset(id = "group-source").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-source",
                memberCount = 2,
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
    fun burstMemberFilmstripOrdersMembersByDerivedOrderAndHighlightsCurrentAndBest() {
        val best = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 3,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.91,
            ),
            quality = qualityScore(0.91),
        )
        val alternate = inboxAsset(id = "group-alt").copy(
            burst = best.burst,
            quality = qualityScore(0.76),
        )
        val low = inboxAsset(id = "group-low").copy(
            burst = best.burst,
            quality = qualityScore(0.31),
        )

        val filmstrip = burstMemberFilmstrip(
            currentAsset = alternate,
            allProjectAssets = listOf(low, alternate, best, inboxAsset(id = "single")),
        )

        assertEquals(listOf("group-alt", "group-best", "group-low"), filmstrip.map { it.asset.id })
        assertEquals(listOf("当前", "最佳", "低分"), filmstrip.map { it.badgeText })
        assertEquals(listOf(76, 91, 31), filmstrip.map { it.scoreText?.toInt() })
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
                recommendationStatus = "ready",
                bestAssetGroupId = "group-first",
                bestScore = 0.91,
            ),
            quality = qualityScore(0.91),
        )
        val second = inboxAsset(id = "group-second").copy(
            burst = first.burst,
            quality = qualityScore(0.76),
        )
        val third = inboxAsset(id = "group-third").copy(
            burst = first.burst,
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
                recommendationStatus = "ready",
                bestAssetGroupId = "burst-best",
                bestScore = 0.91,
            ),
            quality = qualityScore(0.91),
        )
        val burstOther = inboxAsset(id = "burst-other").copy(
            burst = burstBest.burst,
            quality = qualityScore(0.66),
        )
        val single = inboxAsset(id = "single")
        val nextBurstBest = inboxAsset(id = "next-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-2",
                memberCount = 2,
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
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.91,
            ),
            quality = qualityScore(0.91),
        )
        val current = inboxAsset(id = "group-current").copy(
            burst = best.burst,
            quality = qualityScore(0.55),
        )
        val highAlternative = inboxAsset(id = "group-alt").copy(
            burst = best.burst,
            quality = qualityScore(0.84),
        )
        val low = inboxAsset(id = "group-low").copy(
            burst = best.burst,
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
    fun photoDetailActionBarVisibleForUnsupportedBurstMember() {
        val raw = inboxAsset(id = "group-raw", displayPath = "group-raw.NEF").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
            quality = null,
        )
        val decision = photoDetailDecisionUi(raw, actionsEnabled = true)

        assertTrue(photoDetailActionBarVisible(decision, hasActionCallbacks = true))
        assertFalse(photoDetailActionBarVisible(decision, hasActionCallbacks = false))
        assertTrue(decision.splitBurstEnabled)
    }

    @Test
    fun photoDetailFavoriteSelectedUsesPersistedUserMarksOnly() {
        val recommended = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )
        val favorite = recommended.copy(userMarks = InboxAssetUserMarks(favorite = true))

        assertFalse(photoDetailFavoriteSelected(recommended))
        assertTrue(photoDetailFavoriteSelected(favorite))
    }

    @Test
    fun photoDetailMarkedSelectedUsesPersistedUserMarksOnly() {
        val recommended = inboxAsset(id = "group-best").copy(
            burst = InboxAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )
        val marked = recommended.copy(userMarks = InboxAssetUserMarks(marked = true))

        assertFalse(photoDetailMarkedSelected(recommended))
        assertTrue(photoDetailMarkedSelected(marked))
    }

    @Test
    fun photoDetailActionBarVisibleForSingleAssetFavoriteOnly() {
        val decision = photoDetailDecisionUi(inboxAsset(id = "single"), actionsEnabled = true)

        assertTrue(photoDetailActionBarVisible(decision, hasActionCallbacks = true))
        assertFalse(decision.hasAnyAction)
        assertFalse(decision.splitBurstEnabled)
    }

    @Test
    fun detailPageSlideOffsetUsesFullPageDirection() {
        assertEquals(320, detailPageSlideOffset(320, DetailNavigationDirection.Next, entering = true))
        assertEquals(-320, detailPageSlideOffset(320, DetailNavigationDirection.Next, entering = false))
        assertEquals(-320, detailPageSlideOffset(320, DetailNavigationDirection.Previous, entering = true))
        assertEquals(320, detailPageSlideOffset(320, DetailNavigationDirection.Previous, entering = false))
    }

    @Test
    fun projectEvaluationSettingsDefaultsModelEvaluationOff() {
        val settings = ProjectEvaluationSettingsUi(projectId = "project-client")
        val ui = projectIntelligenceSettingsUi(settings = settings, providerConfigured = true)

        assertFalse(settings.modelEvaluationEnabled)
        assertFalse(ui.modelEvaluationEnabled)
    }

    @Test
    fun promptProfileStyleTagsRenderAsCompactText() {
        val profile = PromptProfileUi(
            promptProfileId = "portrait-conservative",
            scope = "global",
            projectId = null,
            name = "Portrait Conservative",
            styleTags = listOf("portrait", "conservative"),
            sceneProfile = "portrait",
            activeVersionId = "version-1",
            builtIn = true,
            enabled = true,
        )

        assertEquals("人像 / 稳健", promptStyleTagsText(profile))
    }

    @Test
    fun noKeyStateDisablesManualProjectRecommendation() {
        val ui = manualProjectRecommendationActionUi(
            provider = ModelProviderSettingsUi(configured = false),
            settings = ProjectEvaluationSettingsUi(projectId = "project-client"),
            actionInFlight = false,
        )

        assertFalse(ui.enabled)
        assertEquals("生成项目优选", ui.ctaLabel)
        assertEquals("模型服务未配置", ui.disabledReason)
    }

    @Test
    fun noActiveProjectDisablesManualProjectRecommendation() {
        val ui = manualProjectRecommendationActionUi(
            provider = ModelProviderSettingsUi(configured = true),
            settings = ProjectEvaluationSettingsUi(projectId = ""),
            actionInFlight = false,
        )

        assertFalse(ui.enabled)
        assertEquals("请先进入项目", ui.disabledReason)
    }

    @Test
    fun automaticProjectRecommendationModeDisablesManualActionUi() {
        val ui = manualProjectRecommendationActionUi(
            provider = ModelProviderSettingsUi(configured = true),
            settings = ProjectEvaluationSettingsUi(
                projectId = "project-client",
                projectRecommendationMode = "automatic",
            ),
            actionInFlight = false,
        )

        assertFalse(ui.enabled)
        assertEquals("生成项目优选", ui.ctaLabel)
    }

    @Test
    fun localStubSourceIsNotLabelledAsRealModelOutput() {
        assertEquals("本地占位结果", modelEvaluationSourceLabel("local_stub"))
        assertEquals("导入结果", modelEvaluationSourceLabel("imported"))
        assertEquals("模型评价", modelEvaluationSourceLabel("llm_vlm"))
    }

    @Test
    fun projectIntelligenceUiPreservesManualRecommendationMode() {
        val ui = projectIntelligenceSettingsUi(
            settings = ProjectEvaluationSettingsUi(
                projectId = "project-client",
                projectRecommendationMode = "automatic",
            ),
            providerConfigured = true,
        )

        assertEquals("manual", ui.projectRecommendationMode)
    }

    @Test
    fun providerBatchSizeControlPreservesSingleItemBatch() {
        assertEquals(1, providerBatchSizeValue(1))
        assertEquals(1, providerBatchSizeValue(0))
        assertEquals(8, providerBatchSizeValue(99))
    }

    @Test
    fun manualProjectRecommendationActionCallsGatewayOnceAndReportsStatus() = runBlocking {
        val gateway = RecordingProjectRecommendationGateway()
        var feedback: String? = null

        runManualProjectRecommendationAction(
            projectId = "project-client",
            provider = ModelProviderSettingsUi(providerKind = "openai", configured = true),
            gateway = gateway,
            onFeedback = { feedback = it },
        )

        assertEquals(listOf("project-client"), gateway.calls)
        assertEquals("项目优选：已更新", feedback)
    }

    @Test
    fun manualProjectRecommendationActionShowsSetupFeedbackWithoutGatewayCall() = runBlocking {
        val gateway = RecordingProjectRecommendationGateway()
        var feedback: String? = null

        runManualProjectRecommendationAction(
            projectId = "project-client",
            provider = ModelProviderSettingsUi(providerKind = "none", configured = false),
            gateway = gateway,
            onFeedback = { feedback = it },
        )

        assertEquals(emptyList<String>(), gateway.calls)
        assertEquals("请先配置模型服务", feedback)
    }

    @Test
    fun projectRecommendationFeedbackRequiresSameActiveProject() {
        val run = com.cameraconnector.app.core.EvaluationRunUi(
            runId = "run-1",
            projectId = "project-client",
            runType = "project_recommendation",
            trigger = "manual",
            status = "ready",
            providerKind = "openai",
            providerModel = "gpt-5.5",
        )

        assertEquals(
            "项目优选：已更新",
            projectRecommendationFeedbackForActiveProject(run, "project-client"),
        )
        assertNull(projectRecommendationFeedbackForActiveProject(run, "project-other"))
        assertNull(projectRecommendationFeedbackForActiveProject(run, null))
    }

    @Test
    fun projectRecommendationRunIsScopedToActiveProjectForDisplay() {
        val run = com.cameraconnector.app.core.EvaluationRunUi(
            runId = "run-1",
            projectId = "project-client",
            runType = "project_recommendation",
            trigger = "manual",
            status = "ready",
            providerKind = "openai",
            providerModel = "gpt-5.5",
        )

        assertEquals(run, scopedProjectRecommendationRun(run, "project-client"))
        assertNull(scopedProjectRecommendationRun(run, "project-other"))
        assertNull(scopedProjectRecommendationRun(run, null))
        assertNull(scopedProjectRecommendationRun(null, "project-client"))
    }

    @Test
    fun photoDetailDecisionUiHidesActionsForSingleAssets() {
        val decision = photoDetailDecisionUi(inboxAsset(id = "single"), actionsEnabled = true)

        assertFalse(decision.hasAnyAction)
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

    private class RecordingProjectRecommendationGateway : ProjectRecommendationGateway {
        val calls = mutableListOf<String>()

        override suspend fun generateProjectRecommendation(projectId: String) =
            com.cameraconnector.app.core.EvaluationRunUi(
                runId = "run-1",
                projectId = projectId,
                runType = "project_recommendation",
                trigger = "manual",
                status = "ready",
                providerKind = "openai",
                providerModel = "gpt-5.5",
            ).also {
                calls += projectId
            }
    }
}
