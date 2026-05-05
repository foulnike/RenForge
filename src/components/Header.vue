<template>
  <header class="header">
    <!-- LEFT SECTION -->
    <div class="header-section">
      <template v-if="['dashboard', 'gallery', 'audio'].includes(currentMode)">
        <div class="logo" style="margin-right: 15px;">RenForge <span class="version">v1.1</span></div>
        
        <div class="popover-wrapper">
          <button class="btn btn-secondary" @click="togglePopover('settings')" :class="{active: activePopover === 'settings'}">{{ t('settings') }}</button>
          <div v-if="activePopover === 'settings'" class="popover-menu">
            <div class="setting-row">
                <label>{{ t('ui_theme') }}</label>
                <select class="settings-select" v-model="uiTheme" @change="saveSettings">
                    <option value="dark">{{ t('theme_dark') }}</option>
                    <option value="black">{{ t('theme_black') }}</option>
                    <option value="light">{{ t('theme_light') }}</option>
                </select>
            </div>
            <div class="setting-row">
                <label>{{ t('ui_lang') }}</label>
                <select class="settings-select" v-model="uiLang" @change="saveSettings">
                    <option value="ru">RU</option>
                    <option value="en">EN</option>
                </select>
            </div>
            <div class="setting-row">
                <label>{{ t('target_lang') }}</label>
                <div style="display:flex; flex-direction:column; gap:8px; align-items:flex-end;">
                  <select class="settings-select" v-model="targetLangSelect" @change="onTargetLangSelect">
                      <option value="russian">Русский (russian)</option>
                      <option value="english">English (english)</option>
                      <option value="spanish">Español (spanish)</option>
                      <option value="french">Français (french)</option>
                      <option value="german">Deutsch (german)</option>
                      <option value="custom">{{ t('custom_lang') }}</option>
                  </select>
                  <input v-if="targetLangSelect === 'custom'" type="text" class="settings-select" v-model="targetLang" @change="saveSettings" :placeholder="t('custom_lang_placeholder')" style="width: 100%; box-sizing: border-box;" />
                </div>
            </div>
          </div>
        </div>
        
        <div class="popover-wrapper">
          <button class="btn btn-secondary" @click="togglePopover('help')" :class="{active: activePopover === 'help'}">{{ t('help_title') }}</button>
          <div v-if="activePopover === 'help'" class="popover-menu" style="width: 450px;">
            <template v-if="currentMode === 'dashboard'">
              <ol style="margin: 0; padding-left: 15px; line-height: 1.6; font-size: 13px;">
                <li style="margin-bottom: 10px;" v-html="t('help_step1')"></li>
                <li style="margin-bottom: 10px;" v-html="t('help_step2')"></li>
                <li style="margin-bottom: 10px;" v-html="t('help_step3')"></li>
                <li style="margin-bottom: 10px;" v-html="t('help_step4')"></li>
                <li v-html="t('help_step5')"></li>
              </ol>
            </template>
            <template v-else-if="currentMode === 'gallery'">
              <p style="font-size: 13px; line-height: 1.5; margin: 0;">{{ t('help_img_desc') }}</p>
            </template>
            <template v-else-if="currentMode === 'audio'">
              <p style="font-size: 13px; line-height: 1.5; margin: 0;">{{ t('help_audio_desc') }}</p>
            </template>
          </div>
        </div>
      </template>

      <template v-else-if="['editor', 'fallback-editor'].includes(currentMode)">
        <button class="btn btn-secondary" @click="closeEditor">{{ t('back') }}</button>
        <span class="header-filename" :title="currentMode === 'editor' ? currentFilePath : fallbackRelPath">
          {{ getFileName(currentMode === 'editor' ? currentFilePath : fallbackRelPath) }}
        </span>
      </template>
    </div>

    <!-- CENTER SECTION -->
    <div class="header-section center">
      <template v-if="['dashboard', 'gallery', 'audio'].includes(currentMode)">
        <div class="segmented-control">
          <button :class="['seg-btn', { active: currentMode === 'dashboard' }]" @click="currentMode = 'dashboard'">{{ t('text') }}</button>
          <button :class="['seg-btn', { active: currentMode === 'gallery' }]" @click="currentMode = 'gallery'">{{ t('images') }}</button>
          <button :class="['seg-btn', { active: currentMode === 'audio' }]" @click="currentMode = 'audio'">{{ t('audio') }}</button>
        </div>
      </template>

      <template v-else-if="currentMode === 'editor'">
        <div class="btn-group" v-if="parsedBlocks.length > 0 && !showRawView && !isEditorLoading">
          <button class="group-btn" @click="isAiModalOpen = true">{{ t('ai_assistant') }}</button>
          
          <div class="popover-wrapper" style="display: inline-flex;">
            <button class="group-btn" @click="togglePopover('export')" :class="{active: activePopover === 'export'}">{{ t('export_btn') }} ▾</button>
            <div v-if="activePopover === 'export'" class="popover-menu popover-menu-sm">
              <button class="dropdown-item-btn" @click="exportCSV">Export .CSV</button>
              <button class="dropdown-item-btn" @click="exportJSON">Export .JSON</button>
            </div>
          </div>

          <div class="popover-wrapper" style="display: inline-flex;">
            <button class="group-btn" @click="togglePopover('import')" :class="{active: activePopover === 'import'}">{{ t('import_btn') }} ▾</button>
            <div v-if="activePopover === 'import'" class="popover-menu popover-menu-sm">
              <button class="dropdown-item-btn" @click="importCSV">Import .CSV</button>
              <button class="dropdown-item-btn" @click="importJSON">Import .JSON</button>
            </div>
          </div>

          <button :class="['group-btn', { active: showRawView }]" @click="showRawView = !showRawView">{{ t('raw_code') }}</button>
        </div>
      </template>
    </div>

    <!-- RIGHT SECTION -->
    <div class="header-section right">
      <template v-if="['dashboard', 'gallery', 'audio'].includes(currentMode)">
        <label class="toggle-hidden" v-if="projectPath && hiddenFiles.length > 0 && currentMode === 'dashboard'">
          <input type="checkbox" v-model="showHidden">
          {{ t('show_hidden') }} ({{ hiddenFiles.length }})
        </label>
        <button class="btn btn-primary" @click="openProjectFolder">{{ t('select_folder') }}</button>
      </template>
      
      <template v-else-if="currentMode === 'editor'">
        <button v-if="hasErrors && !showRawView && !isEditorLoading" class="btn btn-outline error-jump-btn" @click="jumpToNextError" :title="t('next_error')">{{ t('next_error') }}</button>
        
        <div class="popover-wrapper" v-if="!showRawView && !isEditorLoading">
          <button class="btn btn-secondary" @click="togglePopover('settings')" :class="{active: activePopover === 'settings'}">{{ t('settings') }}</button>
          <div v-if="activePopover === 'settings'" class="popover-menu popover-right">
            <div class="setting-row">
                <label>{{ t('ui_theme') }}</label>
                <select class="settings-select" v-model="uiTheme" @change="saveSettings">
                    <option value="dark">{{ t('theme_dark') }}</option>
                    <option value="black">{{ t('theme_black') }}</option>
                    <option value="light">{{ t('theme_light') }}</option>
                </select>
            </div>
            <div class="setting-row">
                <label>{{ t('ui_lang') }}</label>
                <select class="settings-select" v-model="uiLang" @change="saveSettings">
                    <option value="ru">RU</option>
                    <option value="en">EN</option>
                </select>
            </div>
          </div>
        </div>
        
        <div class="popover-wrapper" v-if="!showRawView && !isEditorLoading">
          <button class="btn btn-secondary" @click="togglePopover('help')" :class="{active: activePopover === 'help'}">{{ t('help_title') }}</button>
          <div v-if="activePopover === 'help'" class="popover-menu popover-right" style="width: 450px;">
            <ol style="margin: 0; padding-left: 15px; line-height: 1.6; font-size: 13px;">
              <li style="margin-bottom: 10px;" v-html="t('help_editor_1')"></li>
              <li style="margin-bottom: 10px;" v-html="t('help_editor_2')"></li>
              <li v-html="t('help_editor_3')"></li>
            </ol>
          </div>
        </div>

        <div style="display: flex; align-items: center; gap: 15px; margin-left: 5px;" v-if="!isEditorLoading">
          <button v-if="!showRawView && parsedBlocks.length > 0" class="btn btn-primary" @click="saveFile">{{ t('save') }}</button>
        </div>
      </template>

      <template v-else-if="currentMode === 'fallback-editor'">
        <div class="btn-group" style="margin-right: 15px;">
          <button class="group-btn" @click="autoSelectFallback">{{ t('fallback_auto_btn') }}</button>
          <button class="group-btn" @click="selectAllFallback">{{ t('select_all') }}</button>
          <button class="group-btn" @click="clearAllFallback">{{ t('clear_all') }}</button>
          
          <div class="popover-wrapper" style="display: inline-flex;">
            <button class="group-btn" @click="togglePopover('help')" :class="{active: activePopover === 'help'}">{{ t('help_title') }} ▾</button>
            <div v-if="activePopover === 'help'" class="popover-menu popover-right" style="width: 400px; text-align: left; white-space: normal;">
              <strong>{{ t('fallback_help_title') }}</strong>
              <p style="font-size: 13px; color: var(--text-secondary); line-height: 1.5; margin: 5px 0;">{{ t('fallback_help_1') }}</p>
              <p style="font-size: 13px; color: var(--text-secondary); line-height: 1.5; margin: 5px 0;">{{ t('fallback_help_2') }}</p>
              <p style="font-size: 13px; color: var(--error-text); line-height: 1.5; margin: 5px 0; font-weight: bold;">{{ t('fallback_help_3') }}</p>
              <hr style="border: none; border-top: 1px solid var(--border-main); margin: 8px 0;" />
              <p style="font-size: 12px; color: var(--text-muted); line-height: 1.4; margin: 0;">{{ t('fallback_help_4') }}</p>
            </div>
          </div>
        </div>
        <button class="btn btn-primary" @click="generateFallbackFile" :disabled="isProcessing">{{ t('fallback_gen_btn') }}</button>
      </template>
    </div>
  </header>
