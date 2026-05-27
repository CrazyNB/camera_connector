package com.cameraconnector.app.ui

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PreviewLocationTest {
    @Test
    fun mediaStoreImageUrisArePreviewableWithoutFilenameExtensions() {
        assertTrue(isDecodablePreviewLocation("content://media/external/images/media/6010"))
        assertTrue(isDecodablePreviewLocation("content://media/external_primary/images/media/6010"))
        assertFalse(isRawPreviewLocation("content://media/external/images/media/6010"))
    }

    @Test
    fun documentUrisStillUseFilenameExtensionsWhenAvailable() {
        assertTrue(
            isDecodablePreviewLocation(
                "content://com.android.externalstorage.documents/tree/primary%3ADCIM/document/primary%3ADCIM%2FIMG_0100.JPG",
            ),
        )
        assertTrue(
            isJpegPreviewLocation(
                "content://com.android.externalstorage.documents/tree/primary%3ADCIM/document/primary%3ADCIM%2FIMG_0100.JPG",
            ),
        )
    }
}
