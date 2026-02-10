# Distribution Guide: Mac App Store vs Direct Distribution

## How macOS App Signing Works (Plain English)

When you download an app on macOS, the system checks: **"Who made this, and has it been tampered with?"** That's what code signing does. Here's how the pieces fit together:

### Certificates — Your Digital Identity

A **certificate** is like a digital passport that says "this app was made by [you]." Apple issues these certificates through their Developer Program ($99/year). There are two types relevant to us:

| Certificate | Used For |
|-------------|----------|
| **Developer ID Application** | Signing apps you distribute **directly** (your website, GitHub Releases) |
| **Apple Distribution** | Signing apps you submit to the **Mac App Store** |

You create a certificate by generating a "Certificate Signing Request" (CSR) in Keychain Access on your Mac, uploading it to the Apple Developer portal, and downloading the certificate back. It lives in your Keychain and is used automatically when you run `codesign`.

### Notarization — Apple's Automated Scan

**Notarization** is an automated process where you upload your signed app to Apple, they scan it for malware, and stamp it as "safe." This is **not** App Review — there's no human involved, and it takes minutes, not days.

Without notarization, macOS shows a scary "this app can't be opened" dialog. With it, users get a clean "downloaded from the internet" prompt and the app just works.

The tool for this is `xcrun notarytool submit`. You give it your `.dmg` or `.app`, wait a few minutes, and you're done.

### Provisioning Profiles — App Store Only

A **provisioning profile** ties together your certificate, your app's bundle ID, and your entitlements (sandbox permissions). It's only needed for the Mac App Store. Think of it as Apple's permission slip that says "this specific app, signed by this developer, is allowed to use these capabilities."

For direct distribution, you don't need one.

### Entitlements — What Your App Is Allowed to Do

**Entitlements** are the permissions your app declares. We already created `Entitlements.plist` with:
- **App Sandbox** — required for the App Store, restricts what your app can access
- **User-selected files (read-write)** — lets the app access folders the user picks via the file dialog

For direct distribution, the sandbox is **optional**. You can remove it entirely, and your app gets full filesystem access (can read `~/.gitconfig`, open any repo without a dialog, etc.).

### The $TEAM_ID Placeholder

In our `Entitlements.plist`, there are `$TEAM_ID` fields. Your Team ID is a 10-character alphanumeric string Apple assigns when you enroll in the Developer Program. You find it at [developer.apple.com](https://developer.apple.com) under Membership. Replace `$TEAM_ID` with the real value when you're ready to sign.

---

## Option A: Direct Distribution with Notarization

### How It Works

1. Build the app with `npx tauri build`
2. Sign it with your **Developer ID Application** certificate
3. Submit to Apple's notarization service (automated, ~2 minutes)
4. Distribute via your website, GitHub Releases, or any file host
5. Users download, drag to Applications, done

### Pros

- **No sandbox restrictions** — the app gets full filesystem access. It can read `~/.gitconfig`, access any repository without the user having to browse to it, and the "recent repos" feature works perfectly across restarts.
- **No App Review** — push updates whenever you want. Fix a bug, build, notarize, release. Takes minutes, not days.
- **No revenue cut** — Apple takes 0%. If you sell the app, you keep 100% (minus payment processor fees if you use Stripe, Gumroad, etc.).
- **Full macOS API access** — no sandbox-imposed limitations on what libraries or system features you can use.
- **Simpler build process** — no provisioning profiles, no manual re-signing workarounds, no `.pkg` packaging.
- **Tauri has good support** — the `tauri-plugin-updater` provides built-in auto-update functionality for directly distributed apps.

### Cons

- **No App Store discoverability** — users won't find your app by searching the App Store.
- **You handle distribution** — hosting downloads, managing updates, dealing with payment processing (if paid).
- **Slightly less user trust** — some non-technical users prefer App Store apps, though notarized apps show a clean macOS dialog.
- **No built-in payment infrastructure** — if you charge for the app, you need to set up your own payment system.

---

## Option B: Mac App Store

### How It Works

1. Build a universal binary (Intel + Apple Silicon)
2. Sign with your **Apple Distribution** certificate
3. Embed a provisioning profile
4. Package as `.pkg` using `productbuild`
5. Upload to App Store Connect
6. Wait for App Review (typically 24–48 hours)
7. Apple hosts and distributes the app

### Pros

- **Discoverability** — users can find the app by searching the Mac App Store.
- **Trust** — "Available on the Mac App Store" carries weight with some users.
- **Apple handles everything** — hosting, downloads, payments, refunds, tax compliance, auto-updates.
- **Simple install experience** — one click to install and update.

### Cons

- **Sandbox is mandatory and severely limiting for this app:**
  - Cannot read `~/.gitconfig` — git2 won't have access to the user's global git config (name, email, aliases). This means the app behaves differently than command-line git.
  - Cannot read `~/.ssh/` — if we ever add remote operations, SSH authentication won't work.
  - **"Recent repos" won't actually work across restarts** — Tauri lacks Security-Scoped Bookmarks support ([open issue since 2022](https://github.com/tauri-apps/tauri/issues/3716)). The `persisted-scope` plugin saves the path but macOS won't restore the sandbox permission. Users must re-browse every time.
  - Only the exact folder the user selects (and its children) is accessible.

- **Apple takes a cut** — 30% of revenue (15% if you qualify for the Small Business Program, under $1M/year).

- **App Review friction** — every update goes through review. A critical bug fix could take 1–2 days to reach users. Rejections happen and can be opaque.

- **Tauri's App Store support is immature:**
  - Manual re-signing may be needed after `tauri build` ([issue #13118](https://github.com/tauri-apps/tauri/issues/13118))
  - Security-scoped bookmarks not supported ([issue #3716](https://github.com/tauri-apps/tauri/issues/3716))
  - Limited first-party documentation for the App Store path

- **Universal binary required** — must build for both Intel and Apple Silicon, which means longer build times and more testing.

- **Additional metadata needed** — screenshots (multiple sizes), description, keywords, privacy policy URL, support URL, age rating questionnaire, 1024x1024 icon without alpha channel.

---

## Recommendation

**Direct distribution with notarization is the clear winner for Git History Editor.** The reasons:

1. **The sandbox breaks core functionality.** A git tool that can't read `~/.gitconfig` or persist repository access across restarts is a compromised experience. Users will be confused when their git config defaults aren't reflected.

2. **The target audience is developers.** They install tools from GitHub, Homebrew, and direct downloads every day. The App Store's trust signal adds little value for this audience.

3. **Iteration speed matters for a v1.0.** Being able to ship fixes in minutes instead of waiting for App Review is a significant advantage while the app is maturing.

4. **The App Store can come later.** If the app gains traction and there's demand, the sandbox work we've already done (Entitlements.plist, persisted-scope plugin, etc.) makes it possible to add App Store distribution later. It's not now-or-never.

### What You'd Need to Set Up (Direct Distribution)

1. **Apple Developer Program** — enroll at [developer.apple.com](https://developer.apple.com), $99/year
2. **Developer ID Application certificate** — create via the Developer portal
3. **Notarize the build** — `xcrun notarytool submit "Git History Editor.dmg" --apple-id you@email.com --team-id YOURTEAMID --password app-specific-password`
4. **Host the download** — GitHub Releases is free and works well
5. **Auto-updates** — add `tauri-plugin-updater` (checks GitHub Releases for new versions)
