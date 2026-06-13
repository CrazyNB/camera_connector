package com.cameraconnector.app.share

import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.GuestMark
import com.cameraconnector.app.core.ProjectAsset
import java.io.BufferedInputStream
import java.net.ServerSocket
import java.net.Socket
import java.net.URLDecoder
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.json.JSONArray
import org.json.JSONObject

interface LanShareGateway {
    suspend fun loadAssets(token: String, offset: Int = 0, limit: Int = 2_000): List<ProjectAsset>
    suspend fun setGuestMark(token: String, groupId: String, guestMark: GuestMark?): GuestMark?
}

class CoreLanShareGateway(private val coreGateway: CoreGateway) : LanShareGateway {
    override suspend fun loadAssets(token: String, offset: Int, limit: Int): List<ProjectAsset> =
        coreGateway.loadLanShareAssets(token, offset, limit)

    override suspend fun setGuestMark(token: String, groupId: String, guestMark: GuestMark?): GuestMark? =
        coreGateway.setLanShareGuestMark(token, groupId, guestMark)
}

typealias LanSharePreviewLoader = suspend (token: String, groupId: String) -> ByteArray?

data class LanShareRequest(
    val method: String,
    val path: String,
    val body: String = "",
)

data class LanShareResponse(
    val status: Int,
    val contentType: String = "application/json; charset=utf-8",
    val body: ByteArray = ByteArray(0),
) {
    companion object {
        fun text(status: Int, contentType: String, body: String): LanShareResponse =
            LanShareResponse(status, contentType, body.toByteArray(Charsets.UTF_8))

        fun json(status: Int, value: JSONObject): LanShareResponse =
            text(status, "application/json; charset=utf-8", value.toString())
    }
}

class LanShareRouter(
    private val gateway: LanShareGateway,
    private val previewLoader: LanSharePreviewLoader,
) {
    suspend fun handle(request: LanShareRequest): LanShareResponse {
        val segments = request.path.substringBefore('?')
            .trim('/')
            .split('/')
            .filter { it.isNotBlank() }
        return when {
            request.method == "GET" && segments.size == 2 && segments[0] == "s" ->
                guestPage(segments[1])

            request.method == "GET" &&
                segments.size == 4 &&
                segments[0] == "api" &&
                segments[1] == "s" &&
                segments[3] == "assets" ->
                assetList(segments[2])

            request.method == "GET" &&
                segments.size == 5 &&
                segments[0] == "api" &&
                segments[1] == "s" &&
                segments[3] == "preview" ->
                preview(segments[2], decodePathPart(segments[4]))

            request.method == "PUT" &&
                segments.size == 6 &&
                segments[0] == "api" &&
                segments[1] == "s" &&
                segments[3] == "assets" &&
                segments[5] == "guest-mark" ->
                updateGuestMark(segments[2], decodePathPart(segments[4]), request.body)

            else -> LanShareResponse.json(404, JSONObject().put("error", "not_found"))
        }
    }

    private fun guestPage(token: String): LanShareResponse =
        LanShareResponse.text(
            200,
            "text/html; charset=utf-8",
            """
            <!doctype html>
            <html>
            <head><meta name="viewport" content="width=device-width, initial-scale=1"><title>Camera Connector</title></head>
            <body data-token="$token"><main id="app"></main></body>
            </html>
            """.trimIndent(),
        )

    private suspend fun assetList(token: String): LanShareResponse {
        val assets = gateway.loadAssets(token)
        return LanShareResponse.json(
            200,
            JSONObject().put(
                "assets",
                JSONArray().apply {
                    assets.forEach { put(it.toLanShareJson(token)) }
                },
            ),
        )
    }

    private suspend fun preview(token: String, groupId: String): LanShareResponse {
        val bytes = previewLoader(token, groupId)
            ?: return LanShareResponse.json(404, JSONObject().put("error", "preview_not_found"))
        return LanShareResponse(200, "image/jpeg", bytes)
    }

    private suspend fun updateGuestMark(token: String, groupId: String, body: String): LanShareResponse {
        val payload = body.takeIf { it.isNotBlank() }?.let(::JSONObject) ?: JSONObject()
        val nextMark = if (payload.has("guest_mark") && !payload.isNull("guest_mark")) {
            guestMarkFromWire(payload.optString("guest_mark"))
                ?: return LanShareResponse.json(400, JSONObject().put("error", "invalid_guest_mark"))
        } else {
            null
        }
        val saved = gateway.setGuestMark(token, groupId, nextMark)
        return LanShareResponse.json(
            200,
            JSONObject().put("guest_mark", saved?.wireName ?: JSONObject.NULL),
        )
    }
}

