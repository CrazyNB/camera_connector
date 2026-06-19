package com.cameraconnector.app.share

import com.cameraconnector.app.core.GuestMark
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetUserMarks
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class LanShareHttpServerTest {
    @Test
    fun guestMarkRejectRouteWritesGuestMarkWithoutDeleteCallback() = runBlocking {
        val gateway = RecordingShareGateway()
        val router = LanShareRouter(
            gateway = gateway,
            previewLoader = { _, _, _ -> ByteArray(0) },
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

    @Test
    fun guestMarkRouteRejectsInvalidGuestMark() = runBlocking {
        val gateway = RecordingShareGateway()
        val router = LanShareRouter(
            gateway = gateway,
            previewLoader = { _, _, _ -> null },
        )

        val response = router.handle(
            LanShareRequest(
                method = "PUT",
                path = "/api/s/token-1/assets/group-1/guest-mark",
                body = """{"guest_mark":"delete"}""",
            ),
        )

        assertEquals(400, response.status)
        assertFalse(gateway.marks.containsKey("group-1"))
    }

    @Test
    fun assetsRouteReturnsGuestMarkAndUserMarks() = runBlocking {
        val gateway = RecordingShareGateway(
            assets = listOf(
                ProjectAsset(
                    id = "group-1",
                    displayPath = "IMG_1001.JPG",
                    format = "Jpeg",
                    receivedAt = "10",
                    modelScore = 74,
                    userMarks = ProjectAssetUserMarks(favorite = true),
                    guestMark = GuestMark.Marked,
                ),
            ),
        )
        val router = LanShareRouter(
            gateway = gateway,
            previewLoader = { _, _, _ -> null },
        )

        val response = router.handle(
            LanShareRequest(method = "GET", path = "/api/s/token-1/assets"),
        )
        val body = JSONObject(response.body.toString(Charsets.UTF_8))
        val asset = body.getJSONArray("assets").getJSONObject(0)

        assertEquals(200, response.status)
        assertEquals("marked", asset.getString("guest_mark"))
        assertEquals(74, asset.getInt("model_score"))
        assertEquals(true, asset.getJSONObject("user_marks").getBoolean("favorite"))
    }

    @Test
    fun guestPageIncludesRunnableClientApp() = runBlocking {
        val router = LanShareRouter(
            gateway = RecordingShareGateway(),
            previewLoader = { _, _, _ -> null },
        )

        val response = router.handle(
            LanShareRequest(method = "GET", path = "/s/token-1"),
        )
        val body = response.body.toString(Charsets.UTF_8)

        assertEquals(200, response.status)
        assertEquals("text/html; charset=utf-8", response.contentType)
        assert(body.contains("controls"))
        assert(body.contains("filterButton"))
        assert(body.contains("visibleAssets"))
        assert(body.contains("lightbox"))
        assert(body.contains("openLightbox"))
        assert(body.contains(".chip.photographer"))
        assert(body.contains(".chip.guest"))
        assert(body.contains("asset.guest_mark === mark ? null : mark"))
        assert(body.contains("chip(markLabel(asset.guest_mark), \"guest\")"))
        assert(body.contains("fetch(\"/api/s/\" + encodeURIComponent(token) + \"/assets\")"))
        assert(body.contains("guest-mark"))
        assertFalse(body.contains("取消标记"))
        assertFalse(body.contains("摄影师"))
        assertFalse(body.contains("访客\" +"))
        assertFalse(body.contains("访客："))
        assertFalse(body.contains("Camera Connector</div>"))
        assertFalse(body.contains("class=\"format\""))
    }

    @Test
    fun assetsRouteEncodesPreviewPathAndFallsBackToDisplayPathId() = runBlocking {
        val gateway = RecordingShareGateway(
            assets = listOf(
                ProjectAsset(
                    id = "",
                    displayPath = "DCIM/100/IMG 1001.JPG",
                    format = "Jpeg",
                    receivedAt = "10",
                ),
            ),
        )
        val router = LanShareRouter(
            gateway = gateway,
            previewLoader = { _, _, _ -> null },
        )

        val response = router.handle(
            LanShareRequest(method = "GET", path = "/api/s/token-1/assets"),
        )
        val asset = JSONObject(response.body.toString(Charsets.UTF_8))
            .getJSONArray("assets")
            .getJSONObject(0)

        assertEquals("DCIM/100/IMG 1001.JPG", asset.getString("id"))
        assertEquals("/api/s/token-1/preview/DCIM%2F100%2FIMG%201001.JPG", asset.getString("preview_url"))
        assertEquals(
            "/api/s/token-1/preview-full/DCIM%2F100%2FIMG%201001.JPG",
            asset.getString("full_preview_url"),
        )
    }

    @Test
    fun fullPreviewRouteRequestsFullQualityImage() = runBlocking {
        var requestedFullQuality: Boolean? = null
        val router = LanShareRouter(
            gateway = RecordingShareGateway(),
            previewLoader = { _, _, fullQuality ->
                requestedFullQuality = fullQuality
                byteArrayOf(1, 2, 3)
            },
        )

        val response = router.handle(
            LanShareRequest(method = "GET", path = "/api/s/token-1/preview-full/group-1"),
        )

        assertEquals(200, response.status)
        assertEquals("image/jpeg", response.contentType)
        assertEquals(true, requestedFullQuality)
    }
}

private class RecordingShareGateway(
    private val assets: List<ProjectAsset> = emptyList(),
) : LanShareGateway {
    val marks = mutableMapOf<String, GuestMark?>()
    var deleteCalled = false

    override suspend fun loadAssets(token: String, offset: Int, limit: Int): List<ProjectAsset> =
        assets.drop(offset.coerceAtLeast(0)).take(limit.coerceAtLeast(0))

    override suspend fun setGuestMark(token: String, groupId: String, guestMark: GuestMark?): GuestMark? {
        marks[groupId] = guestMark
        return guestMark
    }
}
