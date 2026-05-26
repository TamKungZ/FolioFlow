<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { open } from '@tauri-apps/plugin-dialog'
  import { openPath } from '@tauri-apps/plugin-opener'

  type ImageEntry = {
    path: string
    fileName: string
    parentFolder: string
    modifiedUnix: number
  }

  type FolderBrowseResult = {
    root: string
    total: number
    items: ImageEntry[]
  }

  let currentFolder = ''
  let randomMode = false
  let includeSub = false
  let summary = '0 รูป'
  let images: ImageEntry[] = []

  async function chooseFolder() {
    const selected = await open({ directory: true, multiple: false, title: 'เลือกโฟลเดอร์รูป' })
    if (typeof selected !== 'string') return
    currentFolder = selected
    await loadFolder()
  }

  async function loadFolder() {
    if (!currentFolder) return
    const data = await invoke<FolderBrowseResult>('browse_images', {
      folder: currentFolder,
      randomMode,
      includeSubfolders: includeSub
    })
    images = data.items
    summary = `${data.total} รูป | ${data.root}`
  }
</script>

<main class="shell">
  <h1>FolioFlow</h1>
  <p class="subtitle">เปิดโฟลเดอร์รูป แล้วแสดงแบบเรียงชื่อ (short) หรือ random พร้อมตัวเลือกแสดง sub folder</p>

  <section class="controls">
    <button on:click={chooseFolder}>เลือกโฟลเดอร์รูป</button>
    <label>
      <input type="checkbox" bind:checked={randomMode} on:change={loadFolder} /> random
    </label>
    <label>
      <input type="checkbox" bind:checked={includeSub} on:change={loadFolder} /> show sub folder
    </label>
    <span id="picked-path">{currentFolder || 'ยังไม่ได้เลือกโฟลเดอร์'}</span>
  </section>

  <section class="meta">{summary}</section>

  <section class="gallery">
    {#each images as item}
      <button class="card" type="button" on:click={() => openPath(item.path)}>
        <img src={`asset://${encodeURI(item.path)}`} alt={item.fileName} loading="lazy" />
        <div class="card-info">
          <strong>{item.fileName}</strong>
          <small>{item.parentFolder}</small>
        </div>
      </button>
    {/each}
  </section>
</main>
