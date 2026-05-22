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
    $content = Get-Content -LiteralPath $fullPath -Raw
    if ($content -notmatch $Pattern) {
        throw "Expected '$Path' to contain pattern: $Pattern"
    }
}

Assert-File "docs\product\android-app-architecture.md"
Assert-File "apps\android\settings.gradle.kts"
Assert-File "apps\android\build.gradle.kts"
Assert-File "apps\android\gradle.properties"
Assert-File "apps\android\app\build.gradle.kts"
Assert-File "apps\android\app\src\main\AndroidManifest.xml"
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

Assert-Contains "apps\android\settings.gradle.kts" "CameraConnectorAndroid"
Assert-Contains "apps\android\build.gradle.kts" "com\.android\.application.*9\.2\.0"
Assert-Contains "apps\android\app\build.gradle.kts" "org\.jetbrains\.kotlin\.plugin\.compose"
Assert-Contains "apps\android\app\build.gradle.kts" "compileSdk = 36"
Assert-Contains "apps\android\app\build.gradle.kts" "USE_NATIVE_CORE"
Assert-Contains "apps\android\app\build.gradle.kts" "androidx\.compose:compose-bom:2026\.05\.00"
Assert-Contains "apps\android\app\src\main\AndroidManifest.xml" "android\.permission\.FOREGROUND_SERVICE_DATA_SYNC"
Assert-Contains "apps\android\app\src\main\AndroidManifest.xml" "ReceiverForegroundService"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\MainActivity.kt" "CoreGatewayFactory\.create"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\MainActivity.kt" "ActivityResultContracts\.RequestPermission"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\permissions\AndroidPermissionGateway.kt" "POST_NOTIFICATIONS"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "notificationPermissionGranted"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\ui\CameraConnectorApp.kt" "onRequestNotificationPermission"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt" "BuildConfig\.USE_NATIVE_CORE"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt" "NativeCoreGateway"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt" "ReceiverServiceController"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt" '"inbox"'
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\CoreGatewayFactory.kt" "saveAndroidReceiverDefaults"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" 'System\.loadLibrary\("camera_connector_ffi"\)'
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" "NativeEnvelope"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" '"state_dir"'
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" "startReceiverJson"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt" "stopReceiverJson"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "class NativeCoreGateway"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "CoreGateway"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "pollDashboard"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "gatewayScope\.cancel"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "receiverServiceController\.startReceiver"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeCoreGateway.kt" "receiverServiceController\.stopReceiver"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "ACTION_START"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "ACTION_STOP"
Assert-Contains "apps\android\app\src\main\java\com\cameraconnector\app\service\ReceiverForegroundService.kt" "NativeMobileCore"
Assert-Contains "docs\product\android-app-architecture.md" "Kotlin \+ Jetpack Compose"
Assert-Contains "docs\product\android-app-architecture.md" "Rust core"

Write-Host "Android skeleton checks passed."
