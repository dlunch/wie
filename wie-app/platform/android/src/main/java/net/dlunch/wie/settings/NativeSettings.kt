package net.dlunch.wie.settings

import android.content.Context
import androidx.annotation.Keep
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.floatPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.json.JSONObject

private val Context.settingsDataStore by preferencesDataStore(name = "settings")
private val midiVolumeKey = floatPreferencesKey("midiVolume")
private val pcmVolumeKey = floatPreferencesKey("pcmVolume")
private val helpDismissedKey = booleanPreferencesKey("helpDismissed")
private val welcomeSeenKey = booleanPreferencesKey("welcomeSeen")

@Keep
class NativeSettings(context: Context) {
    private val context = context.applicationContext

    fun read(): String = runBlocking(Dispatchers.IO) {
        val settings = context.settingsDataStore.data.first()
        JSONObject()
            .put("midiVolume", settings[midiVolumeKey] ?: 0.5f)
            .put("pcmVolume", settings[pcmVolumeKey] ?: 0.5f)
            .put("helpDismissed", settings[helpDismissedKey] ?: false)
            .put("welcomeSeen", settings[welcomeSeenKey] ?: false)
            .toString()
    }

    fun write(midiVolume: Float, pcmVolume: Float, helpDismissed: Boolean, welcomeSeen: Boolean): Unit = runBlocking(Dispatchers.IO) {
        context.settingsDataStore.edit { settings ->
            settings[midiVolumeKey] = midiVolume
            settings[pcmVolumeKey] = pcmVolume
            settings[helpDismissedKey] = helpDismissed
            settings[welcomeSeenKey] = welcomeSeen
        }
        Unit
    }
}
