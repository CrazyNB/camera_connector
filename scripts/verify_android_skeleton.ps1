$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")

function Assert-File {
    param([string]$Path)
    $fullPath = Join-Path $root $Path
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "Missing expected file: $Path"
    }
}

function Assert-Contains {
    param(
        [string]$Path,
        [string]$Pattern
    )
    $fullPath = Join-Path $root $Path
    $content = Get-Content -LiteralPath $fullPath -Raw -Encoding UTF8
    if ($content -notmatch $Pattern) {
        throw "Expected '$Path' to contain pattern: $Pattern"
    }
}

function Assert-NotContains {
    param(
        [string]$Path,
        [string]$Pattern
    )
    $fullPath = Join-Path $root $Path
    $content = Get-Content -LiteralPath $fullPath -Raw -Encoding UTF8
    if ($content -match $Pattern) {
        throw "Expected '$Path' not to contain pattern: $Pattern"
    }
}

function U {
    param([int[]]$CodePoints)
    -join ($CodePoints | ForEach-Object { [char]$_ })
}

Assert-File "docs\product\android-app-architecture.md"
Assert-File "apps\android\settings.gradle.kts"
Assert-File "apps\android\build.gradle.kts"
Assert-File "apps\android\gradle.properties"
Assert-File "apps\android\app\build.gradle.kts"
Assert-File "apps\android\app\src\main\AndroidManifest.xml"
Assert-File "apps\android\app\src\main\res\values\strings.xml"
Assert-File "apps\android\app\src\main\java\com\cameraconnector\app\MainActivity.kt"
Assert-File "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt"
Assert-File "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGateway.kt"
Assert-File "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt"
Assert-File "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt"
Assert-File "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt"
Assert-File "apps\android\app\src\main\java\com\cameraconnector\app\permissions\AndroidPermissionGateway.kt"
Assert-File "apps\android\app\src\main\java\com\cameraconnector\app\storage\AndroidStorageGateway.kt"
Assert-File "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverServiceController.kt"
Assert-File "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt"
Assert-File "apps\android\README.md"
Assert-File "scripts\install_android_debug.ps1"
Assert-File "scripts\collect_android_diagnostics.ps1"
Assert-File "scripts\smoke_android_device.ps1"
Assert-File "scripts\inspect_android_apk.ps1"
Assert-File "scripts\preflight_android_device.ps1"
Assert-File "scripts\start_android_emulator.ps1"
Assert-File "scripts\verify_android_emulator_ui.ps1"
Assert-File "scripts\verify_android_emulator_ftp_upload.ps1"

