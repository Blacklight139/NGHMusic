// 职责：Android 项目级 build 脚本，声明插件版本。
// 集成方式：与 settings.gradle.kts 配合，Android Studio 打开工程后自动同步。

plugins {
    id("com.android.application") version "8.2.2" apply false
    id("org.jetbrains.kotlin.android") version "1.9.22" apply false
    id("io.gitlab.arturbosch.detekt") version "1.23.7" apply false
}
