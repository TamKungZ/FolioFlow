# FolioFlow

FolioFlow is a desktop image browser built with **Tauri v2 + Rust + Svelte + Vite + Tailwind CSS**.

## Features

- Open Image (file picker + open with system app)
- Open Folder
- Gallery View: Grid / Masonry / Detail Preview
- Random Mode: random image / random from subfolders
- Short Mode: first N / recent / selected amount
- Subfolder Mode: group by folder / folder cards / enter folder
- Settings: theme / last folder / supported formats / thumbnail cache flag

## Tech Stack

- Frontend: Svelte + Vite + Tailwind CSS + bits-ui + lucide-svelte + svelte-motion
- Backend: Rust + Tauri
- Tauri plugins: dialog, fs, opener, store
- Rust crates: image, kamadak-exif, walkdir, ignore, rand, rayon, tokio, serde, serde_json, dirs, dunce, sha2, base64

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
