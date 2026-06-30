// 职责：Android 工程根 settings 脚本，声明仓库与模块。
// 集成方式：在 Android Studio（Arctic Fox+）中 File → Open 选中 android/ 目录即可识别为 Gradle 工程。
// 说明：Gradle 8.x + AGP 8.x + Kotlin 1.9.x。Compose BOM 统一管理 Compose 版本。

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "MusicPlayer"
include(":app")
