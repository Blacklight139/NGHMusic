// 职责：app 模块 build 脚本，含 Jetpack Compose + Media3 ExoPlayer 依赖。
// 集成 music-core（Rust）方式：
// 1. 在 core/ 下为 Android 目标编译动态库：
//    cargo build --release --target aarch64-linux-android
//    cargo build --release --target armv7-linux-androideabi
//    cargo build --release --target x86_64-linux-android
// 2. 通过 UniFFI 生成 Kotlin 绑定：
//    uniffi-bindgen generate core/src/ffi.udl --language kotlin \
//      --out-dir android/app/src/main/java/com/musicplayer/core
//    得到 com/musicplayer/core/musiccore.kt（包名 com.musicplayer.core）
// 3. 将各 ABI 的 libmusic_core.so 放入：
//    android/app/src/main/jniLibs/<abi>/libmusic_core.so
// 4. 在下方 android.defaultConfig 中配置 ndk abiFilters，并在 dependencies 引入 jniLibs。
// 备注：脚手架未实际链接 .so，MusicCoreBridge 提供占位实现保证编译通过。

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.musicplayer.app"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.musicplayer.app"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
        // 链接 music-core 后启用对应 ABI
        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
    buildFeatures { compose = true }
    composeOptions { kotlinCompilerExtensionVersion = "1.5.8" }
    packaging {
        resources.excludes += "/META-INF/{AL2.0,LGPL2.1}"
    }
}

dependencies {
    // Compose BOM 统一版本
    val composeBom = platform("androidx.compose:compose-bom:2024.02.00")
    implementation(composeBom)
    androidTestImplementation(composeBom)

    implementation("androidx.core:core-ktx:1.12.0")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.7.0")
    implementation("androidx.activity:activity-compose:1.8.2")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("androidx.navigation:navigation-compose:2.7.7")

    // Media3 ExoPlayer 播放组件
    implementation("androidx.media3:media3-exoplayer:1.2.1")
    implementation("androidx.media3:media3-ui:1.2.1")
    implementation("androidx.media3:media3-session:1.2.1")

    // music-core UniFFI Kotlin 绑定（生成后启用）
    // implementation(files("src/main/java/com/musicplayer/core/musiccore.kt"))

    debugImplementation("androidx.compose.ui:ui-tooling")
}
