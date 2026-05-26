# FolioFlow

FolioFlow is a desktop image browser built with **Tauri v2 + Rust + Svelte + Vite**.

## Features

- Open a folder and show images in a gallery
- Toggle **short (sorted)** or **random** display mode
- Toggle **subfolder scanning** on/off
- Click an image card to open it with the system default app

## Tech Stack

- Frontend: Svelte + Vite
- Backend: Rust + Tauri
- Tauri plugins: dialog, fs, opener, store
- Rust crates: kamadak-exif, walkdir, ignore, rand, rayon, tokio, serde, serde_json, dirs, dunce, sha2, base64

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## License

This project is licensed under the Apache License 2.0. See the `LICENSE` file for details.
