package com.devptp

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import org.json.JSONObject

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent { DevptpScreen() }
    }
}

@Composable
private fun DevptpScreen() {
    var ticket by remember { mutableStateOf("") }
    var state by remember { mutableStateOf("Starting…") }
    var handle by remember { mutableLongStateOf(0L) }

    DisposableEffect(Unit) {
        handle = runCatching { MobileNative.start() }.getOrDefault(0L)
        state = if (handle == 0L) "Native runtime failed to start" else "Disconnected"
        onDispose {
            if (handle != 0L) MobileNative.shutdown(handle)
        }
    }

    MaterialTheme {
        Column(
            modifier = Modifier.fillMaxSize().padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            Text("devptp", style = MaterialTheme.typography.headlineMedium)
            Text(state)
            OutlinedTextField(
                value = ticket,
                onValueChange = { ticket = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Serving ticket") },
                minLines = 3,
            )
            Button(
                onClick = {
                    val response = MobileNative.connect(handle, ticket)
                    state = responseState(response)
                },
                enabled = handle != 0L && ticket.isNotBlank(),
            ) { Text("Connect") }
            Button(
                onClick = {
                    state = responseState(MobileNative.disconnect(handle))
                },
                enabled = handle != 0L,
            ) { Text("Disconnect") }
        }
    }
}

private fun responseState(response: String): String = runCatching {
    val json = JSONObject(response)
    if (json.optBoolean("ok")) "Connected" else json.optString("error", "Operation failed")
}.getOrDefault("Native operation failed")
