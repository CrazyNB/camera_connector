package com.cameraconnector.app.media

import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertEquals
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

    @Test
    fun persistentThumbnailNamesUseDedicatedFolderAndHashedJpegNames() {
        val first = persistentThumbnailFileName("/storage/emulated/0/DCIM/DSC_0001.JPG")
        val second = persistentThumbnailFileName("/storage/emulated/0/DCIM/DSC_0002.JPG")

        assertEquals("preview_thumbnails", PERSISTENT_THUMBNAIL_DIRECTORY_NAME)
        assertTrue(first.endsWith(".jpg"))
        assertFalse(first.contains("DSC_0001"))
        assertNotEquals(first, second)
        assertEquals(first, persistentThumbnailFileName("/storage/emulated/0/DCIM/DSC_0001.JPG"))
    }

    @Test
    fun detailAndFullscreenPreviewUseScreenSizedDecodeBounds() {
        assertEquals(512, previewDecodeMaxDimensionPx(PreviewQuality.Thumbnail))
        assertEquals(2560, previewDecodeMaxDimensionPx(PreviewQuality.Detail))
        assertEquals(2560, previewDecodeMaxDimensionPx(PreviewQuality.FullScreen))
    }
}
