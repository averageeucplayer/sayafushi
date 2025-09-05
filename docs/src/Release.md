# Release

## Windows

1. **Set up a self-signed certificate**  
    Run the PowerShell script:  
    ```powershell
    ./tools/new-self-certificate.ps1

2. **Create .secrets.json with the following content:**

    ```json
    {
        "WINDOWS_CERTIFICATE_PASSWORD": <>,
        "TAURI_SIGNING_PRIVATE_KEY_PASSWORD": <>
    }
    ```

3. **Build the project**

    Run `./tools/build.ps1`

## Linux

Simply run `cargo tauri build`