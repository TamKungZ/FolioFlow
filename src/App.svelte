<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { invoke } from '@tauri-apps/api/core'
  import { open } from '@tauri-apps/plugin-dialog'
  import { openPath } from '@tauri-apps/plugin-opener'
  import Icon from '@iconify/svelte'
  import windowMinimize from '@iconify-icons/mdi/window-minimize'
  import windowMaximize from '@iconify-icons/mdi/window-maximize'
  import closeThick from '@iconify-icons/mdi/close-thick'
  import {
    Menu,
    FolderOpen,
    RefreshCcw,
    ChevronLeft,
    ChevronRight,
    Shuffle,
    Image as ImageIcon,
    FolderTree,
    Timer,
    ExternalLink
  } from 'lucide-svelte'

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

  type RandomMode = 'off' | 'one-current' | 'shuffle'
  type ShortMode = 'all' | 'first-n' | 'recent'

  const appWindow = getCurrentWindow()
  const slotCount = 8

  let sidebarOpen = true
  let loading = false
  let currentFolder = ''
  let allImages: ImageEntry[] = []
  let visibleImages: ImageEntry[] = []
  let page = 0

  let includeSubfolders = true
  let randomMode: RandomMode = 'off'
  let shortMode: ShortMode = 'all'
  let shortCount = 80

  $: totalPages = Math.max(1, Math.ceil(visibleImages.length / slotCount))
  $: page = Math.min(page, totalPages - 1)
  $: pageStart = page * slotCount
  $: pageImages = visibleImages.slice(pageStart, pageStart + slotCount)
  $: slotImages = Array.from({ length: slotCount }, (_, i) => pageImages[i] ?? null)
  $: statusText = loading
    ? 'Loading...'
    : `${visibleImages.length}/${allImages.length} images | page ${page + 1}/${totalPages}`

  async function minimizeWindow() {
    await appWindow.minimize()
  }

  async function toggleMaximizeWindow() {
    await appWindow.toggleMaximize()
  }

  async function closeWindow() {
    await appWindow.close()
  }

  function toggleSidebar() {
    sidebarOpen = !sidebarOpen
  }

  function applyDisplayRules(resetPage = true) {
    let items = [...allImages]

    if (shortMode === 'first-n') {
      items = items.slice(0, Math.max(1, shortCount))
    } else if (shortMode === 'recent') {
      items = [...items].sort((a, b) => b.modifiedUnix - a.modifiedUnix).slice(0, Math.max(1, shortCount))
    }

    if (randomMode === 'shuffle') {
      items = [...items].sort(() => Math.random() - 0.5)
    } else if (randomMode === 'one-current') {
      if (items.length > 0) {
        const pick = Math.floor(Math.random() * items.length)
        items = [items[pick]]
      }
    }

    visibleImages = items
    if (resetPage) page = 0
  }

  async function loadFolder() {
    if (!currentFolder) return
    loading = true
    try {
      const data = await invoke<FolderBrowseResult>('browse_images', {
        folder: currentFolder,
        randomMode: false,
        includeSubfolders
      })
      allImages = data.items
      applyDisplayRules(true)
    } finally {
      loading = false
    }
  }

  async function chooseFolder() {
    const selected = await open({ directory: true, multiple: false, title: 'เลือกโฟลเดอร์รูป' })
    if (typeof selected !== 'string') return
    currentFolder = selected
    await loadFolder()
  }

  async function refreshCurrent() {
    await loadFolder()
  }

  function nextPage() {
    if (page < totalPages - 1) page += 1
  }

  function prevPage() {
    if (page > 0) page -= 1
  }

  async function openOriginal(item: ImageEntry | null) {
    if (!item) return
    await openPath(item.path)
  }
</script>

<main class="desktop-shell" data-tauri-drag-region>
  <section class="window-controls" data-tauri-drag-region="false">
    <button class="win-btn" type="button" aria-label="Minimize" on:click={minimizeWindow}>
      <Icon icon={windowMinimize} width="14" height="14" aria-hidden="true" />
    </button>
    <button class="win-btn" type="button" aria-label="Maximize" on:click={toggleMaximizeWindow}>
      <Icon icon={windowMaximize} width="13" height="13" aria-hidden="true" />
    </button>
    <button class="win-btn close" type="button" aria-label="Close" on:click={closeWindow}>
      <Icon icon={closeThick} width="12" height="12" aria-hidden="true" />
    </button>
  </section>

  <section class="layout-root">
    <aside class="sidebar" class:hidden={!sidebarOpen} data-tauri-drag-region="false">
      <button class="sidebar-toggle" type="button" aria-label="Toggle sidebar" on:click={toggleSidebar}>
        <Menu size={18} strokeWidth={1.75} aria-hidden="true" />
      </button>

      <button class="folder-btn" type="button" on:click={chooseFolder}>
        <FolderOpen size={16} strokeWidth={1.8} aria-hidden="true" />
        <span>Open Folder</span>
      </button>

      <button class="folder-btn" type="button" disabled={!currentFolder || loading} on:click={refreshCurrent}>
        <RefreshCcw size={16} strokeWidth={1.8} aria-hidden="true" />
        <span>Reload</span>
      </button>

      <label class="select-row">
        <span><Shuffle size={12} /> Random</span>
        <select bind:value={randomMode} on:change={() => applyDisplayRules(true)}>
          <option value="off">Off</option>
          <option value="one-current">One current</option>
          <option value="shuffle">Shuffle list</option>
        </select>
      </label>

      <label class="select-row">
        <span><Timer size={12} /> Short</span>
        <select bind:value={shortMode} on:change={() => applyDisplayRules(true)}>
          <option value="all">All</option>
          <option value="first-n">First N</option>
          <option value="recent">Recent N</option>
        </select>
      </label>

      <label class="number-row">
        <span><ImageIcon size={12} /> Count</span>
        <input type="number" min="1" max="10000" bind:value={shortCount} on:change={() => applyDisplayRules(true)} />
      </label>

      <label class="check-row">
        <input type="checkbox" bind:checked={includeSubfolders} on:change={refreshCurrent} />
        <span><FolderTree size={12} /> Include subfolders</span>
      </label>

      <div class="folder-text" title={currentFolder}>{currentFolder || 'ยังไม่ได้เลือกโฟลเดอร์'}</div>
      <div class="count-text">{statusText}</div>

      <div class="pager">
        <button class="tiny-btn" type="button" on:click={prevPage} disabled={page === 0}>
          <ChevronLeft size={14} /> Prev
        </button>
        <button class="tiny-btn" type="button" on:click={nextPage} disabled={page >= totalPages - 1}>
          Next <ChevronRight size={14} />
        </button>
      </div>
    </aside>

    <button class="reopen-btn" class:show={!sidebarOpen} type="button" aria-label="Open sidebar" on:click={toggleSidebar} data-tauri-drag-region="false">
      <Menu size={18} strokeWidth={1.75} aria-hidden="true" />
    </button>

    <section class="content-grid" class:expanded={!sidebarOpen} data-tauri-drag-region="false">
      {#each slotImages as image, i}
        <article class={`frame card-${String.fromCharCode(97 + i)}`} class:filled={!!image}>
          {#if image}
            <img src={`asset://${encodeURI(image.path)}`} alt={image.fileName} loading="lazy" />
            <button class="open-file" type="button" on:click={() => openOriginal(image)} aria-label="Open original file">
              <ExternalLink size={14} />
            </button>
          {/if}
        </article>
      {/each}
    </section>
  </section>
</main>