class LanShareHttpServer(
    private val router: LanShareRouter,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : AutoCloseable {
    private var serverSocket: ServerSocket? = null
    private var acceptJob: Job? = null

    val port: Int
        get() = serverSocket?.localPort ?: 0

    fun start(port: Int = 0): Int {
        check(serverSocket == null) { "LAN share server is already running" }
        val socket = ServerSocket(port)
        serverSocket = socket
        acceptJob = scope.launch {
            while (isActive && !socket.isClosed) {
                runCatching { socket.accept() }
                    .onSuccess { client -> launch { handleClient(client) } }
            }
        }
        return socket.localPort
    }

    fun stop() {
        acceptJob?.cancel()
        acceptJob = null
        serverSocket?.close()
        serverSocket = null
    }

    override fun close() {
        stop()
        scope.cancel()
    }

    private fun handleClient(client: Socket) {
        client.use { socket ->
            val request = readHttpRequest(socket) ?: return
            val response = runBlocking { router.handle(request) }
            socket.getOutputStream().use { output ->
                output.write(httpResponseBytes(response))
                output.flush()
            }
        }
    }
}

private fun ProjectAsset.toLanShareJson(token: String): JSONObject =
    JSONObject()
        .put("id", id)
        .put("display_path", displayPath)
        .put("format", format)
        .put("preview_url", "/api/s/$token/preview/$id")
        .put("guest_mark", guestMark?.wireName ?: JSONObject.NULL)
        .put(
            "user_marks",
            JSONObject()
                .put("favorite", userMarks.favorite)
                .put("marked", userMarks.marked),
        )

private fun guestMarkFromWire(value: String): GuestMark? =
    when (value.trim().lowercase()) {
        GuestMark.Favorite.wireName -> GuestMark.Favorite
        GuestMark.Marked.wireName -> GuestMark.Marked
        GuestMark.Reject.wireName -> GuestMark.Reject
        else -> null
    }

private fun readHttpRequest(socket: Socket): LanShareRequest? {
    val input = BufferedInputStream(socket.getInputStream())
    val headerBytes = mutableListOf<Byte>()
    while (true) {
        val next = input.read()
        if (next < 0) return null
        headerBytes.add(next.toByte())
        val size = headerBytes.size
        if (
            size >= 4 &&
            headerBytes[size - 4] == '\r'.code.toByte() &&
            headerBytes[size - 3] == '\n'.code.toByte() &&
            headerBytes[size - 2] == '\r'.code.toByte() &&
            headerBytes[size - 1] == '\n'.code.toByte()
        ) {
            break
        }
    }
    val headers = headerBytes.toByteArray().toString(Charsets.ISO_8859_1)
    val lines = headers.split("\r\n")
    val requestLine = lines.firstOrNull()?.split(' ') ?: return null
    val contentLength = lines
        .firstOrNull { it.startsWith("content-length:", ignoreCase = true) }
        ?.substringAfter(':')
        ?.trim()
        ?.toIntOrNull()
        ?.coerceAtLeast(0)
        ?: 0
    val body = ByteArray(contentLength)
    var read = 0
    while (read < contentLength) {
        val count = input.read(body, read, contentLength - read)
        if (count < 0) break
        read += count
    }
    return LanShareRequest(
        method = requestLine.getOrNull(0).orEmpty(),
        path = requestLine.getOrNull(1).orEmpty(),
        body = body.copyOf(read).toString(Charsets.UTF_8),
    )
}

private fun httpResponseBytes(response: LanShareResponse): ByteArray {
    val statusText = when (response.status) {
        200 -> "OK"
        400 -> "Bad Request"
        404 -> "Not Found"
        else -> "Error"
    }
    val header = buildString {
        append("HTTP/1.1 ${response.status} $statusText\r\n")
        append("Connection: close\r\n")
        append("Content-Type: ${response.contentType}\r\n")
        append("Content-Length: ${response.body.size}\r\n")
        append("\r\n")
    }.toByteArray(Charsets.UTF_8)
    return header + response.body
}

private fun decodePathPart(value: String): String =
    URLDecoder.decode(value, Charsets.UTF_8.name())
