<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { open } from '@tauri-apps/plugin-dialog'
  import { openPath } from '@tauri-apps/plugin-opener'
  import { Store } from '@tauri-apps/plugin-store'

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

  type GalleryMode = 'grid' | 'masonry'
  type RandomMode = 'off' | 'one-current' | 'one-subfolders'
  type ShortMode = 'all' | 'first-n' | 'recent' | 'selected-amount'

  const formatList = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'tiff', 'avif']

  let store: Store | null = null
  let currentFolder = ''
  let summary = '0 รูป'
  let loading = false

  let allImages: ImageEntry[] = []
  let visibleImages: ImageEntry[] = []
  let selectedImage: ImageEntry | null = null

  let galleryMode: GalleryMode = 'grid'
  let randomMode: RandomMode = 'off'
  let shortMode: ShortMode = 'all'
  let includeSub = false
  let shortCount = 50
  let selectedAmount = 20
  let folderViewOnly = ''

  let theme: 'sand' | 'midnight' = 'sand'
  let useThumbnailCache = true
  let supportedFormats: string[] = [...formatList]

  function getExtension(path: string): string {
    const idx = path.lastIndexOf('.')
    return idx < 0 ? '' : path.slice(idx + 1).toLowerCase()
  }

  function applyDisplayRules() {
    let list = [...allImages]

    list = list.filter((item) => supportedFormats.includes(getExtension(item.path)))

    if (folderViewOnly) {
      list = list.filter((item) => item.parentFolder === folderViewOnly)
    }

    if (shortMode === 'first-n') {
      list = list.slice(0, Math.max(1, shortCount))
    } else if (shortMode === 'recent') {
      list = [...list].sort((a, b) => b.modifiedUnix - a.modifiedUnix).slice(0, Math.max(1, shortCount))
    } else if (shortMode === 'selected-amount') {
      list = list.slice(0, Math.max(1, selectedAmount))
    }

    if (randomMode === 'one-current' || randomMode === 'one-subfolders') {
      if (list.length > 1) {
        const pick = Math.floor(Math.random() * list.length)
        list = [list[pick]]
      }
    }

    visibleImages = list
    summary = `${visibleImages.length}/${allImages.length} รูป | ${currentFolder || '-'}${folderViewOnly ? ` | folder: ${folderViewOnly}` : ''}`
  }

  function groupedByFolder(items: ImageEntry[]): Record<string, ImageEntry[]> {
    return items.reduce<Record<string, ImageEntry[]>>((acc, item) => {
      const key = item.parentFolder || '-'
      if (!acc[key]) acc[key] = []
      acc[key].push(item)
      return acc
    }, {})
  }

  $: folderGroups = groupedByFolder(allImages)
  $: folderCards = Object.keys(folderGroups).sort((a, b) => a.localeCompare(b))

  async function saveSettings() {
    if (!store) return
    await store.set('theme', theme)
    await store.set('lastFolder', currentFolder)
    await store.set('supportedFormats', supportedFormats)
    await store.set('thumbnailCache', useThumbnailCache)
    await store.set('galleryMode', galleryMode)
    await store.save()
  }

  async function loadSettings() {
    store = await Store.load('settings.json')
    theme = ((await store.get('theme')) as 'sand' | 'midnight') || 'sand'
    currentFolder = ((await store.get('lastFolder')) as string) || ''
    supportedFormats = ((await store.get('supportedFormats')) as string[]) || [...formatList]
    useThumbnailCache = ((await store.get('thumbnailCache')) as boolean) ?? true
    galleryMode = ((await store.get('galleryMode')) as GalleryMode) || 'grid'
    if (currentFolder) await loadFolder()
    document.documentElement.dataset.theme = theme
  }

  async function chooseFolder() {
    const selected = await open({ directory: true, multiple: false, title: 'เลือกโฟลเดอร์รูป' })
    if (typeof selected !== 'string') return
    currentFolder = selected
    folderViewOnly = ''
    await saveSettings()
    await loadFolder()
  }

  async function chooseImage() {
    const selected = await open({
      directory: false,
      multiple: false,
      title: 'เลือกไฟล์รูป',
      filters: [{ name: 'Images', extensions: formatList }]
    })
    if (typeof selected !== 'string') return
    await openPath(selected)
  }

  async function loadFolder() {
    if (!currentFolder) return
    loading = true
    const wantsSub = includeSub || randomMode === 'one-subfolders'
    const wantsRandom = randomMode !== 'off'
    try {
      const data = await invoke<FolderBrowseResult>('browse_images', {
        folder: currentFolder,
        randomMode: wantsRandom,
        includeSubfolders: wantsSub
      })
      allImages = data.items
      applyDisplayRules()
      await saveSettings()
    } finally {
      loading = false
    }
  }

  async function setTheme(next: 'sand' | 'midnight') {
    theme = next
    document.documentElement.dataset.theme = next
    await saveSettings()
  }

  async function toggleFormat(format: string) {
    supportedFormats = supportedFormats.includes(format)
      ? supportedFormats.filter((f) => f !== format)
      : [...supportedFormats, format]
    applyDisplayRules()
    await saveSettings()
  }

  async function refreshAndApply() {
    if (!currentFolder) return
    await loadFolder()
  }

  onMount(loadSettings)
