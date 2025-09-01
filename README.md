# Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

# Initial config

Set up android / ios development with (https://v2.tauri.app/start/prerequisites/)

- My installation of android studio had

  - Android sdk folder at $HOME/Library/Android/sdk rather than $HOME/Android/Sdk
  - The jbr at /Applications/Android\ Studio.app/Contents/jbr rather than /opt/android-studio/jbr

npm run tauri android init

npm run tauri ios init
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer

# Development

Building and releasing for different targets requires different features corresponding to that target. Currently the rust features cover OS specific details around the following:

Midi support

## Desktop

npm run tauri dev -- -f macos

## Android

npm run tauri android dev -- -f android

## iOS

npm run tauri ios dev -- -f ios
You may need to set a team for signing in xcode

npm run tauri ios dev -- --open -f ios

Or maybe this? Unclear if this is necessary

npm run tauri ios dev -- --open --host -f ios

What about web/wasm?

# Debugging

Check console for logs when possible. When debugging device-specific issues:

## Android

- Run with device attached via USB for USB debugging
- Run with android studio

```
npm run tauri android dev -- -f android --open
```

- Start application (in debug mode?)
- View logs with View -> Tool Windows -> LogCat

# Releasing

## iOS

### One time

### Every time

https://v2.tauri.app/distribute/sign/ios/
https://v2.tauri.app/develop/#developing-your-mobile-application
https://v2.tauri.app/distribute/app-store/
Upload builds and test via test flight.

(may need to remove libraries from xcode -> build phases -> copy bundle resources)

npm run tauri ios build -- -f ios --export-method app-store-connect
xcrun altool --upload-app --type ios --file "src-tauri/gen/apple/build/arm64/maistro.ipa" --apiKey $APPLE_API_KEY_ID --apiIssuer $APPLE_API_ISSUER

## Android

### One time

Set up a signing key: https://v2.tauri.app/distribute/sign/android/

#https://github.com/aws/aws-lc-rs/issues/665
export NDK_ROOT="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk | tail -1)"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk | tail -1)"
export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk | tail -1)"
export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/sysroot"

### Every time

npm run tauri android build -- -f android --apk
Upload to google drive AppApks

## Next steps

Fix radio button bug (not deselecting)
Fix android button styling
Force android landscape
Android everything is too big
Get creds out of app - https://github.com/FabianLars/tauri-plugin-oauth
Sheet history
Create new sheet
Redo in react

## Secrets

FILL ME IN

## Updating app icon

```
npm run tauri icon ./public/appIcon.png
```
