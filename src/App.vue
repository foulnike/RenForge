<template>
  <div class="app-container" :data-theme="uiTheme">
    <Header />
    <div v-if="activePopover" class="dropdown-overlay-bg" @click="activePopover = null"></div>
    <GlobalMessages />

    <Dashboard v-if="currentMode === 'dashboard'" />
    <Editor v-if="currentMode === 'editor'" />
    <FallbackEditor v-if="currentMode === 'fallback-editor'" />
    <ImageGallery v-if="currentMode === 'gallery'" />
    <AudioGallery v-if="currentMode === 'audio'" />

    <AiModal v-if="isAiModalOpen" />
  </div>
</template>

<script setup>
import { onMounted, onUnmounted } from 'vue';
import { uiTheme, currentMode, activePopover, isAiModalOpen } from './store.js';

import Header from './components/Header.vue';
import GlobalMessages from './components/GlobalMessages.vue';
import AiModal from './components/AiModal.vue';
import Dashboard from './components/Dashboard.vue';
import Editor from './components/Editor.vue';
import FallbackEditor from './components/FallbackEditor.vue';
import ImageGallery from './components/ImageGallery.vue';
import AudioGallery from './components/AudioGallery.vue';

function handleContextMenu(e) {
  const target = e.target;
  const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable || target.closest('.raw-code') !== null;
  const hasSelection = window.getSelection().toString().length > 0;
  if (!isInput && !hasSelection) e.preventDefault();
}

onMounted(() => { window.addEventListener('contextmenu', handleContextMenu); });
onUnmounted(() => { window.removeEventListener('contextmenu', handleContextMenu); });
</script>

<style>
/* Здесь подключаем стили из 4 пункта: */
@import './assets/style.css';
</style>