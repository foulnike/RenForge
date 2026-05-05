<template>
  <div class="gallery-workspace">
      <aside class="sidebar media-sidebar">
          <div class="sidebar-title">{{ t('folders') }}</div>
          <div class="sidebar-list">
              <div class="sidebar-item media-folder-item" :class="{ active: gallerySelectedFolder === '' }" @click="gallerySelectedFolder = ''">
                  <span class="folder-name">{{ t('all_folders') }}</span>
              </div>
              <div class="sidebar-item media-folder-item" :class="{ active: gallerySelectedFolder === '/', 'is-faded': hiddenFolders.includes('/') }" @click="gallerySelectedFolder = '/'">
                  <span class="folder-name">{{ t('root_folder') }}</span>
                  <button class="icon-text-btn folder-eye-btn" @click.stop="toggleHideFolder('/')" :title="t('hide_folder_hint')">[{{ hiddenFolders.includes('/') ? t('btn_show') : t('btn_hide') }}]</button>
              </div>
              <div class="sidebar-item media-folder-item" v-for="f in galleryFolders" :key="f" :class="{ active: gallerySelectedFolder === f, 'is-faded': hiddenFolders.includes(f) }" @click="gallerySelectedFolder = f">
                  <span class="folder-name" :title="f">{{ f }}</span>
                  <button class="icon-text-btn folder-eye-btn" @click.stop="toggleHideFolder(f)" :title="t('hide_folder_hint')">[{{ hiddenFolders.includes(f) ? t('btn_show') : t('btn_hide') }}]</button>
              </div>
          </div>
      </aside>

      <main class="media-main">
          <div class="gallery-header">
              <h2>{{ t('images') }} <span style="color: var(--text-muted); font-weight: normal; font-size: 14px; margin-left: 5px;">› {{ gallerySelectedFolder === '' ? t('all_folders') : gallerySelectedFolder }}</span></h2>
              <div class="gallery-actions" style="display: flex; gap: 15px;">
                  <label class="toggle-hidden" style="margin: 0; align-items: center; display: flex;" v-if="hiddenImages.length > 0 || hiddenFolders.length > 0">
                    <input type="checkbox" v-model="showHiddenMedia">
                    {{ t('show_hidden') }}
                  </label>
                  <input type="text" v-model="gallerySearch" :placeholder="t('search_placeholder')" class="search-input" style="width: 250px; padding: 8px 15px;"/>
              </div>
          </div>

          <div v-if="isGalleryLoading" class="gallery-loading" style="padding: 30px; text-align: center;">
              <p>{{ t('loading_gallery') }}</p>
          </div>

          <div v-else class="gallery-scroll-container">
              <div class="gallery-grid">
                  <div class="gallery-card" v-for="img in paginatedGallery" :key="img.rel_path" :class="{ 'is-hidden': hiddenImages.includes(img.rel_path) }">
                      <div class="gallery-img-container" @click="importImageDialog(img)" :title="t('drop_here')">
                          <img :src="getImgSrc(img)" loading="lazy" class="gallery-img" />
                          <div class="gallery-img-overlay"><span>{{ t('drop_here') }}</span></div>
                          <div v-if="img.is_translated" class="status-badge status-done img-badge">{{ t('status_translated') }}</div>
                      </div>
                      <div class="gallery-card-info">
                          <div class="img-path" :title="img.rel_path">{{ img.rel_path }}</div>
                          <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
                              <button class="icon-text-btn" @click="openImgFolder(img.original_path)" style="margin-left: 0; padding-left: 0;">[{{ t('open_folder') }}]</button>
                              <button class="icon-text-btn" @click="toggleHideImage(img.rel_path)">[{{ hiddenImages.includes(img.rel_path) ? t('btn_show') : t('btn_hide') }}]</button>
                              <button v-if="img.is_translated" class="icon-text-btn" style="color: var(--error-text); padding-right: 0;" @click="revertImage(img)" :title="t('revert_img')">[{{ t('revert') }}]</button>
                          </div>
                      </div>
                  </div>
              </div>

              <div v-if="filteredGallery.length === 0" style="text-align: center; color: var(--text-muted); padding: 40px;">{{ t('no_images_found') }}</div>

              <div v-if="galleryTotalPages > 1" class="pagination-container">
                  <button class="btn btn-secondary" :disabled="galleryCurrentPage === 1" @click="galleryCurrentPage = 1">«</button>
                  <button class="btn btn-secondary" :disabled="galleryCurrentPage === 1" @click="galleryCurrentPage--">‹</button>
                  <span class="pagination-info">{{ t('page') }} <input type="number" v-model.lazy="galleryCurrentPage" min="1" :max="galleryTotalPages" class="page-input" @change="validateGalleryPage" /> {{ t('out_of') }} {{ galleryTotalPages }}</span>
                  <button class="btn btn-secondary" :disabled="galleryCurrentPage === galleryTotalPages" @click="galleryCurrentPage++">›</button>
                  <button class="btn btn-secondary" :disabled="galleryCurrentPage === galleryTotalPages" @click="galleryCurrentPage = galleryTotalPages">»</button>
              </div>
          </div>
      </main>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { t } from '../locales.js';
