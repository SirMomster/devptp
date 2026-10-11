FROM base-image:latest

ARG ANDROID_CMDLINE_TOOLS_VERSION=11076708
ARG ANDROID_NDK_VERSION=27.2.12479018
ARG GRADLE_VERSION=8.9

USER root

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        openjdk-17-jdk-headless \
        unzip \
        wget \
    && rm -rf /var/lib/apt/lists/*

ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_SDK_ROOT=/opt/android-sdk
ENV ANDROID_NDK_HOME=/opt/android-sdk/ndk/${ANDROID_NDK_VERSION}
ENV GRADLE_HOME=/opt/gradle/gradle-${GRADLE_VERSION}
ENV PATH=${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${ANDROID_HOME}/emulator:${GRADLE_HOME}/bin:${PATH}

RUN mkdir -p "${ANDROID_HOME}/cmdline-tools" \
    && wget -q -O /tmp/android-tools.zip \
        "https://dl.google.com/android/repository/commandlinetools-linux-${ANDROID_CMDLINE_TOOLS_VERSION}_latest.zip" \
    && unzip -q /tmp/android-tools.zip -d "${ANDROID_HOME}/cmdline-tools" \
    && mv "${ANDROID_HOME}/cmdline-tools/cmdline-tools" "${ANDROID_HOME}/cmdline-tools/latest" \
    && rm /tmp/android-tools.zip \
    && yes | sdkmanager --licenses >/dev/null \
    && sdkmanager \
        "platform-tools" \
        "platforms;android-35" \
        "build-tools;35.0.0" \
        "ndk;${ANDROID_NDK_VERSION}" \
    && wget -q -O /tmp/gradle.zip \
        "https://services.gradle.org/distributions/gradle-${GRADLE_VERSION}-bin.zip" \
    && mkdir -p /opt/gradle \
    && unzip -q /tmp/gradle.zip -d /opt/gradle \
    && rm /tmp/gradle.zip \
    && chown -R vscode:vscode "${ANDROID_HOME}" /opt/gradle

USER vscode

RUN mkdir -p /home/vscode/.cargo \
    && rustup target add aarch64-linux-android

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# The base image already provides Rust. Updating rustup here can fail on
# mounted/overlay filesystems while replacing rust-analyzer components.
RUN cargo binstall --no-confirm --locked tree-sitter-cli

WORKDIR /home/vscode/
