// 职责：MainActivity，ComponentActivity， setContent { MusicPlayerTheme { MainScreen() } }。
// 集成方式：在 AndroidManifest.xml 中注册为 LAUNCHER 入口。

package com.musicplayer.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import com.musicplayer.app.ui.MainScreen
import com.musicplayer.app.ui.theme.MusicPlayerTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            MusicPlayerTheme {
                MainScreen()
            }
        }
    }
}
