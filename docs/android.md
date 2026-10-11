# Android client

The Android client is a **local port-forwarding client**. It connects to a
Linux serving peer using a devptp ticket and listens on Android loopback for
the ports exposed by that peer. It does not implement `VpnService` or route
all device traffic.

## Rust API boundary

`devptp::mobile::MobileDaemon` is the frontend-neutral API for the Android
service. It owns one `Daemon` and provides:

- `start` / `shutdown`
- `connect` / `disconnect` / `reconnect`
- `ping`
- `status`
- `traffic`
- bounded `logs` and combined `snapshot`

Tickets are retained only for reconnect and are not included in status or log
snapshots. The Android foreground service should own this object; the UI
should observe service state rather than own a second daemon.

## Planned Android layout

- Kotlin + Jetpack Compose UI
- A foreground `DevptpService` while connected
- A generated/native bridge around `MobileDaemon`
- One daemon per service instance

The first Android milestone should expose forwarded services through
`127.0.0.1:<port>` on the device. Full-device VPN routing is intentionally not
supported.

## Native build prerequisites

The devcontainer installs OpenJDK 17, Android command-line tools, Android
platform 35, build-tools 35, NDK `27.2.12479018`, and Gradle 8.9. Rebuild the
container after changing `.devcontainer/Dockerfile`:

```bash
devcontainer up --remove-existing-container
```

Inside the container, install the Rust target:

```bash
rustup target add aarch64-linux-android
```

The container exports `ANDROID_HOME`, `ANDROID_SDK_ROOT`, and
`ANDROID_NDK_HOME`. The native library can then be checked with:

```bash
./scripts/build-android.sh
```

The current Compose app shell and JNI bridge can be built from the
container. The UI/service integration is still an early prototype and needs
device testing against a real serving peer.
