package com.cameraconnector.app.ui

import com.cameraconnector.app.core.ProjectAsset

internal data class ManualBurstSplitTarget(
    val burstGroupId: String,
    val memberGroupId: String,
)

internal data class ManualBurstMergeTarget(
    val memberGroupIds: List<String>,
)

internal fun manualBurstSplitTargets(selectedItems: List<ProjectPhotoGridItemUi>): List<ManualBurstSplitTarget> =
    selectedItems
        .flatMap { item ->
            if (item.isBurstGroup) {
                item.members
            } else {
                listOf(item.coverAsset)
            }
        }
        .mapNotNull(::manualBurstSplitTarget)
        .distinctBy { "${it.burstGroupId}\t${it.memberGroupId}" }

internal fun manualBurstSplitTarget(asset: ProjectAsset): ManualBurstSplitTarget? {
    val burst = asset.burst ?: return null
    if (burst.memberCount <= 1) {
        return null
    }
    val burstGroupId = burst.burstGroupId.takeIf { it.isNotBlank() } ?: return null
    val memberGroupId = asset.assetGroupId() ?: return null
    return ManualBurstSplitTarget(
        burstGroupId = burstGroupId,
        memberGroupId = memberGroupId,
    )
}

internal fun projectPhotoGridItems(assets: List<ProjectAsset>): List<ProjectPhotoGridItemUi> {
    val orderedKeys = mutableListOf<String>()
    val grouped = linkedMapOf<String, MutableList<ProjectAsset>>()
    assets.forEach { asset ->
        val burst = asset.burst
        val burstGroupId = burst?.burstGroupId?.takeIf { it.isNotBlank() }
        val key = if (burstGroupId != null && burst.memberCount > 1) {
            "burst:$burstGroupId"
        } else {
            "asset:${asset.assetSelectionId()}"
        }
        val members = grouped.getOrPut(key) {
            orderedKeys += key
            mutableListOf()
        }
        if (members.none { it.assetSelectionId() == asset.assetSelectionId() }) {
            members += asset
        }
    }
    return orderedKeys.mapNotNull { key ->
        val members = grouped[key].orEmpty()
        val sortedMembers = if (key.startsWith("burst:")) {
            members.sortedWith(burstMemberOrderComparator())
        } else {
            members
        }
        val cover = if (key.startsWith("burst:")) {
            sortedMembers.burstCoverAsset()
        } else {
            sortedMembers.firstOrNull()
        } ?: return@mapNotNull null
        ProjectPhotoGridItemUi(
            key = key,
            coverAsset = cover,
            members = sortedMembers,
        )
    }
}

internal fun adjacentBurstMemberAsset(
    currentAsset: ProjectAsset,
    allProjectAssets: List<ProjectAsset>,
    direction: DetailNavigationDirection,
): ProjectAsset? {
    val members = burstMemberFilmstrip(currentAsset, allProjectAssets).map { it.asset }
    val currentIndex = members.indexOfFirst { it.assetSelectionId() == currentAsset.assetSelectionId() }
    if (currentIndex < 0) {
        return null
    }
    val targetIndex = when (direction) {
        DetailNavigationDirection.Previous -> currentIndex - 1
        DetailNavigationDirection.Next -> currentIndex + 1
    }
    return members.getOrNull(targetIndex)
}

internal fun photoDetailBurstPositionText(
    asset: ProjectAsset,
    burstMembers: List<ProjectAsset>,
): String? {
    val burst = asset.burst ?: return null
    if (burst.memberCount <= 1) {
        return null
    }
    val stableAssetId = asset.assetSelectionId()
    val orderedMembers = burstMembers
        .distinctBy { it.assetSelectionId() }
        .sortedWith(burstMemberOrderComparator())
    val rank = orderedMembers
        .indexOfFirst { it.assetSelectionId() == stableAssetId }
        .takeIf { it >= 0 }
        ?.plus(1)
        ?: 1
    return "$rank/${burst.memberCount}"
}

internal fun photoDetailSelectionAfterDelete(
    deletedAsset: ProjectAsset,
    burstMembers: List<BurstMemberFilmstripItemUi>,
): ProjectAsset? = null

internal fun photoDetailSelectionAfterSplit(
    splitAsset: ProjectAsset,
    burstMembers: List<BurstMemberFilmstripItemUi>,
): ProjectAsset? =
    burstMembers
        .map { it.asset }
        .firstOrNull { it.assetSelectionId() != splitAsset.assetSelectionId() }

internal fun adjacentProjectGridAsset(
    currentAsset: ProjectAsset,
    visibleAssets: List<ProjectAsset>,
    direction: DetailNavigationDirection,
): ProjectAsset? {
    val gridItems = projectPhotoGridItems(visibleAssets)
    val currentId = currentAsset.assetSelectionId()
    val currentIndex = gridItems.indexOfFirst { item ->
        item.coverAsset.assetSelectionId() == currentId ||
            item.members.any { member -> member.assetSelectionId() == currentId }
    }
    if (currentIndex < 0) {
        return null
    }
    val targetIndex = when (direction) {
        DetailNavigationDirection.Previous -> currentIndex - 1
        DetailNavigationDirection.Next -> currentIndex + 1
    }
    return gridItems.getOrNull(targetIndex)?.coverAsset
}

private fun List<ProjectAsset>.burstCoverAsset(): ProjectAsset? {
    if (isEmpty()) {
        return null
    }
    val bestGroupId = firstNotNullOfOrNull { asset ->
        asset.burst?.bestAssetGroupId?.takeIf { it.isNotBlank() }
    }
    return if (bestGroupId != null) {
        firstOrNull { asset ->
            asset.id == bestGroupId ||
                asset.assetGroupId() == bestGroupId ||
                asset.assetSelectionId() == bestGroupId
        }
    } else {
        null
    } ?: maxByOrNull { it.modelScore ?: -1 }
        ?: firstOrNull()
}

