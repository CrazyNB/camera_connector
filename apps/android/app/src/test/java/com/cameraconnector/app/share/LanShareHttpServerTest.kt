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
            previewLoader = { _, _ -> ByteArray(0) },
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
            previewLoader = { _, _ -> null },
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
                    userMarks = ProjectAssetUserMarks(favorite = true),
                    guestMark = GuestMark.Marked,
                ),
            ),
        )
        val router = LanShareRouter(
            gateway = gateway,
            previewLoader = { _, _ -> null },
        )

        val response = router.handle(
            LanShareRequest(method = "GET", path = "/api/s/token-1/assets"),
        )
        val body = JSONObject(response.body.toString(Charsets.UTF_8))
        val asset = body.getJSONArray("assets").getJSONObject(0)

        assertEquals(200, response.status)
        assertEquals("marked", asset.getString("guest_mark"))
        assertEquals(true, asset.getJSONObject("user_marks").getBoolean("favorite"))
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
