plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
}

val useNativeCore = providers.gradleProperty("cameraConnector.useNativeCore")
    .map { it.toBoolean() }
    .orElse(false)
val nativeCoreFallbackToPreview = providers.gradleProperty("cameraConnector.nativeCoreFallbackToPreview")
    .map { it.toBoolean() }
    .orElse(false)
val repoRoot = rootProject.layout.projectDirectory.asFile.parentFile.parentFile
val buildNativeCoreScript = repoRoot.resolve("scripts/build_android_native.ps1")
val nativeCoreOutputs = listOf(
    layout.projectDirectory.file("src/main/jniLibs/arm64-v8a/libcamera_connector_ffi.so"),
    layout.projectDirectory.file("src/main/jniLibs/x86_64/libcamera_connector_ffi.so"),
)

val buildNativeCore by tasks.registering(Exec::class) {
    group = "build"
    description = "Builds Rust native core libraries for Android when native core packaging is enabled."
    onlyIf { useNativeCore.get() }

    workingDir = repoRoot
    commandLine(
        "powershell",
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        buildNativeCoreScript.absolutePath,
    )

    inputs.file(buildNativeCoreScript)
    inputs.file(repoRoot.resolve("Cargo.toml"))
    inputs.file(repoRoot.resolve("Cargo.lock"))
    inputs.file(repoRoot.resolve("core/Cargo.toml"))
    inputs.file(repoRoot.resolve("core-ffi/Cargo.toml"))
    inputs.dir(repoRoot.resolve("core/src"))
    inputs.dir(repoRoot.resolve("core-ffi/src"))
    outputs.files(nativeCoreOutputs)
}

android {
    namespace = "com.cameraconnector.app"
    compileSdk = 36

    buildFeatures {
        buildConfig = true
    }

    defaultConfig {
        applicationId = "com.cameraconnector.app"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"

        buildConfigField("boolean", "USE_NATIVE_CORE", useNativeCore.get().toString())
        buildConfigField("boolean", "NATIVE_CORE_FALLBACK_TO_PREVIEW", nativeCoreFallbackToPreview.get().toString())
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }
}

tasks.named("preBuild") {
    dependsOn(buildNativeCore)
}

kotlin {
    jvmToolchain(17)
}

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2026.05.00")

    implementation(composeBom)

    implementation("androidx.activity:activity-compose:1.12.1")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.core:core:1.17.0")
    implementation("androidx.documentfile:documentfile:1.1.0")
    implementation("androidx.exifinterface:exifinterface:1.4.1")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.10.0")
    implementation("com.google.mlkit:face-detection:16.1.7")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")

    testImplementation("junit:junit:4.13.2")
    testImplementation("org.json:json:20240303")
}
