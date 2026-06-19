package com.cameraconnector.app.ui

import android.content.Context
import android.net.ConnectivityManager
import android.net.wifi.WifiManager
import java.net.Inet4Address
import java.net.NetworkInterface

internal enum class ReceiverLanEndpointSource {
    Hotspot,
    SameWifi,
    OtherLan,
}

internal data class ReceiverLanEndpointCandidate(
    val host: String,
    val source: ReceiverLanEndpointSource,
)

internal data class ReceiverNetworkTransportProfile(
    val wifi: Boolean = false,
    val cellular: Boolean = false,
    val vpn: Boolean = false,
    val ethernet: Boolean = false,
    val hotspotInterface: Boolean = false,
    val wifiInterface: Boolean = false,
)

internal data class ReceiverCameraEndpointRowUi(
    val label: String,
    val endpoint: String,
)

internal fun receiverAdvertisedHost(
    localIpv4Addresses: List<String>,
): String? =
    localIpv4Addresses
        .asSequence()
        .map { it.trim() }
        .filter { isReceiverLanEndpointHost(it) }
        .distinct()
        .firstOrNull()

internal fun receiverCameraEndpointRows(
    candidates: List<ReceiverLanEndpointCandidate>,
    port: Int,
): List<ReceiverCameraEndpointRowUi> {
    val rows = candidates
        .asSequence()
        .map { candidate -> candidate.copy(host = candidate.host.trim()) }
        .filter { isReceiverLanEndpointHost(it.host) }
        .groupBy { it.host }
        .values
        .map { duplicates -> duplicates.minBy { it.source.priority } }
        .sortedWith(compareBy<ReceiverLanEndpointCandidate> { it.source.priority }.thenBy { it.host })
        .map { candidate ->
            ReceiverCameraEndpointRowUi(
                label = candidate.source.label,
                endpoint = "${candidate.host}:$port",
            )
        }
        .toList()
    return rows.ifEmpty {
        listOf(
            ReceiverCameraEndpointRowUi(
                label = "\u672a\u68c0\u6d4b\u5230",
                endpoint = "\u672a\u68c0\u6d4b\u5230\u624b\u673a\u5c40\u57df\u7f51\u5730\u5740",
            ),
        )
    }
}

internal fun localIpv4Addresses(): List<String> =
    runCatching {
        val interfaces = NetworkInterface.getNetworkInterfaces() ?: return@runCatching emptyList()
        buildList {
            while (interfaces.hasMoreElements()) {
                val networkInterface = interfaces.nextElement()
                if (!networkInterface.isUp || networkInterface.isLoopback) {
                    continue
                }
                val addresses = networkInterface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val address = addresses.nextElement()
                    if (address is Inet4Address && !address.isLoopbackAddress) {
                        val hostAddress = address.hostAddress ?: continue
                        if (isReceiverLanEndpointHost(hostAddress)) {
                            add(hostAddress)
                        }
                    }
                }
            }
        }.distinct()
    }.getOrDefault(emptyList())

internal fun localIpv4Addresses(context: Context): List<String> =
    localReceiverLanEndpointCandidates(context).map { it.host }.distinct()

internal fun localReceiverLanEndpointCandidates(context: Context): List<ReceiverLanEndpointCandidate> {
    val wifiHosts = wifiEndpointCandidates(context)
        .map { it.host }
        .toSet()
    return buildList {
        addAll(networkInterfaceEndpointCandidates(wifiHosts))
        addAll(connectivityEndpointCandidates(context, wifiHosts))
        addAll(wifiHosts.map { host ->
            ReceiverLanEndpointCandidate(host = host, source = ReceiverLanEndpointSource.SameWifi)
        })
    }
        .map { it.copy(host = it.host.trim()) }
        .filter { isReceiverLanEndpointHost(it.host) }
        .groupBy { it.host }
        .values
        .map { duplicates -> duplicates.minBy { it.source.priority } }
        .sortedWith(compareBy<ReceiverLanEndpointCandidate> { it.source.priority }.thenBy { it.host })
}

internal fun receiverNetworkEndpointSource(
    host: String,
    wifiHosts: Set<String>,
    transportProfile: ReceiverNetworkTransportProfile,
): ReceiverLanEndpointSource? {
    val cleanHost = host.trim()
    if (!isReceiverLanEndpointHost(cleanHost)) {
        return null
    }
    if (cleanHost in wifiHosts) {
        return ReceiverLanEndpointSource.SameWifi
    }
    if (transportProfile.cellular || transportProfile.vpn) {
        return null
    }
    if (transportProfile.hotspotInterface || cleanHost.isLikelyPhoneHotspotGateway()) {
        return ReceiverLanEndpointSource.Hotspot
    }
    if (transportProfile.wifi) {
        return ReceiverLanEndpointSource.Hotspot
    }
    return if (transportProfile.ethernet || transportProfile.wifiInterface) {
        ReceiverLanEndpointSource.OtherLan
    } else {
        null
    }
}

