# CI Secrets Setup Guide

How to migrate Apple signing credentials from your local keychain to GitHub Secrets so the CI/CD pipeline can build, sign, and notarize releases.

## 1. Required GitHub Secrets

| Secret | Description | Source |
|--------|-------------|--------|
| `APPLE_CERTIFICATE_BASE64` | Base64-encoded .p12 Developer ID certificate | Keychain Access export (Section 2) |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 file | Set during export (Section 2, step 6) |
| `APPLE_ID` | Apple ID email for notarization | `lakshmantvnm@gmail.com` |
| `APPLE_TEAM_ID` | Apple Developer Team ID | `36L722DZ7X` |
| `APPLE_APP_PASSWORD` | App-specific password for notarization | appleid.apple.com (Section 3) |

## 2. Export Developer ID Certificate (.p12)

1. Open **Keychain Access** (Applications > Utilities > Keychain Access)
2. In the sidebar, select the **login** keychain
3. Set the category filter to **My Certificates**
4. Find **"Developer ID Application: VENKAT LUKSSHMAN TURLAPATI (36L722DZ7X)"**
5. Right-click the certificate and select **Export "Developer ID Application..."**
6. Choose **.p12** format, save the file (e.g., `certificate.p12`), and set a strong password when prompted
7. Base64-encode the exported file:

   ```bash
   base64 -i certificate.p12 | pbcopy
   ```

8. Your clipboard now contains the value for `APPLE_CERTIFICATE_BASE64`
9. Delete the .p12 file from disk after copying:

   ```bash
   rm certificate.p12
   ```

## 3. Generate App-Specific Password

1. Go to [appleid.apple.com](https://appleid.apple.com/) and sign in
2. Navigate to **Sign-In and Security > App-Specific Passwords**
3. Click **Generate an app-specific password**
4. Label it **CMD-K CI Notarization**
5. Copy the generated password -- this is the `APPLE_APP_PASSWORD` value

## 4. Add Secrets to GitHub

1. Go to the GitHub repository page
2. Navigate to **Settings > Secrets and variables > Actions**
3. Click **New repository secret** for each of the following:

   | Name | Value |
   |------|-------|
   | `APPLE_CERTIFICATE_BASE64` | The base64 string from Section 2, step 7 |
   | `APPLE_CERTIFICATE_PASSWORD` | The password you set in Section 2, step 6 |
   | `APPLE_ID` | `lakshmantvnm@gmail.com` |
   | `APPLE_TEAM_ID` | `36L722DZ7X` |
   | `APPLE_APP_PASSWORD` | The app-specific password from Section 3 |

## 5. Windows Signing (Future)

Windows code signing is not yet configured. When ready, add these additional secrets:

| Secret | Description |
|--------|-------------|
| `WINDOWS_CERTIFICATE` | Base64-encoded .pfx code signing certificate |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for the .pfx file |

The release workflow contains a conditional block that activates Windows signing when these secrets are present. No workflow changes needed -- just add the secrets.

## 6. Verification

After adding all secrets:

1. Create and push a version tag:

   ```bash
   git tag v0.2.4
   git push origin v0.2.4
   ```

2. Go to the repository's **Actions** tab
3. Watch the release workflow run
4. Verify the macOS build completes with signing and notarization
5. Check the release artifacts are uploaded correctly

If the workflow fails at the notarization step, double-check that:
- `APPLE_CERTIFICATE_BASE64` was encoded correctly (no line breaks in the secret value)
- `APPLE_APP_PASSWORD` is an app-specific password, not your Apple ID password
- `APPLE_TEAM_ID` matches exactly: `36L722DZ7X`