</script>

<main class="shell">
  <h1>FolioFlow</h1>
  <p class="subtitle">Open Image / Open Folder / Gallery Grid-Masonry / Random-Short-Subfolder / Settings</p>

  <section class="controls">
    <button on:click={chooseImage}>Open Image</button>
    <button on:click={chooseFolder}>Open Folder</button>

    <label>
      Gallery
      <select bind:value={galleryMode} on:change={saveSettings}>
        <option value="grid">Grid</option>
        <option value="masonry">Masonry</option>
      </select>
    </label>

    <label>
      Random
      <select bind:value={randomMode} on:change={refreshAndApply}>
        <option value="off">Off</option>
        <option value="one-current">Random image</option>
        <option value="one-subfolders">Random from subfolders</option>
      </select>
    </label>

    <label>
      Short
      <select bind:value={shortMode} on:change={applyDisplayRules}>
        <option value="all">All</option>
        <option value="first-n">show first N images</option>
        <option value="recent">show recent images</option>
        <option value="selected-amount">show selected amount</option>
      </select>
    </label>

    <label>
      <input type="checkbox" bind:checked={includeSub} on:change={refreshAndApply} /> Subfolder Mode
    </label>

    <label>
      N
      <input type="number" min="1" max="10000" bind:value={shortCount} on:change={applyDisplayRules} />
    </label>

    <label>
      Amount
      <input type="number" min="1" max="10000" bind:value={selectedAmount} on:change={applyDisplayRules} />
    </label>
  </section>

  <section class="settings">
    <strong>Settings</strong>
    <div class="settings-row">
      <span>Theme</span>
      <button class="ghost" on:click={() => setTheme('sand')}>Sand</button>
      <button class="ghost" on:click={() => setTheme('midnight')}>Midnight</button>
    </div>
    <div class="settings-row formats">
      <span>Supported formats</span>
      {#each formatList as fmt}
        <label class="inline">
          <input
            type="checkbox"
            checked={supportedFormats.includes(fmt)}
            on:change={() => toggleFormat(fmt)}
          />
          {fmt}
        </label>
      {/each}
    </div>
    <div class="settings-row">
      <label class="inline">
        <input type="checkbox" bind:checked={useThumbnailCache} on:change={saveSettings} /> Thumbnail cache
      </label>
      <small>Last folder: {currentFolder || '-'}</small>
    </div>
  </section>

  <section class="meta">{loading ? 'Loading...' : summary}</section>

  {#if includeSub && folderCards.length > 0}
    <section class="folder-cards">
      <button class="ghost" on:click={() => { folderViewOnly = ''; applyDisplayRules(); }}>All folders</button>
      {#each folderCards as folder}
        <button class="ghost" on:click={() => { folderViewOnly = folder; applyDisplayRules(); }}>
          {folder} ({folderGroups[folder].length})
        </button>
      {/each}
    </section>
  {/if}

  <section class:gallery={galleryMode === 'grid'} class:masonry={galleryMode === 'masonry'}>
    {#each visibleImages as item}
      <button class="card" type="button" on:click={() => (selectedImage = item)}>
        <img src={`asset://${encodeURI(item.path)}`} alt={item.fileName} loading="lazy" />
        <div class="card-info">
          <strong>{item.fileName}</strong>
          <small>{item.parentFolder}</small>
        </div>
      </button>
    {/each}
  </section>

  {#if selectedImage}
    {@const preview = selectedImage}
    <section class="preview">
      <div class="preview-head">
        <strong>Detail Preview</strong>
        <button class="ghost" on:click={() => (selectedImage = null)}>Close</button>
      </div>
      <img src={`asset://${encodeURI(preview.path)}`} alt={preview.fileName} />
      <div class="preview-meta">
        <div>{preview.fileName}</div>
        <div>{preview.parentFolder}</div>
        <button on:click={() => openPath(preview.path)}>Open with default app</button>
      </div>
    </section>
  {/if}
</main>