private fun networkInterfaceEndpointCandidates(
    wifiHosts: Set<String> = emptySet(),
): List<ReceiverLanEndpointCandidate> =
    runCatching {
        val interfaces = NetworkInterface.getNetworkInterfaces() ?: return@runCatching emptyList()
        buildList {
            while (interfaces.hasMoreElements()) {
                val networkInterface = interfaces.nextElement()
                if (!networkInterface.isUp || networkInterface.isLoopback) {
                    continue
                }
                val transportProfile = networkInterface.receiverTransportProfile()
                val addresses = networkInterface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val address = addresses.nextElement()
                    if (address is Inet4Address && !address.isLoopbackAddress) {
                        val hostAddress = address.hostAddress ?: continue
                        val source = receiverNetworkEndpointSource(
                            host = hostAddress,
                            wifiHosts = wifiHosts,
                            transportProfile = transportProfile,
                        ) ?: continue
                        add(ReceiverLanEndpointCandidate(host = hostAddress, source = source))
                    }
                }
            }
        }
    }.getOrDefault(emptyList())

private fun connectivityEndpointCandidates(
    context: Context,
    wifiHosts: Set<String> = emptySet(),
): List<ReceiverLanEndpointCandidate> =
    runCatching {
        val manager = context.applicationContext
            .getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            ?: return@runCatching emptyList()
        manager.allNetworks.flatMap { network ->
            val transportProfile = manager.getNetworkCapabilities(network)?.receiverTransportProfile()
                ?: ReceiverNetworkTransportProfile()
            manager.getLinkProperties(network)
                ?.linkAddresses
                ?.mapNotNull { address ->
                    (address.address as? Inet4Address)?.hostAddress?.let { host ->
                        val source = receiverNetworkEndpointSource(
                            host = host,
                            wifiHosts = wifiHosts,
                            transportProfile = transportProfile,
                        ) ?: return@mapNotNull null
                        ReceiverLanEndpointCandidate(host = host, source = source)
                    }
                }
                .orEmpty()
        }
    }.getOrDefault(emptyList())

private fun wifiEndpointCandidates(context: Context): List<ReceiverLanEndpointCandidate> =
    runCatching {
        val manager = context.applicationContext
            .getSystemService(Context.WIFI_SERVICE) as? WifiManager
            ?: return@runCatching emptyList()
        listOf(
            manager.connectionInfo?.ipAddress ?: 0,
            manager.dhcpInfo?.ipAddress ?: 0,
        ).mapNotNull { address ->
            address.takeIf { it != 0 }?.toLittleEndianIpv4Address()?.let { host ->
                ReceiverLanEndpointCandidate(host = host, source = ReceiverLanEndpointSource.SameWifi)
            }
        }
    }.getOrDefault(emptyList())

private val ReceiverLanEndpointSource.priority: Int
    get() = when (this) {
        ReceiverLanEndpointSource.Hotspot -> 0
        ReceiverLanEndpointSource.SameWifi -> 1
        ReceiverLanEndpointSource.OtherLan -> 2
    }

private val ReceiverLanEndpointSource.label: String
    get() = when (this) {
        ReceiverLanEndpointSource.Hotspot -> "\u624b\u673a\u70ed\u70b9"
        ReceiverLanEndpointSource.SameWifi -> "\u540c Wi-Fi"
        ReceiverLanEndpointSource.OtherLan -> "\u5176\u4ed6\u5c40\u57df\u7f51"
    }

private fun NetworkInterface.receiverTransportProfile(): ReceiverNetworkTransportProfile {
    val name = listOf(name, displayName).joinToString(" ").lowercase()
    return ReceiverNetworkTransportProfile(
        wifiInterface = name.contains("wlan") || name.contains("wifi"),
        hotspotInterface = name.contains("softap") ||
            name.contains("hotspot") ||
            name.contains("swlan") ||
            Regex("""(^|\s)ap\d*(\s|$)""").containsMatchIn(name),
        cellular = name.contains("rmnet") ||
            name.contains("ccmni") ||
            name.contains("wwan") ||
            name.contains("pdp") ||
            name.contains("cell") ||
            name.contains("mobile"),
        vpn = name.contains("tun") || name.contains("tap") || name.contains("ppp"),
        ethernet = name.contains("eth"),
    )
}

private fun android.net.NetworkCapabilities.receiverTransportProfile(): ReceiverNetworkTransportProfile =
    ReceiverNetworkTransportProfile(
        wifi = hasTransport(android.net.NetworkCapabilities.TRANSPORT_WIFI),
        cellular = hasTransport(android.net.NetworkCapabilities.TRANSPORT_CELLULAR),
        vpn = hasTransport(android.net.NetworkCapabilities.TRANSPORT_VPN),
        ethernet = hasTransport(android.net.NetworkCapabilities.TRANSPORT_ETHERNET),
    )

private fun String.isLikelyPhoneHotspotGateway(): Boolean {
    val parts = split('.').mapNotNull { it.toIntOrNull() }
    if (parts.size != 4) {
        return false
    }
    return parts[3] == 1 && when (parts[0]) {
        10 -> true
        172 -> parts[1] in 16..31
        192 -> parts[1] == 168
        else -> false
    }
}

private fun Int.toLittleEndianIpv4Address(): String =
    listOf(
        this and 0xff,
        this shr 8 and 0xff,
        this shr 16 and 0xff,
        this shr 24 and 0xff,
    ).joinToString(".")

internal fun isReceiverLanEndpointHost(value: String): Boolean {
    val parts = value.trim().split('.').map { part -> part.toIntOrNull() ?: return false }
    if (parts.size != 4 || parts.any { it !in 0..255 }) {
        return false
    }
    return when (parts[0]) {
        10 -> true
        172 -> parts[1] in 16..31
        192 -> parts[1] == 168
        169 -> parts[1] == 254
        else -> false
    }
}
