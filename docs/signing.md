# macOS Code Signing and Distribution

Because `jsonette` is currently developed without a paid Apple Developer Program enrollment, the automated CI pipeline via GitHub Actions utilizes **ad-hoc signing**. It does not perform Apple Notarization or stapling.

## Ad-Hoc Signing & Gatekeeper Bypass

When users download the `.dmg` release artifact from GitHub, macOS Gatekeeper will flag the application as coming from an "Unidentified developer" and display an "App is damaged and can't be opened" warning.

This is expected for locally/ad-hoc signed applications. Users must bypass Gatekeeper using the `xattr` tool in the terminal:

```bash
# After dragging jsonette.app to the Applications folder:
xattr -cr /Applications/jsonette.app
```

This removes the quarantine attribute and allows macOS to run the executable.

## Optional: Upgrading to Apple Developer Program

If the project is ever enrolled in the Apple Developer Program to provide official Developer ID signing and Notarization, the GitHub Action release pipeline can be updated to use the `notarytool` and `stapler` steps.

You would need to define the following GitHub Secrets:

- `APPLE_CERTIFICATE_BASE64`: The base64-encoded `.p12` file containing the Developer ID Application certificate.
- `APPLE_CERTIFICATE_PASSWORD`: The passphrase for the `.p12` file.
- `APPLE_TEAM_ID`: The 10-character Apple Team ID.
- `APPLE_ID`: The Apple ID email address used for notarization.
- `APPLE_ID_PASSWORD`: An app-specific password generated at appleid.apple.com.

**Generating a new Certificate (For enrolled accounts):**

1. Enroll in the Apple Developer Program.
2. Open Xcode -> Preferences -> Accounts.
3. Select your Apple ID and team, then click "Manage Certificates".
4. Click the `+` button and select "Developer ID Application".
5. Export the newly generated certificate to a `.p12` file.
6. Base64-encode the `.p12` file: `base64 -i my_cert.p12 -o my_cert_base64.txt`
