package com.cameraconnector.app.ui

import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.EvaluationRunUi
import com.cameraconnector.app.core.GuestMark
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetBurst
import com.cameraconnector.app.core.ProjectAssetTechnicalDefect
import com.cameraconnector.app.core.ProjectAssetUserMarks
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.core.TechnicalAssessmentPolicyUi
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ProjectUiModelsTest {
    @Test
    fun cvThresholdControlSpecsExposeUserFacingRiskKnobsInsteadOfInternalThresholds() {
        val controls = cvThresholdControlSpecs(
            TechnicalAssessmentPolicyUi(
                blurSevereEdgeThreshold = 0.04,
                blurSevereFrequencyThreshold = 0.04,
                blurHighEdgeThreshold = 0.12,
                blurHighFrequencyThreshold = 0.12,
                highlightClipThreshold = 245,
                shadowClipThreshold = 5,
                clippingHighRatio = 0.12,
                clippingHighConnectedRatio = 0.18,
                clippingSevereRatio = 0.50,
                clippingSevereConnectedRatio = 0.50,
                colorCastHighThreshold = 0.42,
                colorCastSevereThreshold = 0.70,
                faceEyeOpenWarnThreshold = 0.35,
                faceExposureWarnRatio = 0.25,
                faceColorCastWarnThreshold = 0.42,
            ),
        )

        assertEquals(
            listOf(
                CvThresholdControlKey.BlurHigh,
                CvThresholdControlKey.Clipping,
                CvThresholdControlKey.ShadowClipThreshold,
                CvThresholdControlKey.HighlightClipThreshold,
                CvThresholdControlKey.ColorCast,
            ),
            controls.map { it.key },
        )
        assertEquals(38, controls.first { it.key == CvThresholdControlKey.BlurHigh }.displayPercent)
        assertEquals(
            59,
            controls.first { it.key == CvThresholdControlKey.Clipping }.displayPercent,
        )
        assertEquals(
            56,
            controls.first { it.key == CvThresholdControlKey.ColorCast }.displayPercent,
        )
        assertEquals("<=5", controls.first { it.key == CvThresholdControlKey.ShadowClipThreshold }.displayLabel)
        assertEquals(">=245", controls.first { it.key == CvThresholdControlKey.HighlightClipThreshold }.displayLabel)
        assertEquals(
            "当前：边缘和高频细节都低于 12% 时标记失焦；低于 4% 视为严重。",
            controls.first { it.key == CvThresholdControlKey.BlurHigh }.description,
        )
        assertTrue(controls.first { it.key == CvThresholdControlKey.Clipping }.description.contains("<=5"))
        assertTrue(controls.first { it.key == CvThresholdControlKey.Clipping }.description.contains(">=245"))
        assertEquals(
            "当前：RGB 通道相对亮度差异超过 0.42 时标记偏色；超过 0.70 视为严重。",
            controls.first { it.key == CvThresholdControlKey.ColorCast }.description,
        )
    }

    @Test
    fun portraitCvThresholdControlSpecsExposeFaceSpecificRiskKnobs() {
        val controls = cvThresholdControlSpecs(
            TechnicalAssessmentPolicyUi(
                blurSevereEdgeThreshold = 0.04,
                blurSevereFrequencyThreshold = 0.04,
                blurHighEdgeThreshold = 0.12,
                blurHighFrequencyThreshold = 0.12,
                highlightClipThreshold = 245,
                shadowClipThreshold = 5,
                clippingHighRatio = 0.12,
                clippingHighConnectedRatio = 0.18,
                clippingSevereRatio = 0.50,
                clippingSevereConnectedRatio = 0.50,
                colorCastHighThreshold = 0.42,
                colorCastSevereThreshold = 0.70,
                faceEyeOpenWarnThreshold = 0.35,
                faceExposureWarnRatio = 0.25,
                faceColorCastWarnThreshold = 0.42,
            ),
            sceneProfile = "portrait",
        )

        assertEquals(
            listOf(
                CvThresholdControlKey.BlurHigh,
                CvThresholdControlKey.Clipping,
                CvThresholdControlKey.ShadowClipThreshold,
                CvThresholdControlKey.HighlightClipThreshold,
                CvThresholdControlKey.ColorCast,
                CvThresholdControlKey.FaceEyes,
                CvThresholdControlKey.FaceExposure,
                CvThresholdControlKey.FaceColorCast,
            ),
            controls.map { it.key },
        )
    }

    @Test
    fun cvThresholdControlUpdatesPairedTechnicalFields() {
        val policy = TechnicalAssessmentPolicyUi(
            blurSevereEdgeThreshold = 0.04,
            blurSevereFrequencyThreshold = 0.04,
            blurHighEdgeThreshold = 0.12,
            blurHighFrequencyThreshold = 0.12,
            highlightClipThreshold = 245,
            shadowClipThreshold = 5,
            clippingHighRatio = 0.12,
            clippingHighConnectedRatio = 0.18,
            clippingSevereRatio = 0.50,
            clippingSevereConnectedRatio = 0.50,
            colorCastHighThreshold = 0.42,
            colorCastSevereThreshold = 0.70,
            faceEyeOpenWarnThreshold = 0.35,
            faceExposureWarnRatio = 0.25,
            faceColorCastWarnThreshold = 0.42,
        )

        val blurUpdated = updateCvThresholdControl(policy, CvThresholdControlKey.BlurHigh, 0.75)
        assertEquals(0.18, blurUpdated.blurHighEdgeThreshold, 0.0001)
        assertEquals(0.18, blurUpdated.blurHighFrequencyThreshold, 0.0001)

        val clippingUpdated = updateCvThresholdControl(policy, CvThresholdControlKey.Clipping, 1.0)
        assertEquals(0.04, clippingUpdated.clippingHighRatio, 0.0001)
        assertEquals(0.04, clippingUpdated.clippingHighConnectedRatio, 0.0001)
        assertEquals(0.35, clippingUpdated.clippingSevereRatio, 0.0001)
        assertEquals(0.35, clippingUpdated.clippingSevereConnectedRatio, 0.0001)

        val shadowThresholdUpdated = updateCvThresholdControl(policy, CvThresholdControlKey.ShadowClipThreshold, 0.0)
        assertEquals(0, shadowThresholdUpdated.shadowClipThreshold)

        val highlightThresholdUpdated = updateCvThresholdControl(
            policy,
            CvThresholdControlKey.HighlightClipThreshold,
            0.5,
        )
        assertEquals(245, highlightThresholdUpdated.highlightClipThreshold)

        val colorCastUpdated = updateCvThresholdControl(policy, CvThresholdControlKey.ColorCast, 1.0)
        assertEquals(0.28, colorCastUpdated.colorCastHighThreshold, 0.0001)
        assertEquals(0.50, colorCastUpdated.colorCastSevereThreshold, 0.0001)

        val faceEyesUpdated = updateCvThresholdControl(policy, CvThresholdControlKey.FaceEyes, 1.0)
        assertEquals(0.55, faceEyesUpdated.faceEyeOpenWarnThreshold, 0.0001)

        val faceExposureUpdated = updateCvThresholdControl(policy, CvThresholdControlKey.FaceExposure, 1.0)
        assertEquals(0.12, faceExposureUpdated.faceExposureWarnRatio, 0.0001)

        val faceColorCastUpdated = updateCvThresholdControl(policy, CvThresholdControlKey.FaceColorCast, 1.0)
        assertEquals(0.28, faceColorCastUpdated.faceColorCastWarnThreshold, 0.0001)
    }

    @Test
    fun cvThresholdModeSelectionUsesCustomAsAPresetOption() {
        val settings = ProjectEvaluationSettingsUi(projectId = "project-1", cvPolicy = "standard")

        assertEquals("standard", selectedCvThresholdMode(settings))

        val custom = projectSettingsAfterCvThresholdModeSelection(settings, "custom")
        assertEquals("custom", selectedCvThresholdMode(custom))
        assertEquals("standard", custom.cvPolicy)
        assertEquals(5, custom.cvPolicyOverrides?.shadowClipThreshold)
        assertEquals(245, custom.cvPolicyOverrides?.highlightClipThreshold)

        val strict = projectSettingsAfterCvThresholdModeSelection(custom, "strict")
        assertEquals("strict", selectedCvThresholdMode(strict))
        assertEquals("strict", strict.cvPolicy)
        assertNull(strict.cvPolicyOverrides)
    }

    @Test
    fun detailCarouselHeightGivesLandscapePhotosMoreViewingSpace() {
        assertEquals(340.dp, detailCarouselHeight(null))
        assertEquals(304.dp, detailCarouselHeight(1.5f))
        assertEquals(520.dp, detailCarouselHeight(0.67f))
    }

    @Test
    fun photoMetadataAspectRatioUsesExifRotationForPortraitCameraFiles() {
        assertEquals(1.5f, photoMetadataDisplayAspectRatio("6048 x 4032", "正常")!!, 0.0001f)
        assertEquals(4032f / 6048f, photoMetadataDisplayAspectRatio("6048 x 4032", "旋转 90°")!!, 0.0001f)
        assertEquals(4032f / 6048f, photoMetadataDisplayAspectRatio("6048 x 4032", "旋转 270°")!!, 0.0001f)
    }

    @Test
    fun projectStorageSegmentsSplitProjectBytesFromOtherUsedSpace() {
        assertEquals(
            StorageBarSegments(projectRatio = 0.10f, otherUsedRatio = 0.30f),
            storageBarSegments(
                storage = DeviceStorageSnapshot(totalBytes = 100, availableBytes = 60),
                projectBytes = 10,
            ),
        )
        assertEquals(
            StorageBarSegments(projectRatio = 0.40f, otherUsedRatio = 0f),
            storageBarSegments(
                storage = DeviceStorageSnapshot(totalBytes = 100, availableBytes = 60),
                projectBytes = 80,
            ),
        )
    }

    @Test
    fun detailInfoLinesArePreparedForCompactGrid() {
        val asset = projectAsset(
            id = "group-a",
            displayPath = "DCIM/100/DSC_0001.JPG",
        ).copy(
            displaySource = "Verify Camera",
            username = "camera-a",
            originalPath = "DCIM/100/DSC_0001.JPG",
            sizeBytes = 12_345,
            rawPath = "DCIM/100/DSC_0001.NEF",
            jpegPath = "DCIM/100/DSC_0001.JPG",
        )

        assertEquals(
            listOf(
                "来源" to "Verify Camera",
                "账号" to "camera-a",
                "原始路径" to "DCIM/100/DSC_0001.JPG",
                "接收时间" to formatEpochMillisTextForDisplay(asset.receivedAt),
                "文件大小" to "12345 bytes",
            ),
            photoDetailSourceLines(asset),
        )
        assertEquals(
            listOf(
                "位置" to "DCIM/100/DSC_0001.JPG",
                "RAW" to "DCIM/100/DSC_0001.NEF",
                "JPEG" to "DCIM/100/DSC_0001.JPG",
            ),
            photoDetailFileLines(asset),
        )
    }

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
    fun projectWorkspaceNavigationPreservesPhotoListWhenReturningFromOtherTabs() {
        val openState = ProjectWorkspaceNavigationState(workspaceOpen = true)
        val closedState = ProjectWorkspaceNavigationState(workspaceOpen = false)

        assertTrue(
            projectWorkspaceStateAfterBottomDestinationClick(
                current = openState,
                destination = GlobalDestination.Settings,
            ).workspaceOpen,
        )
        assertTrue(
            projectWorkspaceStateAfterBottomDestinationClick(
                current = openState,
                destination = GlobalDestination.Projects,
            ).workspaceOpen,
        )
        assertFalse(
            projectWorkspaceStateAfterBottomDestinationClick(
                current = openState,
                destination = GlobalDestination.Projects,
                collapseCurrentProjectWorkspace = true,
            ).workspaceOpen,
        )
        assertFalse(
            projectWorkspaceStateAfterBottomDestinationClick(
                current = closedState,
                destination = GlobalDestination.Projects,
            ).workspaceOpen,
        )
        assertFalse(projectWorkspaceStateAfterOpenProjects(openState).workspaceOpen)
    }

    @Test
    fun projectWorkspaceVisibilityNeedsBothUserIntentAndActiveProject() {
        assertTrue(projectWorkspaceVisible(workspaceOpen = true, activeProjectId = "project-client"))
        assertFalse(projectWorkspaceVisible(workspaceOpen = true, activeProjectId = null))
        assertFalse(projectWorkspaceVisible(workspaceOpen = false, activeProjectId = "project-client"))
    }

    @Test
    fun projectPhotoCollectionLabelsUseCurrentProductSemantics() {
        assertEquals(
            listOf("全部", "模型优选", "收藏", "标记", "技术风险", "待分析"),
            ProjectPhotoCollection.entries.map { it.label },
        )
    }

    @Test
    fun selectingProjectModelProviderDoesNotImplicitlyRequirePromptPack() {
        val settings = ProjectEvaluationSettingsUi(
            projectId = "project-1",
            modelProviderSettingsId = null,
            promptPackId = null,
        )

        val updated = projectSettingsAfterModelProviderSelection(settings, "provider-openai")

        assertEquals("provider-openai", updated.modelProviderSettingsId)
        assertNull(updated.promptPackId)
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
    fun lanShareActionRequiresActiveProjectAndAssets() {
        assertFalse(lanShareActionUi(activeProjectId = null, assetCount = 3, running = false).enabled)
        assertFalse(lanShareActionUi(activeProjectId = "project-1", assetCount = 0, running = false).enabled)
        assertFalse(lanShareActionUi(activeProjectId = "project-1", assetCount = 3, running = true).enabled)
        assertTrue(lanShareActionUi(activeProjectId = "project-1", assetCount = 3, running = false).enabled)
    }

    @Test
    fun lanShareMenuSeparatesGuestSelectionFromProjectSync() {
        assertEquals(
            listOf("\u591a\u65b9\u7b5b\u9009", "\u5c40\u57df\u7f51\u9879\u76ee\u5171\u4eab"),
            lanShareMenuItems().map { it.label },
        )
    }

    @Test
    fun projectSyncShareDoesNotExposeUserVisibleLink() {
        assertFalse(lanShareDialogShowsUserVisibleLink(LanShareMenuAction.ProjectSync, sharingActive = true))
        assertTrue(lanShareDialogShowsUserVisibleLink(LanShareMenuAction.GuestSelection, sharingActive = true))
        assertFalse(lanShareDialogShowsUserVisibleLink(LanShareMenuAction.GuestSelection, sharingActive = false))
    }

    @Test
    fun receiverAdvertisedHostUsesPhoneLanAddressAndIgnoresPublicAddresses() {
        assertEquals(
            "192.168.50.2",
            receiverAdvertisedHost(
                localIpv4Addresses = listOf("8.8.8.8", "192.168.50.2"),
            ),
        )
        assertNull(
            receiverAdvertisedHost(
                localIpv4Addresses = listOf("8.8.8.8"),
            ),
        )
    }

    @Test
    fun receiverCameraEndpointRowsShowAllLanAddressesWithPort() {
        assertEquals(
            listOf(
                ReceiverCameraEndpointRowUi("\u624b\u673a\u70ed\u70b9", "192.168.43.1:2121"),
                ReceiverCameraEndpointRowUi("\u540c Wi-Fi", "192.168.50.23:2121"),
                ReceiverCameraEndpointRowUi("\u5176\u4ed6\u5c40\u57df\u7f51", "172.20.10.1:2121"),
            ),
            receiverCameraEndpointRows(
                candidates = listOf(
                    ReceiverLanEndpointCandidate("8.8.8.8", ReceiverLanEndpointSource.OtherLan),
                    ReceiverLanEndpointCandidate("192.168.50.23", ReceiverLanEndpointSource.SameWifi),
                    ReceiverLanEndpointCandidate("172.20.10.1", ReceiverLanEndpointSource.OtherLan),
                    ReceiverLanEndpointCandidate("192.168.43.1", ReceiverLanEndpointSource.Hotspot),
                ),
                port = 2121,
            ),
        )
    }

    @Test
    fun receiverCameraEndpointRowsShowUnavailableStateWhenNoLanAddress() {
        assertEquals(
            listOf(ReceiverCameraEndpointRowUi("\u672a\u68c0\u6d4b\u5230", "\u672a\u68c0\u6d4b\u5230\u624b\u673a\u5c40\u57df\u7f51\u5730\u5740")),
            receiverCameraEndpointRows(
                candidates = emptyList(),
                port = 2121,
            ),
        )
    }

    @Test
    fun receiverEndpointSourceSeparatesWifiHotspotAndCellularPrivateAddresses() {
        val wifiHosts = setOf("192.168.31.158")

        assertEquals(
            ReceiverLanEndpointSource.SameWifi,
            receiverNetworkEndpointSource(
                host = "192.168.31.158",
                wifiHosts = wifiHosts,
                transportProfile = ReceiverNetworkTransportProfile(wifi = true),
            ),
        )
        assertEquals(
            ReceiverLanEndpointSource.Hotspot,
            receiverNetworkEndpointSource(
                host = "172.19.0.1",
                wifiHosts = wifiHosts,
                transportProfile = ReceiverNetworkTransportProfile(wifi = true),
            ),
        )
        assertNull(
            receiverNetworkEndpointSource(
                host = "10.13.254.167",
                wifiHosts = wifiHosts,
                transportProfile = ReceiverNetworkTransportProfile(cellular = true),
            ),
        )
        assertNull(
            receiverNetworkEndpointSource(
                host = "10.13.254.167",
                wifiHosts = wifiHosts,
                transportProfile = ReceiverNetworkTransportProfile(),
            ),
        )
    }

    @Test
    fun receiverCollapsedStatusDoesNotExposeFtpEndpoint() {
        val receiver = ReceiverState(
            running = true,
            phase = "Running",
            protocol = "FTP",
            authMode = "Accounts",
            accountCount = 1,
            host = "0.0.0.0",
            port = 2121,
            outputLabel = "out",
            message = null,
        )

        assertEquals("\u8fd0\u884c\u4e2d", receiverCollapsedStatusLabel(receiver))
        assertFalse(receiverCollapsedStatusLabel(receiver).contains("2121"))
    }

    @Test
    fun lanShareAssetQueryKeepsCurrentListAndCombinesTagsAsOr() {
        val baseQuery = assetListQuery(
            selectedCollection = ProjectPhotoCollection.ModelSelects,
            selectedFilter = AssetFormatFilter.Raw,
            selectedSort = PhotoSortMode.ModelScore,
            selectedGuestMarkFilter = GuestMarkFilter.Reject,
            selectedMinModelScore = 80,
        )
        val query = lanShareAssetQuery(
            baseQuery = baseQuery,
            favoriteOnly = true,
            markedOnly = true,
            minModelScore = 70,
        )

        assertEquals("model_selects", query.collection)
        assertEquals(com.cameraconnector.app.core.ProjectAssetRole.Raw, query.role)
        assertEquals(PhotoSortMode.ModelScore, query.sort)
        assertNull(query.favorite)
        assertNull(query.marked)
        assertEquals(listOf("favorite", "marked"), query.userMarkAny)
        assertEquals("reject", query.guestMark)
        assertEquals(70, query.minModelScore)
    }

    @Test
    fun assetListQueryIncludesGuestMarkAndScoreFilters() {
        val query = assetListQuery(
            selectedCollection = ProjectPhotoCollection.All,
            selectedFilter = AssetFormatFilter.All,
            selectedSort = PhotoSortMode.LatestReceived,
            selectedGuestMarkFilter = GuestMarkFilter.None,
            selectedMinModelScore = 80,
        )

        assertEquals("none", query.guestMark)
        assertEquals(80, query.minModelScore)
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

    @Test
    fun projectEvaluationFeedbackNamesBurstFlowAsEvaluation() {
        assertEquals("已完成单张评价 2", projectEvaluationFeedback(2, 0))
        assertEquals("已完成连拍评价 1", projectEvaluationFeedback(0, 1))
        assertEquals("已完成单张评价 2 · 连拍评价 1", projectEvaluationFeedback(2, 1))
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
        assertEquals("通用 / 均衡", promptStyleTagsText(promptPack(listOf("general", "balanced"))))
        assertEquals("未命名提示词", promptPack(name = "").let(::promptPackDisplayName))
        assertEquals("\u672c\u5730\u5206\u6790\u7ed3\u679c", modelEvaluationSourceLabel("local_stub"))
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

    private fun promptPack(
        tags: List<String> = emptyList(),
        name: String = "General Default",
    ) =
        com.cameraconnector.app.core.PromptPackUi(
            promptPackId = "general-default",
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
