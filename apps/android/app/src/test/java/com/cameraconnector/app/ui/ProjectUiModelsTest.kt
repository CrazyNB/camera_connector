package com.cameraconnector.app.ui

import com.cameraconnector.app.core.EvaluationRunUi
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetBurst
import com.cameraconnector.app.core.ProjectAssetTechnicalDefect
import com.cameraconnector.app.core.ProjectAssetUserMarks
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ProjectSummary
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ProjectUiModelsTest {
    @Test
    fun globalDestinationsMatchProjectFirstNavigation() {
        assertEquals(listOf("项目", "账号", "设置"), GlobalDestination.entries.map { it.label })
    }

    @Test
    fun projectWorkspaceDefaultsToPhotos() {
        assertEquals(ProjectDestination.Photos, defaultProjectDestination())
        assertEquals("项目照片", ProjectDestination.Photos.assetScreenTitle())
        assertEquals("照片分组与原始文件", ProjectDestination.Photos.assetScreenSubtitle())
    }

    @Test
    fun projectPhotoCollectionLabelsUseCurrentProductSemantics() {
        assertEquals(
            listOf("全部", "模型优选", "收藏", "标记", "技术风险", "待分析"),
            ProjectPhotoCollection.entries.map { it.label },
        )
    }

    @Test
    fun assetListQueryMapsCollectionFiltersToCurrentModelSemantics() {
        val modelSelects = assetListQuery(
            selectedCollection = ProjectPhotoCollection.ModelSelects,
            selectedFilter = AssetFormatFilter.All,
            selectedSort = PhotoSortMode.LatestReceived,
        )
        val technicalRisk = assetListQuery(
            selectedCollection = ProjectPhotoCollection.TechnicalRisk,
            selectedFilter = AssetFormatFilter.Raw,
            selectedSort = PhotoSortMode.ModelScore,
        )
        val marked = assetListQuery(
            selectedCollection = ProjectPhotoCollection.Marked,
            selectedFilter = AssetFormatFilter.All,
            selectedSort = PhotoSortMode.Filename,
        )

        assertEquals("model_selects", modelSelects.collection)
        assertEquals("technical_risk", technicalRisk.collection)
        assertEquals(com.cameraconnector.app.core.ProjectAssetRole.Raw, technicalRisk.role)
        assertEquals(true, marked.marked)
        assertEquals(PhotoSortMode.Filename, marked.sort)
    }

    @Test
    fun projectPhotoGridItemsCollapseBurstMembersToBestCover() {
        val burst = ProjectAssetBurst(
            burstGroupId = "burst-1",
            memberCount = 2,
            recommendationStatus = "ready",
            bestAssetGroupId = "group-best",
            bestScore = 0.91,
        )
        val alternate = projectAsset(id = "group-alt").copy(burst = burst, modelScore = 67)
        val best = projectAsset(id = "group-best").copy(burst = burst, modelScore = 91)
        val single = projectAsset(id = "single")

        val items = projectPhotoGridItems(listOf(alternate, single, best))

        assertEquals(listOf("burst:burst-1", "asset:single"), items.map { it.key })
        assertTrue(items.first().isBurstGroup)
        assertEquals("group-best", items.first().coverAsset.id)
        assertEquals(listOf("group-alt", "group-best"), items.first().members.map { it.id })
    }

    @Test
    fun burstDetailPositionUsesDerivedMemberOrderOnlyInDetail() {
        val burst = ProjectAssetBurst(
            burstGroupId = "burst-1",
            memberCount = 3,
            recommendationStatus = "ready",
            bestAssetGroupId = "group-a",
            bestScore = 0.93,
        )
        val first = projectAsset(id = "group-a").copy(burst = burst)
        val second = projectAsset(id = "group-b").copy(burst = burst)
        val third = projectAsset(id = "group-c").copy(burst = burst)

        assertEquals("2/3", photoDetailBurstPositionText(second, listOf(first, second, third)))
        assertEquals("3", third.burstCountBadgeText())
    }

    @Test
    fun userFavoriteAndMarkedDoNotChangeAlgorithmRecommendation() {
        val recommended = projectAsset(id = "group-best").copy(
            burst = ProjectAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = "group-best",
                bestScore = 0.93,
            ),
        )
        val favorite = recommended.copy(userMarks = ProjectAssetUserMarks(favorite = true))
        val marked = recommended.copy(userMarks = ProjectAssetUserMarks(marked = true))

        assertTrue(recommended.isBestRecommendedAsset())
        assertFalse(photoDetailFavoriteSelected(recommended))
        assertTrue(photoDetailFavoriteSelected(favorite))
        assertFalse(photoDetailMarkedSelected(recommended))
        assertTrue(photoDetailMarkedSelected(marked))
    }

    @Test
    fun detailActionsStayAvailableForSinglePhotosWhileBurstRemovalIsScopedToBursts() {
        val single = projectAsset(id = "single")
        val burst = ProjectAssetBurst(
            burstGroupId = "burst-1",
            memberCount = 2,
            recommendationStatus = "ready",
            bestAssetGroupId = "group-a",
            bestScore = 0.88,
        )
        val burstMember = projectAsset(id = "group-a").copy(burst = burst)

        val singleDecision = photoDetailDecisionUi(single, actionsEnabled = true)
        val burstDecision = photoDetailDecisionUi(burstMember, actionsEnabled = true)

        assertTrue(photoDetailActionBarVisible(singleDecision, hasActionCallbacks = true))
        assertFalse(singleDecision.splitBurstEnabled)
        assertTrue(photoDetailActionBarVisible(burstDecision, hasActionCallbacks = true))
        assertTrue(burstDecision.splitBurstEnabled)
    }

    @Test
    fun technicalRiskReasonsAreLocalizedForProjectGrid() {
        val asset = projectAsset(id = "group-risk").copy(
            technicalGateStatus = "warn",
            technicalDefects = listOf(
                ProjectAssetTechnicalDefect(
                    defectType = "blur",
                    severity = "high",
                    confidence = 0.82,
                    reason = "soft detail risk",
                ),
            ),
        )

        assertEquals("细节偏软", asset.tileSmartMeta())
    }

    @Test
    fun photoListDefaultFilterSummaryIsHidden() {
        assertNull(photoListFilterSummary(AssetFormatFilter.All, PhotoSortMode.LatestReceived))
        assertEquals("RAW", photoListFilterSummary(AssetFormatFilter.Raw, PhotoSortMode.LatestReceived))
        assertEquals("模型优先", photoListFilterSummary(AssetFormatFilter.All, PhotoSortMode.ModelScore))
        assertEquals("RAW / 模型优先", photoListFilterSummary(AssetFormatFilter.Raw, PhotoSortMode.ModelScore))
    }

    @Test
    fun rawJpegPairIsFormatVariantNotBurstCount() {
        val asset = projectAsset(id = "raw-jpeg").copy(
            hasJpeg = true,
            hasRaw = true,
        )

        assertNull(asset.burstCountBadgeText())
        assertEquals(listOf("JPG+RAW"), asset.tileAuxiliaryBadges())
    }

    @Test
    fun burstPreviewTileUiShowsPositionScoreAndHumanSignals() {
        val burst = ProjectAssetBurst(
            burstGroupId = "burst-1",
            memberCount = 3,
            recommendationStatus = "ready",
            bestAssetGroupId = "group-b",
            bestScore = 0.88,
        )
        val asset = projectAsset(id = "group-b").copy(
            burst = burst,
            modelScore = 88,
            hasJpeg = true,
            hasRaw = true,
            userMarks = ProjectAssetUserMarks(favorite = true),
        )

        val ui = burstPreviewTileUi(
            item = BurstMemberFilmstripItemUi(
                asset = asset,
                badgeText = "",
                scoreText = asset.modelScoreText(),
            ),
            index = 1,
            total = 3,
        )

        assertEquals("2/3", ui.positionText)
        assertEquals("\u8bc4\u5206 88", ui.scoreText)
        assertTrue(ui.modelSelected)
        assertEquals(listOf("\u6536\u85cf", "JPG+RAW"), ui.auxiliaryBadges)
    }

    @Test
    fun selectedBurstGridItemEvaluatesWholeBurstInsteadOfCoverAsset() {
        val burst = ProjectAssetBurst(
            burstGroupId = "burst-1",
            memberCount = 2,
            recommendationStatus = "pending",
            bestAssetGroupId = null,
        )
        val first = projectAsset(id = "group-a").copy(burst = burst)
        val second = projectAsset(id = "group-b").copy(burst = burst)
        val single = projectAsset(id = "single")
        val gridItems = projectPhotoGridItems(listOf(first, second, single))
        val burstItem = gridItems.first { it.isBurstGroup }

        val selectedIds = togglePhotoGridItemSelection(emptyList(), burstItem)
        val selectedItems = selectedPhotoGridItemsFromIds(gridItems, selectedIds)
        val targets = projectPhotoEvaluationTargets(selectedItems)

        assertEquals(listOf("burst-1"), targets.burstGroups.map { it.burstGroupId })
        assertEquals(listOf("group-a", "group-b"), targets.burstGroups.single().members.map { it.id })
        assertEquals(emptyList<ProjectAsset>(), targets.assetGroups)
    }

    @Test
    fun selectedSingleGridItemEvaluatesSingleAssetGroup() {
        val single = projectAsset(id = "single")
        val gridItems = projectPhotoGridItems(listOf(single))

        val selectedIds = togglePhotoGridItemSelection(emptyList(), gridItems.single())
        val targets = projectPhotoEvaluationTargets(selectedPhotoGridItemsFromIds(gridItems, selectedIds))

        assertEquals(listOf(single), targets.assetGroups)
        assertTrue(targets.burstGroups.isEmpty())
    }

    @Test
    fun tilePrimaryAndAuxiliaryBadgesPrioritizeDecisionSignals() {
        val asset = projectAsset(id = "scored-risk").copy(
            modelScore = 84,
            technicalGateStatus = "warn",
            userMarks = ProjectAssetUserMarks(favorite = true, marked = true),
        )

        assertEquals("评分 84", asset.tilePrimaryBadgeText())
        assertEquals(listOf("收藏", "标记"), asset.tileAuxiliaryBadges())
    }

    @Test
    fun modelEvaluationLabelsAndPromptTagsAreChinese() {
        assertEquals("通用 / 均衡", promptStyleTagsText(promptProfile(listOf("general", "balanced"))))
        assertEquals("未命名提示词", promptProfile(name = "").let(::promptProfileDisplayName))
        assertEquals("本地占位结果", modelEvaluationSourceLabel("local_stub"))
        assertEquals("导入结果", modelEvaluationSourceLabel("imported"))
        assertEquals("模型评价", modelEvaluationSourceLabel("llm_vlm"))
        assertEquals("\u5df2\u63a8\u8350", recommendationStatusLabel("ready"))
        assertEquals("\u672a\u63a8\u8350", recommendationStatusLabel("no_selection"))
        assertEquals("已评价", modelEvaluationStatusLabel("ready"))
        assertEquals("未评价", modelEvaluationStatusLabel("skipped"))
        assertEquals("优秀", modelEvaluationTierLabel("excellent"))
        assertEquals("不建议入选", modelEvaluationTierLabel("reject"))
        assertEquals("通过", technicalGateStatusLabel("pass"))
        assertEquals("严重风险", technicalGateStatusLabel("reject"))
        assertEquals("模糊", technicalDefectTypeLabel("blur"))
        assertEquals("高光溢出", technicalDefectTypeLabel("highlight_clip"))
        assertEquals("严重", technicalDefectSeverityLabel("severe"))
    }


    @Test
    fun recommendationEmptyStateDoesNotRenderAsBadge() {
        val asset = projectAsset(id = "asset-no-selection").copy(
            burst = ProjectAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                recommendationStatus = "no_selection",
                bestAssetGroupId = null,
            ),
            modelTier = "reject",
        )

        assertNull(asset.recommendationBadgeText())
        assertEquals("\u4e0d\u5efa\u8bae\u5165\u9009", modelEvaluationTierLabel(asset.modelTier))
    }

    @Test
    fun projectRecommendationRunIsScopedToActiveProjectForDisplay() {
        val run = EvaluationRunUi(
            runId = "run-1",
            projectId = "project-client",
            runType = "project_recommendation",
            trigger = "manual",
            status = "ready",
            providerKind = "openai",
            providerModel = "gpt-5.5",
        )

        assertEquals(run, activeProjectRecommendationRun(run, "project-client"))
        assertNull(activeProjectRecommendationRun(run, "project-other"))
        assertNull(activeProjectRecommendationRun(run, null))
        assertNull(activeProjectRecommendationRun(null, "project-client"))
    }

    @Test
    fun projectLifecycleUiUsesCurrentProjectLabels() {
        val active = projectLifecycleUi(project(status = "Active"), selected = false, actionsEnabled = true)
        val selected = projectLifecycleUi(project(status = "Active"), selected = true, actionsEnabled = true)
        val archived = projectLifecycleUi(project(status = "Archived"), selected = false, actionsEnabled = true)

        assertEquals("\u6d3b\u8dc3", active.statusLabel)
        assertEquals("\u5f53\u524d\u9879\u76ee", selected.statusLabel)
        assertEquals("\u5df2\u5f52\u6863", archived.statusLabel)
    }

    private fun projectAsset(
        id: String,
        displayPath: String = "$id.JPG",
    ): ProjectAsset =
        ProjectAsset(
            id = id,
            groupKey = id,
            displayPath = displayPath,
            format = "Jpeg",
            receivedAt = "0",
        )

    private fun project(status: String): ProjectSummary =
        ProjectSummary(
            id = "project-client",
            name = "Project",
            slug = "project",
            status = status,
            createdAtMs = 0,
            updatedAtMs = 0,
            canBeActiveProject = status.equals("Active", ignoreCase = true),
            canArchive = status.equals("Active", ignoreCase = true),
            canRename = true,
            canRestore = status.equals("Archived", ignoreCase = true),
        )

    private fun promptProfile(
        tags: List<String> = emptyList(),
        name: String = "General Default",
    ) =
        com.cameraconnector.app.core.PromptProfileUi(
            promptProfileId = "general-default",
            scope = "global",
            projectId = null,
            name = name,
            styleTags = tags,
            sceneProfile = "general",
            activeVersionId = "general-default-v1",
            builtIn = true,
            enabled = true,
        )

}
