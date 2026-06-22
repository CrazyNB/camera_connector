package com.cameraconnector.app.ui

import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetBurst
import com.cameraconnector.app.core.ProjectAssetUserMarks
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ProjectUiBurstModelsTest {
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
    fun manualBurstMergeRequiresTwoDistinctMergeContainers() {
        val burst = ProjectAssetBurst(
            burstGroupId = "burst-1",
            memberCount = 2,
            recommendationStatus = "ready",
            bestAssetGroupId = "group-a",
            bestScore = 0.91,
        )
        val burstItems = projectPhotoGridItems(
            listOf(
                projectAsset(id = "group-a").copy(burst = burst),
                projectAsset(id = "group-b").copy(burst = burst),
            ),
        )
        val singleA = projectPhotoGridItems(listOf(projectAsset(id = "single-a"))).single()
        val singleB = projectPhotoGridItems(listOf(projectAsset(id = "single-b"))).single()

        assertNull(manualBurstMergeTarget(burstItems))
        assertEquals(
            listOf("single-a", "single-b"),
            manualBurstMergeTarget(listOf(singleA, singleB))?.memberGroupIds,
        )
        assertEquals(
            listOf("group-a", "group-b", "single-a"),
            manualBurstMergeTarget(listOf(burstItems.single(), singleA))?.memberGroupIds,
        )
    }

    @Test
    fun manualBurstSplitTargetsExpandSelectedBurstGroupMembers() {
        val burst = ProjectAssetBurst(
            burstGroupId = "burst-1",
            memberCount = 2,
            recommendationStatus = "ready",
            bestAssetGroupId = "group-a",
            bestScore = 0.91,
        )
        val burstItem = projectPhotoGridItems(
            listOf(
                projectAsset(id = "group-a").copy(burst = burst),
                projectAsset(id = "group-b").copy(burst = burst),
            ),
        ).single()
        val singleItem = projectPhotoGridItems(listOf(projectAsset(id = "single"))).single()

        assertTrue(manualBurstSplitTargets(listOf(singleItem)).isEmpty())
        assertEquals(
            listOf(
                ManualBurstSplitTarget("burst-1", "group-a"),
                ManualBurstSplitTarget("burst-1", "group-b"),
            ),
            manualBurstSplitTargets(listOf(burstItem)),
        )
    }

    @Test
    fun manualBurstMergeSupportsMultipleBurstContainers() {
        val burstA = ProjectAssetBurst(
            burstGroupId = "burst-a",
            memberCount = 2,
            recommendationStatus = "ready",
            bestAssetGroupId = "a-1",
            bestScore = 0.91,
        )
        val burstB = ProjectAssetBurst(
            burstGroupId = "burst-b",
            memberCount = 2,
            recommendationStatus = "ready",
            bestAssetGroupId = "b-1",
            bestScore = 0.89,
        )
        val first = projectPhotoGridItems(
            listOf(
                projectAsset(id = "a-1").copy(burst = burstA),
                projectAsset(id = "a-2").copy(burst = burstA),
            ),
        ).single()
        val second = projectPhotoGridItems(
            listOf(
                projectAsset(id = "b-1").copy(burst = burstB),
                projectAsset(id = "b-2").copy(burst = burstB),
            ),
        ).single()

        assertEquals(
            listOf("a-1", "a-2", "b-1", "b-2"),
            manualBurstMergeTarget(listOf(first, second))?.memberGroupIds,
        )
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
    fun detailLongPressExportUsesPreviewBackedJpegFileName() {
        val asset = projectAsset(id = "group-a").copy(
            displayPath = "DCIM/Camera/DSC_0207.NEF",
            originalPath = "D:/ps/Photos/2026/02/DSC_0207.NEF",
            previewLocation = "/data/user/0/com.cameraconnector.app/files/previews/DSC_0207.jpg",
        )

        assertEquals(
            PhotoDetailExportUi(
                enabled = true,
                fileName = "DSC_0207.jpg",
                unavailableReason = null,
            ),
            photoDetailExportUi(asset),
        )
    }

    @Test
    fun detailLongPressExportDisabledWithoutPreviewSource() {
        val asset = projectAsset(id = "group-a").copy(previewLocation = null)

        assertEquals(
            PhotoDetailExportUi(
                enabled = false,
                fileName = "group-a.jpg",
                unavailableReason = "\u6ca1\u6709\u53ef\u5bfc\u51fa\u7684\u7167\u7247\u9884\u89c8",
            ),
            photoDetailExportUi(asset),
        )
    }

    @Test
    fun deletingPhotoFromDetailReturnsToListInsteadOfSiblingPhoto() {
        val burst = ProjectAssetBurst(
            burstGroupId = "burst-1",
            memberCount = 2,
            recommendationStatus = "ready",
            bestAssetGroupId = "group-a",
            bestScore = 0.88,
        )
        val first = projectAsset(id = "group-a").copy(burst = burst)
        val second = projectAsset(id = "group-b").copy(burst = burst)
        val burstMembers = listOf(
            BurstMemberFilmstripItemUi(first, badgeText = "1/2", scoreText = null),
            BurstMemberFilmstripItemUi(second, badgeText = "2/2", scoreText = null),
        )

        assertNull(photoDetailSelectionAfterDelete(first, burstMembers))
        assertEquals(second, photoDetailSelectionAfterSplit(first, burstMembers))
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
    fun technicalRiskBadgesNormalizeBackendGateStatus() {
        val asset = projectAsset(id = "group-risk").copy(
            technicalGateStatus = " WARN ",
        )

        assertTrue(asset.hasTechnicalRisk())
        assertEquals("风险", asset.tilePrimaryBadgeText())
        assertEquals(ElementDanger, asset.tilePrimaryBadgeColor())
        assertEquals("风险", asset.tileAnalysisBadgeText())
        assertEquals("存在技术风险", asset.smartSummaryText())
    }

    @Test
    fun algorithmRecommendationMatchesStableAssetSelectionId() {
        val displayPath = "DCIM/100NIKON/DSC_0001.JPG"
        val asset = projectAsset(id = "", displayPath = displayPath).copy(
            groupKey = "DSC_0001",
            burst = ProjectAssetBurst(
                burstGroupId = "burst-1",
                memberCount = 2,
                recommendationStatus = "ready",
                bestAssetGroupId = displayPath,
                bestScore = 0.91,
            ),
        )

        assertEquals(displayPath, asset.assetSelectionId())
        assertTrue(asset.isBestRecommendedAsset())
        assertEquals("优选", asset.recommendationBadgeText())
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
    fun jpegOnlyAssetShowsFormatBadge() {
        val asset = projectAsset(id = "jpeg-only").copy(
            hasJpeg = true,
            hasRaw = false,
        )

        assertEquals(listOf("JPG"), asset.tileAuxiliaryBadges())
    }

    @Test
    fun guestMarkBadgeIsHiddenWhenNoGuestMarkExists() {
        assertNull(projectAsset(id = "guest-none").copy(guestMark = null).guestMarkBadgeText())
    }

    @Test
    fun guestMarkBadgeShowsRejectAsGuestDeleteSuggestion() {
        val asset = projectAsset(id = "guest-reject").copy(
            guestMark = GuestMark.Reject,
            hasJpeg = true,
        )

        assertEquals("访客 删除", asset.guestMarkBadgeText())
        assertEquals(listOf("访客 删除", "JPG"), asset.tileAuxiliaryBadges())
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
}