Assert-Contains "apps\android\settings.gradle.kts" "CameraConnectorAndroid"
Assert-Contains "apps\android\build.gradle.kts" "com\.android\.application.*9\.2\.0"
Assert-Contains "apps\android\app\build.gradle.kts" "org\.jetbrains\.kotlin\.plugin\.compose"
Assert-Contains "apps\android\app\build.gradle.kts" "compileSdk = 36"
Assert-Contains "apps\android\app\build.gradle.kts" "USE_NATIVE_CORE"
Assert-Contains "apps\android\app\build.gradle.kts" "androidx\.compose:compose-bom:2026\.05\.00"
Assert-Contains "apps\android\app\src\main\AndroidManifest.xml" "android\.permission\.FOREGROUND_SERVICE_DATA_SYNC"
Assert-Contains "apps\android\app\src\main\AndroidManifest.xml" "ReceiverForegroundService"
Assert-Contains "apps\android\app\src\main\AndroidManifest.xml" "@string/app_name"
Assert-Contains "apps\android\app\src\main\res\values\strings.xml" (U @(0x76F8,0x673A,0x8FDE,0x63A5,0x5668))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\MainActivity.kt" "CoreGatewayFactory\.create"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\MainActivity.kt" "ActivityResultContracts\.RequestPermission"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\MainActivity.kt" "ActivityResultContracts\.OpenDocumentTree"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\MainActivity.kt" "persistInboxDirectory"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\MainActivity.kt" "selectedInboxLabel"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\permissions\AndroidPermissionGateway.kt" "POST_NOTIFICATIONS"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\storage\AndroidStorageGateway.kt" "selectedInboxLabel"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\storage\AndroidStorageGateway.kt" "SharedPreferences"
Assert-Contains "scripts\verify_android_emulator_ui.ps1" "Tap-UiNodeByText"
Assert-Contains "scripts\verify_android_emulator_ui.ps1" "Use-EmulatorBindableReceiverConfig"
Assert-Contains "scripts\verify_android_emulator_ui.ps1" "SFTP 192\.168\.137\.1:2121"
Assert-Contains "scripts\verify_android_emulator_ui.ps1" "FTP 0\.0\.0\.0:2121"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\storage\AndroidStorageGateway.kt" "inbox_uri"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\storage\AndroidStorageGateway.kt" "inbox_grid_columns"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\storage\AndroidStorageGateway.kt" "inboxGridColumnCount"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\storage\AndroidStorageGateway.kt" "persistInboxGridColumnCount"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "notificationPermissionGranted"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "onRequestNotificationPermission"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "selectedInboxLabel"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "onChooseInboxDirectory"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "actionError"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "actionInFlight"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "actionsEnabled"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "runAction"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "finally"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "try"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "onClearActionError"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "PasswordVisualTransformation"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "OutlinedTextField"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "Icons\.Outlined\.Settings"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "SettingsScreen"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "AccountDetailScreen"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "PowerButton"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "ProtocolSegment"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "InboxFilterBar"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "AccountFilterBar"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "OriginalPathFilterBar"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x5168,0x90E8,0x8D26,0x53F7))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x5168,0x90E8,0x8DEF,0x5F84))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "SourceFilterBar"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "PhotoDetailScreen"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "LazyVerticalGrid"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "GridCells\.Fixed\(gridColumnCount\)"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "GridColumnToggle"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "CompactPhotoTile"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "storageGateway\.persistInboxGridColumnCount"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "compactFallback"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "BackHandler"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "stateDescription"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "contentDescription = `"收件箱"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "contentDescription = `"照片"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x0032,0x5217))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x0033,0x5217))
Assert-NotContains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "PhotoInfoCard"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "ReceiverHeroControl"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "PhotoPreview"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "FilterToggleRow"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "loadPreviewBitmap"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "BitmapFactory"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "thumbnailBitmap"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "readRawTiffOrientation"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "TIFF_ORIENTATION_TAG"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "isRawPreviewLocation"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" 'endsWith\("\.nef"\)'
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "FullScreenPhotoPreview"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "ImmersiveSystemBars"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "ContentScale.Fit"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "detectTapGestures"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "rememberTransformableState"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "FULLSCREEN_MAX_SCALE"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "PreviewQuality.Detail"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "PreviewQuality.FullScreen"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "PREVIEW_DETAIL_MAX_DIMENSION_PX"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "PREVIEW_FULLSCREEN_MAX_DIMENSION_PX"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "fitToImageAspect = true"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "FilterQuality.High"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "decodeFullBitmap"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "isJpegPreviewLocation"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "decodeLargestEmbeddedJpeg"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "findEmbeddedJpegRanges"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "Image\("
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "AccountMenuRow"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "onOpenSettings"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "onCloseSettings"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x670D,0x52A1,0x63A7,0x5236))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "ReceiverSettings"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "onSaveReceiverSettings"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "onSaveDeviceAccount"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x603B,0x89C8))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x63A5,0x6536,0x670D,0x52A1))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x8BBE,0x5907,0x8D26,0x53F7))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x5BFC,0x5165,0x4F4D,0x7F6E))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x6536,0x4EF6,0x7BB1))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x4F20,0x8F93,0x8BB0,0x5F55))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x64CD,0x4F5C,0x5931,0x8D25))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x5904,0x7406,0x4E2D))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x4FDD,0x5B58,0x8D26,0x53F7))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x5916,0x90E8,0x6587,0x4EF6,0x5939,0x6388,0x6743))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x5F53,0x524D,0x63A5,0x6536,0x76EE,0x5F55))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x7EDF,0x4E00,0x7AEF,0x53E3))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x672A,0x8FDE,0x63A5))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x65B0,0x589E,0x8D26,0x53F7))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x5220,0x9664,0x8D26,0x53F7))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x767B,0x5F55,0x7528,0x6237,0x540D))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x5168,0x90E8,0x6587,0x4EF6))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "192\.168\.137\.1"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x5168,0x90E8,0x6765,0x6E90))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x8D26,0x53F7))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" (U @(0x8BE6,0x60C5))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "rawPath"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "jpegPath"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGateway.kt" "previewLocation"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "storage_location"
Assert-NotContains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "RAW \\+ JPEG"
Assert-NotContains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "FormatGroupSection"
Assert-NotContains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" '"Overview"'
Assert-NotContains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" '"Start"'
Assert-NotContains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" '"No imported assets yet\."'
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGateway.kt" "password: String\?"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGateway.kt" "activeConnections"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGateway.kt" "latestPort"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGateway.kt" "lastSeenAtMs"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGateway.kt" "authMode"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGateway.kt" "accountCount"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt" "BuildConfig\.USE_NATIVE_CORE"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt" "NativeCoreGateway"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt" "ReceiverServiceController"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt" '"inbox"'
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt" "saveAndroidReceiverPaths"
Assert-NotContains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" "saveAndroidReceiverDefaults"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" 'System\.loadLibrary\("camera_connector_ffi"\)'
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" "NativeEnvelope"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" '"state_dir"'
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" "startReceiverJson"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" "stopReceiverJson"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "class NativeCoreGateway"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "CoreGateway"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "splitHostAndPort"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "pollDashboard"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "recent_failures"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "assets: JSONObject"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" '"Completed"'
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "active_connections"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "last_remote_port"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "last_seen_at_ms"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "auth_mode"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "account_count"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "isNull"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "virtual_display_path"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" '"error"'
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "gatewayScope\.cancel"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "password = password"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "receiverServiceController\.startReceiver"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "receiverServiceController\.stopReceiver"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "ACTION_START"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "ACTION_STOP"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "NativeMobileCore"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "PendingIntent"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "setContentIntent"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "\.addAction\(.*(Stop|$(U @(0x505C,0x6B62)))"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "FLAG_IMMUTABLE"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "CameraConnectorReceiver"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "Log\.i"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "Log\.e"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" (U @(0x76F8,0x673A,0x8FDE,0x63A5,0x5668))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" (U @(0x6B63,0x5728,0x542F,0x52A8,0x63A5,0x6536,0x670D,0x52A1))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" (U @(0x505C,0x6B62))
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" (U @(0x63A5,0x6536,0x670D,0x52A1,0x72B6,0x6001))
Assert-NotContains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" '"Starting receiver"'
Assert-Contains "apps\android\app\src\main\AndroidManifest.xml" "@string/app_name"
Assert-Contains "scripts\install_android_debug.ps1" "verify_android_build\.ps1"
Assert-Contains "scripts\install_android_debug.ps1" "adb"
Assert-Contains "scripts\install_android_debug.ps1" "install"
Assert-Contains "scripts\install_android_debug.ps1" "InstallTimeoutSeconds"
Assert-Contains "scripts\install_android_debug.ps1" "WaitForExit"
Assert-Contains "scripts\install_android_debug.ps1" '\$install\.Refresh\(\)'
Assert-Contains "scripts\install_android_debug.ps1" '\$install\.ExitCode -ne \$null -and \$install\.ExitCode -ne 0'
Assert-Contains "scripts\install_android_debug.ps1" 'Failure \\\['
Assert-Contains "scripts\install_android_debug.ps1" "failed to install"
Assert-Contains "scripts\install_android_debug.ps1" "INSTALL_FAILED"
Assert-Contains "scripts\install_android_debug.ps1" "monkey"
Assert-Contains "scripts\install_android_debug.ps1" "com\.cameraconnector\.app"
Assert-Contains "scripts\verify_android_build.ps1" "inspect_android_apk\.ps1"
Assert-Contains "scripts\verify_android_build.ps1" "cameraConnector\.useNativeCore=true"
Assert-Contains "scripts\build_android_native.ps1" "x86_64-linux-android"
Assert-Contains "scripts\build_android_native.ps1" "arm64-v8a"
Assert-Contains "scripts\build_android_native.ps1" "x86_64"
Assert-Contains "scripts\inspect_android_apk.ps1" "lib/arm64-v8a/libcamera_connector_ffi\.so"
Assert-Contains "scripts\inspect_android_apk.ps1" "lib/x86_64/libcamera_connector_ffi\.so"
Assert-Contains "scripts\inspect_android_apk.ps1" "Get-FileHash"
Assert-Contains "scripts\inspect_android_apk.ps1" "AndroidManifest\.xml"
Assert-Contains "scripts\inspect_android_apk.ps1" "classes\.dex"
Assert-Contains "scripts\inspect_android_apk.ps1" "android-apk-report"
Assert-Contains "scripts\preflight_android_device.ps1" "ro\.build\.version\.sdk"
Assert-Contains "scripts\preflight_android_device.ps1" "ro\.product\.cpu\.abilist"
Assert-Contains "scripts\preflight_android_device.ps1" "x86_64"
Assert-Contains "scripts\preflight_android_device.ps1" "ip addr"
Assert-Contains "scripts\preflight_android_device.ps1" "POST_NOTIFICATIONS"
Assert-Contains "scripts\preflight_android_device.ps1" "android-device-preflight"
Assert-Contains "scripts\start_android_emulator.ps1" "CameraConnector_API36"
Assert-Contains "scripts\start_android_emulator.ps1" "sys\.boot_completed"
Assert-Contains "scripts\start_android_emulator.ps1" "InstallAfterBoot"
Assert-Contains "scripts\start_android_emulator.ps1" "install_android_debug\.ps1"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "USER"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "PASS"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "EPSV"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "STOR"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "RealAssetDirectory"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "RealPairLimit"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "Find-RealRawJpegPairs"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "EDGE_JPG_ONLY"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "EDGE_RAW_ONLY"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "EDGE_DUPLICATE"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "EDGE_NOT_IMAGE"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "transfer-log\.jsonl"
Assert-Contains "scripts\verify_android_emulator_ftp_upload.ps1" "password_hash"
Assert-Contains "scripts\smoke_android_device.ps1" "preflight_android_device\.ps1"
Assert-Contains "scripts\collect_android_diagnostics.ps1" "adb"
Assert-Contains "scripts\collect_android_diagnostics.ps1" "devices"
Assert-Contains "scripts\collect_android_diagnostics.ps1" "dumpsys package"
Assert-Contains "scripts\collect_android_diagnostics.ps1" "dumpsys activity services"
Assert-Contains "scripts\collect_android_diagnostics.ps1" "logcat"
Assert-Contains "scripts\collect_android_diagnostics.ps1" "CameraConnectorReceiver"
Assert-Contains "scripts\smoke_android_device.ps1" "install_android_debug\.ps1"
Assert-Contains "scripts\smoke_android_device.ps1" "collect_android_diagnostics\.ps1"
Assert-Contains "scripts\smoke_android_device.ps1" "SkipInstall"
Assert-Contains "scripts\smoke_android_device.ps1" "pm path"
Assert-Contains "scripts\smoke_android_device.ps1" "POST_NOTIFICATIONS"
Assert-Contains "scripts\smoke_android_device.ps1" "monkey"
Assert-Contains "scripts\smoke_android_device.ps1" "logcat -c"
Assert-Contains "scripts\smoke_android_device.ps1" "pidof"
Assert-Contains "scripts\smoke_android_device.ps1" "Collect-Diagnostics"
Assert-Contains "scripts\smoke_android_device.ps1" "com\.cameraconnector\.app"
Assert-Contains "docs\product\android-app-architecture.md" "Kotlin \+ Jetpack Compose"
Assert-Contains "docs\product\android-app-architecture.md" "Rust core"

Write-Host "Android skeleton checks passed."
