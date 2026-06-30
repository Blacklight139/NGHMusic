// 职责：Application 类，初始化应用级单例（PlayerManager / MusicCoreBridge）。
// 集成方式：在 AndroidManifest.xml 中通过 android:name=".MusicPlayerApp" 注册。

package com.musicplayer.app

import android.app.Application

class MusicPlayerApp : Application() {
    override fun onCreate() {
        super.onCreate()
        instance = this
    }

    companion object {
        lateinit var instance: MusicPlayerApp
            private set
    }
}
