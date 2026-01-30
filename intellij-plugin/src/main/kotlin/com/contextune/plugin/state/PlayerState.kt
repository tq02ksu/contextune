package com.contextune.plugin.state

import com.intellij.openapi.components.*
import com.intellij.util.xmlb.XmlSerializerUtil

/**
 * Persistent state for the music player
 */
@State(
    name = "ContextureMusicPlayerState",
    storages = [Storage("contextune-music-player.xml")]
)
@Service(Service.Level.APP)
class PlayerState : PersistentStateComponent<PlayerState> {
    
    // Playback state
    var lastFilePath: String = ""
    var lastPosition: Double = 0.0
    var volume: Double = 0.75
    var isMuted: Boolean = false
    
    // Window state
    var toolWindowVisible: Boolean = false
    
    // Playlist state (for Phase 4)
    var currentPlaylistId: String = ""
    var currentTrackIndex: Int = 0
    
    override fun getState(): PlayerState {
        return this
    }
    
    override fun loadState(state: PlayerState) {
        XmlSerializerUtil.copyBean(state, this)
    }
    
    companion object {
        fun getInstance(): PlayerState {
            return service()
        }
    }
}