internal fun manualBurstMergeTarget(selectedItems: List<ProjectPhotoGridItemUi>): ManualBurstMergeTarget? {
    val mergeContainerIds = selectedItems
        .mapNotNull { item -> item.mergeContainerId() }
        .distinct()
    if (mergeContainerIds.size < 2) {
        return null
    }
    val memberGroupIds = selectedItems
        .flatMap { item ->
            if (item.isBurstGroup) {
                item.members.mapNotNull { member -> member.assetGroupId() }
            } else {
                listOfNotNull(item.coverAsset.assetGroupId())
            }
        }
        .distinct()
    if (memberGroupIds.size < 2) {
        return null
    }
    return ManualBurstMergeTarget(
        memberGroupIds = memberGroupIds,
    )
}

private fun ProjectPhotoGridItemUi.mergeContainerId(): String? =
    coverAsset.burst
        ?.burstGroupId
        ?.takeIf { isBurstGroup && it.isNotBlank() }
        ?: coverAsset.assetGroupId()

internal fun photoDetailDecisionUi(
    asset: ProjectAsset,
    actionsEnabled: Boolean,
): PhotoDetailDecisionUi {
    val splitBurstTarget = manualBurstSplitTarget(asset)
    return PhotoDetailDecisionUi(
        splitBurstTarget = splitBurstTarget,
        splitBurstEnabled = actionsEnabled && splitBurstTarget != null,
        disabledReason = null,
    )
}

internal fun photoDetailActionBarVisible(
    decision: PhotoDetailDecisionUi,
    hasActionCallbacks: Boolean,
): Boolean = hasActionCallbacks

internal fun photoDetailFavoriteSelected(asset: ProjectAsset): Boolean =
    asset.userMarks.favorite

internal fun photoDetailMarkedSelected(asset: ProjectAsset): Boolean =
    asset.userMarks.marked

internal fun photoDetailExportUi(asset: ProjectAsset): PhotoDetailExportUi {
    val fileName = photoDetailExportFileName(asset)
    val hasPreviewSource = !asset.previewLocation.isNullOrBlank() &&
        !asset.previewLocation.equals("null", ignoreCase = true)
    return PhotoDetailExportUi(
        enabled = hasPreviewSource,
        fileName = fileName,
        unavailableReason = if (hasPreviewSource) {
            null
        } else {
            "\u6ca1\u6709\u53ef\u5bfc\u51fa\u7684\u7167\u7247\u9884\u89c8"
        },
    )
}

private fun photoDetailExportFileName(asset: ProjectAsset): String {
    val sourceName = listOfNotNull(
        asset.originalPath,
        asset.displayPath,
        asset.previewLocation,
        asset.id,
    )
        .firstOrNull { it.isNotBlank() }
        .orEmpty()
    val baseName = sourceName
        .substringBefore('?')
        .substringBefore('#')
        .substringAfterLast('/')
        .substringAfterLast('\\')
    val cleanName = baseName
        .substringBeforeLast('.', baseName)
        .replace(Regex("""[\\/:*?"<>|]+"""), "_")
        .trim()
        .ifBlank { asset.id.ifBlank { "photo" } }
    return "$cleanName.jpg"
}

internal fun burstPreviewTileUi(
    item: BurstMemberFilmstripItemUi,
    index: Int,
    total: Int,
): BurstPreviewTileUi =
    BurstPreviewTileUi(
        positionText = "${index + 1}/$total",
        scoreText = (item.scoreText ?: item.asset.modelScoreText())?.let { "\u8bc4\u5206 $it" },
        modelSelected = item.asset.isBestRecommendedAsset(),
        auxiliaryBadges = item.asset.tileAuxiliaryBadges(),
    )

internal fun burstMemberFilmstrip(
    currentAsset: ProjectAsset,
    allProjectAssets: List<ProjectAsset>,
): List<BurstMemberFilmstripItemUi> {
    val burstGroupId = currentAsset.burst?.burstGroupId?.takeIf { it.isNotBlank() } ?: return emptyList()
    val currentId = currentAsset.assetSelectionId()
    val members = allProjectAssets
        .asSequence()
        .filter { asset -> asset.burst?.burstGroupId == burstGroupId }
        .distinctBy { it.assetSelectionId() }
        .sortedWith(burstMemberOrderComparator())
        .toList()
    if (members.size <= 1) {
        return emptyList()
    }
    return members.map { asset ->
        BurstMemberFilmstripItemUi(
            asset = asset,
            badgeText = burstMemberBadgeText(asset, currentId),
            scoreText = asset.modelScoreText(),
        )
    }
}

private fun burstMemberBadgeText(asset: ProjectAsset, currentId: String): String =
    when {
        asset.isBestRecommendedAsset() -> "\u6700\u4f73"
        asset.assetSelectionId() == currentId -> "\u5f53\u524d"
        (asset.modelScore ?: 100) < 40 -> "\u4f4e\u5206"
        asset.hasTechnicalRisk() -> "\u98ce\u9669"
        else -> "\u5907\u9009"
    }
private fun burstMemberOrderComparator(): Comparator<ProjectAsset> =
    compareBy<ProjectAsset> { it.receivedAt.toLongOrNull() ?: Long.MAX_VALUE }
        .thenBy(::burstMemberOrderKey)
        .thenBy { it.assetSelectionId() }

private fun burstMemberOrderKey(asset: ProjectAsset): String =
    asset.groupKey.ifBlank { asset.displayPath }
