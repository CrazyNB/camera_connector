package com.cameraconnector.app.share

import java.io.BufferedInputStream
import java.net.Socket
import java.net.URLDecoder
import java.net.URLEncoder

internal fun readHttpRequest(socket: Socket): LanShareRequest? {
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

internal fun httpResponseBytes(response: LanShareResponse): ByteArray {
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

internal fun decodePathPart(value: String): String =
    URLDecoder.decode(value, Charsets.UTF_8.name())

internal fun encodePathPart(value: String): String =
    URLEncoder.encode(value, Charsets.UTF_8.name()).replace("+", "%20")

internal fun htmlAttrEscape(value: String): String =
    value
        .replace("&", "&amp;")
        .replace("\"", "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