</template>

<script setup>
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { t } from '../locales.js';
import { 
  uiTheme, uiLang, targetLang, currentMode, activePopover, isAiModalOpen,
  parsedBlocks, showRawView, isEditorLoading, currentFilePath, fallbackRelPath,
  projectPath, hiddenFiles, isProcessing, loadProjectSettings, getFileName, showMsg, fallbackLines, showHidden
} from '../store.js';
import { refreshProject, exportCSV, exportJSON, importCSV, importJSON, saveFile, getBlockStatus } from '../actions.js';

const predefinedLangs =['russian', 'english', 'spanish', 'french', 'german'];
const targetLangSelect = ref(predefinedLangs.includes(targetLang.value) ? targetLang.value : 'custom');

const hasErrors = computed(() => parsedBlocks.value.some(block => getBlockStatus(block) === 'error'));

function togglePopover(name) { activePopover.value = activePopover.value === name ? null : name; }

function saveSettings() {
  localStorage.setItem('renforge_ui_lang', uiLang.value);
  localStorage.setItem('renforge_target_lang', targetLang.value);
  localStorage.setItem('renforge_ui_theme', uiTheme.value);
}

function onTargetLangSelect() {
  if (targetLangSelect.value !== 'custom') targetLang.value = targetLangSelect.value;
  else if (predefinedLangs.includes(targetLang.value)) targetLang.value = '';
  saveSettings();
  if (projectPath.value) refreshProject();
}