import { projectPath, targetLang, hiddenImages, hiddenFolders, showHiddenMedia, showMsg, getFolderFromPath } from '../store.js';

const galleryImages = ref([]);
const gallerySearch = ref('');
const gallerySelectedFolder = ref('');
const galleryCurrentPage = ref(1);
const galleryItemsPerPage = 100;
const isGalleryLoading = ref(false);

watch([gallerySearch, gallerySelectedFolder], () => { galleryCurrentPage.value = 1; });

onMounted(() => { loadGallery(); });

const galleryFolders = computed(() => {
    const folders = new Set();
    galleryImages.value.forEach(img => { folders.add(getFolderFromPath(img.rel_path)); });
    return Array.from(folders).sort().filter(f => f !== '/'); 
});

async function loadGallery() {
    if (!projectPath.value) return;
    isGalleryLoading.value = true;
    try {
        galleryImages.value = await invoke('get_images_list', { projectPath: projectPath.value, targetLang: targetLang.value });
    } catch(e) { showMsg('error', e.toString()); } 
    finally { isGalleryLoading.value = false; }
}

const filteredGallery = computed(() => {
    let result = galleryImages.value;
    if (!showHiddenMedia.value) {
        result = result.filter(img => !hiddenImages.value.includes(img.rel_path) && !hiddenFolders.value.includes(getFolderFromPath(img.rel_path)));
    }
    if (gallerySelectedFolder.value) {
        result = gallerySelectedFolder.value === '/' ? result.filter(img => !img.rel_path.includes('/')) : result.filter(img => getFolderFromPath(img.rel_path) === gallerySelectedFolder.value);
    }
    if (gallerySearch.value) result = result.filter(img => img.rel_path.toLowerCase().includes(gallerySearch.value.toLowerCase()));
    return result;
});

const galleryTotalPages = computed(() => Math.ceil(filteredGallery.value.length / galleryItemsPerPage) || 1);
const paginatedGallery = computed(() => {
    const start = (galleryCurrentPage.value - 1) * galleryItemsPerPage;
    return filteredGallery.value.slice(start, start + galleryItemsPerPage);
});

function validateGalleryPage() {
    let p = parseInt(galleryCurrentPage.value);
    if (isNaN(p) || p < 1) p = 1;
    if (p > galleryTotalPages.value) p = galleryTotalPages.value;
    galleryCurrentPage.value = p;
}

function getImgSrc(img) {
    const path = img.is_translated && img.translated_path ? img.translated_path : img.original_path;
    return convertFileSrc(path);
}

function toggleHideFolder(folder) {
  if (hiddenFolders.value.includes(folder)) hiddenFolders.value = hiddenFolders.value.filter(f => f !== folder);
  else hiddenFolders.value.push(folder);
}

function toggleHideImage(relPath) {
  if (hiddenImages.value.includes(relPath)) hiddenImages.value = hiddenImages.value.filter(p => p !== relPath);
  else hiddenImages.value.push(relPath);
}

async function importImageDialog(img) {
    try {
        const selected = await openDialog({ multiple: false, filters:[{ name: 'Images', extensions:['png', 'jpg', 'jpeg', 'webp'] }] });
        if (selected) {
            const translated_path = await invoke('import_localized_image', {
                projectPath: projectPath.value, targetLang: targetLang.value, relPath: img.rel_path, sourceFilePath: selected
            });
            img.is_translated = true; img.translated_path = translated_path;
            showMsg('success', t('img_copied'));
        }
    } catch(e) { showMsg('error', e.toString()); }
}

async function revertImage(img) {
    if (!confirm(t('confirm_revert'))) return;
    try {
        await invoke('delete_localized_image', { projectPath: projectPath.value, targetLang: targetLang.value, relPath: img.rel_path });
        img.is_translated = false; img.translated_path = null;
        showMsg('success', t('img_reverted'));
    } catch(e) { showMsg('error', e.toString()); }
}

async function openImgFolder(path) {
    try { await invoke('open_in_explorer', { path }); } catch(e) { showMsg('error', e.toString()); }
}
</script>