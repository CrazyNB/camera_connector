package com.cameraconnector.app.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test
import java.lang.reflect.Modifier

class NativeMobileCorePatchTest {
    @Test
    fun receiverSettingsPatchDoesNotOverwriteAndroidOutputDirectory() {
        val patch = receiverSettingsPatchFields(
            ReceiverSettings(
                protocol = "SFTP",
                host = "0.0.0.0",
                ftpPort = 2121,
                sftpPort = 2222,
                outputLabel = "content://picked-tree",
            ),
        )

        assertEquals("sftp", patch["protocol"])
        assertEquals("0.0.0.0", patch["bind_host"])
        assertEquals(2121, patch["ftp_port"])
        assertEquals(2222, patch["sftp_port"])
        assertFalse(patch.containsKey("output_dir"))
        assertFalse(patch.toString().contains("content://picked-tree"))
    }

    @Test
    fun androidReceiverPathsPatchEnablesDeferredPublish() {
        val patch = androidReceiverPathsPatch(
            outputDir = "/data/user/0/com.cameraconnector.app/files/output",
            stateDir = "/data/user/0/com.cameraconnector.app/files/state",
        )

        assertEquals(
            "/data/user/0/com.cameraconnector.app/files/output",
            patch.getString("output_dir"),
        )
        assertEquals(
            "/data/user/0/com.cameraconnector.app/files/state",
            patch.getString("state_dir"),
        )
        assertEquals(true, patch.getBoolean("defer_publish"))
    }

    @Test
    fun projectAssetStableIdPrefersGroupIdentity() {
        assertEquals("group-123", projectAssetStableId("group-123", "asset-1"))
        assertEquals("asset-1", projectAssetStableId("", "asset-1"))
    }

    @Test
    fun nativeCoreCallsDoNotSerializeReadsThroughSingleSynchronizedGate() {
        val callMethod = NativeMobileCore::class.java
            .declaredMethods
            .first { it.name == "call" }

        assertFalse(
            "Read/write concurrency belongs in the core storage layer, not a global JNI gate",
            Modifier.isSynchronized(callMethod.modifiers),
        )
    }
}