async function openProjectFolder() {
  try {
    const selectedPath = await open({ multiple: false, directory: true });
    if (!selectedPath) return; 
    projectPath.value = selectedPath; 
    loadProjectSettings(); 
    await refreshProject();
  } catch (e) { showMsg('error', `Error: ${e}`); }
}

function closeEditor() {
    currentMode.value = 'dashboard';
    showRawView.value = false;
    currentFilePath.value = '';
    parsedBlocks.value =[];
}

function jumpToNextError() {
    const errBlock = parsedBlocks.value.find(b => getBlockStatus(b) === 'error');
    if (errBlock) {
        const el = document.getElementById('block-' + errBlock.id); 
        if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' }); 
    }
}

// Fallback logic
function autoSelectFallback() {
    if (confirm(t('fallback_auto_warn'))) {
        fallbackLines.value.forEach(line => {
            line.parts.forEach(part => { if (part.type === 'string' && part.canAuto) part.selected = true; });
        });
    }
}
function selectAllFallback() {
    fallbackLines.value.forEach(line => {
        line.parts.forEach(part => { if (part.type === 'string' && (part.canAuto || part.suspicious)) part.selected = true; });
    });
}
function clearAllFallback() {
    fallbackLines.value.forEach(line => {
        line.parts.forEach(part => { if (part.type === 'string') part.selected = false; });
    });
}

async function generateFallbackFile() {
    const selectedSet = new Set();
    fallbackLines.value.forEach(line => {
        line.parts.forEach(part => { if (part.type === 'string' && part.selected && part.fullRaw) selectedSet.add(part.fullRaw); });
    });
    
    try {
        isProcessing.value = true;
        await invoke('write_fallback_file', {
            projectPath: projectPath.value, targetLang: targetLang.value,
            origRelPath: fallbackRelPath.value, strings: Array.from(selectedSet)
        });
        showMsg('success', t('msg_fallback_saved'));
        currentMode.value = 'dashboard';
        await refreshProject();
    } catch(e) { showMsg('error', e.toString()); } 
    finally { isProcessing.value = false; }
}
</script>