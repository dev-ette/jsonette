# macOS Code Signing and Notarization

jsonette uses an automated CI pipeline via GitHub Actions to build, sign, and notarize the macOS `.dmg` artifact.

## CI Secrets

The following GitHub Secrets must be set in the repository:
- `APPLE_CERTIFICATE_BASE64`: The base64-encoded `.p12` file containing the Developer ID Application certificate.
- `APPLE_CERTIFICATE_PASSWORD`: The passphrase for the `.p12` file.
- `APPLE_TEAM_ID`: The 10-character Apple Team ID.
- `APPLE_ID`: The Apple ID email address used for notarization.
- `APPLE_ID_PASSWORD`: An app-specific password (not the account password) generated at appleid.apple.com.

## Generating a new Certificate

If the existing certificate is compromised or expires, you must generate a new one:

1. Enroll in the Apple Developer Program.
2. Open Xcode -> Preferences -> Accounts.
3. Select your Apple ID and team, then click "Manage Certificates".
4. Click the `+` button and select "Developer ID Application".
5. Export the newly generated certificate to a `.p12` file. You will be prompted to create a password.
6. Base64-encode the `.p12` file: `base64 -i my_cert.p12 -o my_cert_base64.txt`
7. Update the GitHub Secrets `APPLE_CERTIFICATE_BASE64` and `APPLE_CERTIFICATE_PASSWORD`.

## Certificate Expiration

Apple Developer ID certificates are typically valid for 5 years.
Current certificate expiration: 2031-07-19 (Placeholder, replace with actual date when generated)

## Local Verification

To verify that the app is signed and notarized locally:
```bash
spctl -a -vv /Volumes/jsonette/jsonette.app
xcrun stapler validate /Volumes/jsonette/jsonette.app
```
