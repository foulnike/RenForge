<template>
  <div class="app-container" :data-theme="uiTheme">
    <!-- === HEADER === -->
    <header class="header">
      <div class="logo">RenForge <span class="version">v1.0</span></div>
      
      <div class="actions">
        <button class="info-btn" @click="isSettingsOpen = true">[ {{ t('settings') }} ]</button>
        <button class="info-btn" @click="showHelp = true" :title="t('help_title')">[ ? ]</button>

        <template v-if="currentMode === 'dashboard' || currentMode === 'gallery'">
          <label class="toggle-hidden" v-if="hiddenFiles.length > 0 && currentMode === 'dashboard'">
            <input type="checkbox" v-model="showHidden">
            {{ t('show_hidden') }} ({{ hiddenFiles.length }})
          </label>
          <button class="btn btn-secondary" style="margin-right: 15px;" v-if="projectPath && currentMode !== 'gallery'" @click="openGallery">{{ t('gallery') }}</button>
          <button class="btn btn-primary" @click="openProjectFolder">{{ t('select_folder') }}</button>
        </template>
        
        <template v-else-if="currentMode === 'editor'">
          <div v-if="autosaveEnabled && lastSavedTime" class="autosave-indicator">
            {{ t('saved_at') }} {{ lastSavedTime }}
          </div>

          <label class="toggle-hidden" style="margin-right: 10px;">
            <input type="checkbox" v-model="hideTranslated" />
            {{ t('hide_translated') }}
          </label>

          <button class="btn btn-secondary" @click="openAiModal" v-if="!showRawView && parsedBlocks.length > 0">
            {{ t('ai_assistant') }}
          </button>
          <div class="csv-group" v-if="!showRawView && parsedBlocks.length > 0">
              <button class="btn btn-outline" @click="exportCSV" :title="t('export_csv')">{{ t('export_csv') }}</button>
              <button class="btn btn-outline" @click="importCSV" :title="t('import_csv')">{{ t('import_csv') }}</button>
          </div>
          
          <button class="btn btn-outline" @click="closeEditor">{{ t('back') }}</button>
          <button class="btn btn-outline" @click="showRawView = !showRawView">
            {{ showRawView ? t('visual_editor') : t('raw_code') }}
          </button>
          <button v-if="!showRawView && parsedBlocks.length > 0" class="btn btn-primary" @click="saveFile(false)" :disabled="hasErrors">{{ t('save') }}</button>
        </template>
      </div>
    </header>
    
    <div v-if="error" class="global-msg error-msg">
      <span style="white-space: pre-wrap;">{{ error }}</span>
      <button class="msg-close-btn" @click="closeMsg">×</button>
    </div>
    <div v-if="successMsg" class="global-msg success-msg">
      <span>{{ successMsg }}</span>
      <button class="msg-close-btn" @click="closeMsg">×</button>
    </div>

    <!-- === DASHBOARD === -->
    <div class="dashboard" v-if="currentMode === 'dashboard'">
      <div v-if="!projectPath" class="empty-state">
        <h2>{{ t('empty_state_title') }}</h2>
        <p>{{ t('empty_state_desc1') }}</p>
        <p class="empty-hint">{{ t('empty_state_desc2') }}</p>
      </div>
      
      <div v-else class="dashboard-content">
        <!-- SEARCH -->
        <div class="search-section">
            <div style="display: flex; gap: 10px;">
                <input type="text" v-model="searchQuery" @input="handleSearch" :placeholder="t('search_placeholder')" class="search-input" />
            </div>
            <div v-if="searchResults.length > 0" class="search-results">
                <div v-for="res in searchResults" :key="res.id" class="search-res-item" @click="jumpToFile(res)">
                    <span class="res-file">{{ getFileName(res.file_path) }}</span>
                    <span class="res-text"><b>{{ res.original }}</b></span>
                    <span class="res-tran">{{ res.translation || '...' }}</span>
                </div>
            </div>
        </div>
        
        <!-- TRANSLATION GENERATION BANNER -->
        <div class="translation-banner" v-if="translationFiles.length === 0">
            <div class="banner-content">
                <h3>{{ t('no_translation_found') }}</h3>
                <p v-if="originalRpyFiles.length === 0">{{ t('unpack_first') }}</p>
                <p v-else>{{ t('ready_to_gen') }}</p>
            </div>
            <button class="btn btn-primary generate-btn" :disabled="originalRpyFiles.length === 0 || isProcessing" @click="generateTranslations">
                {{ t('gen_btn') }}
            </button>
        </div>

        <div class="dashboard-grid">
          <!-- Archives -->
          <div class="file-column">
            <div class="column-header column-header-flex">
              <span>{{ t('col_archives') }}</span>
              <button v-if="filteredFiles.rpa.length > 0" class="action-btn bulk-btn" @click="extractAllRpa" :disabled="isProcessing">{{ t('unpack_all') }}</button>
            </div>
            <div class="column-content">
              <div class="file-item" v-for="file in filteredFiles.rpa" :key="file" :class="{ 'is-hidden': hiddenFiles.includes(file) }">
                <div class="file-info">
                  <span class="file-name">{{ getFileName(file) }}</span>
                  <span class="file-path">{{ getRelativePath(file) }}</span>
                </div>
                <button class="icon-text-btn" @click="toggleHide(file)">{{ hiddenFiles.includes(file) ? t('btn_show') : t('btn_hide') }}</button>
              </div>
              <div v-if="filteredFiles.rpa.length === 0" class="no-files">{{ t('no_archives') }}</div>
            </div>
          </div>

          <!-- Scripts -->
          <div class="file-column">
            <div class="column-header column-header-flex">
              <span>{{ t('col_scripts') }}</span>
              <button v-if="filteredFiles.rpyc.length > 0" class="action-btn bulk-btn" @click="decompileAllRpyc" :disabled="isProcessing">{{ t('decompile_all') }}</button>
            </div>
            <div class="column-content">
              <div class="file-item" v-for="file in filteredFiles.rpyc" :key="file" :class="{ 'is-hidden': hiddenFiles.includes(file) }">
                <div class="file-info">
                  <span class="file-name">{{ getFileName(file) }}</span>
                  <span class="file-path">{{ getRelativePath(file) }}</span>
                </div>
                <div style="display:flex; align-items:center; gap: 8px;">
                  <span v-if="isAlreadyDecompiled(file)" class="status-badge status-done">{{ t('status_done') }}</span>
                  <button class="icon-text-btn" @click="toggleHide(file)">{{ hiddenFiles.includes(file) ? t('btn_show') : t('btn_hide') }}</button>
                </div>
              </div>
              <div v-if="filteredFiles.rpyc.length === 0" class="no-files">{{ t('no_scripts') }}</div>
            </div>
          </div>

          <!-- Translations -->
          <div class="file-column tl-column">
            <div class="column-header column-header-flex tl-header">
              <span>{{ t('col_translations') }} (tl/{{ targetLang }})</span>
              <div v-if="translationFiles.length > 0" style="display:flex; align-items:center; gap:10px;">
                <label style="font-size: 11px; cursor: pointer; display: flex; align-items: center; gap: 4px;" :title="t('fix_squares_hint')">
                  <input type="checkbox" v-model="forceFontFix" style="margin:0;">
                  {{ t('fix_squares') }}
                </label>
                <button class="action-btn apply-patch-btn" @click="applyPatch" :disabled="isProcessing">
                  {{ t('apply_patch') }}
                </button>
              </div>
            </div>
            <div class="column-content">
              <div class="file-item rpy-item" v-for="file in translationFiles" :key="file" :class="{ 'is-hidden': hiddenFiles.includes(file), 'is-completed': completedFiles.includes(file) }">
                <input type="checkbox" :checked="completedFiles.includes(file)" @change="toggleFileCompleted(file)" class="done-checkbox"/>
                <div class="file-info">
                  <span class="file-name">{{ getFileName(file) }}</span>
                  <div class="file-stats-wrapper" v-if="fileStats[file]">
                    <span class="file-path">{{ fileStats[file].translated }} / {{ fileStats[file].total }}</span>
                    <div class="progress-bar-bg">
                      <div class="progress-bar-fill" :style="{ width: (fileStats[file].translated / fileStats[file].total * 100) + '%' }"></div>
                    </div>
                  </div>
                  <span v-else class="file-path">{{ getRelativePath(file) }}</span>
                </div>
                <div class="file-actions">
                  <button class="icon-text-btn" @click="toggleHide(file)">{{ hiddenFiles.includes(file) ? t('btn_show') : t('btn_hide') }}</button>
                  <button class="action-btn edit-btn" @click="openEditor(file)" :disabled="isProcessing">{{ t('btn_translate') }}</button>
                </div>
              </div>
              <div v-if="translationFiles.length === 0" class="no-files">{{ t('no_translations') }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- === GALLERY (IMAGES MANAGER) === -->
    <div class="gallery-workspace" v-if="currentMode === 'gallery'">
        <div class="gallery-header">
            <h2>{{ t('gallery') }}</h2>
            <div class="gallery-actions" style="display: flex; gap: 15px;">
                <input type="text" v-model="gallerySearch" :placeholder="t('search_placeholder')" class="search-input" style="width: 300px; padding: 8px 15px;"/>
                <button class="btn btn-outline" @click="currentMode = 'dashboard'">{{ t('back') }}</button>
            </div>
        </div>

        <div v-if="isGalleryLoading" class="gallery-loading" style="padding: 30px; text-align: center;">
            <p>{{ t('loading_gallery') }}</p>
        </div>

        <div v-else class="gallery-scroll-container">
            <div class="gallery-grid">
                <div class="gallery-card" v-for="img in paginatedGallery" :key="img.rel_path">
                    <div class="gallery-img-container" 
                         @click="importImageDialog(img)" 
                         :title="t('drop_here')">
                        <img :src="getImgSrc(img)" loading="lazy" class="gallery-img" />
                        <div class="gallery-img-overlay">
                            <span>{{ t('drop_here') }}</span>
                        </div>
                        <div v-if="img.is_translated" class="status-badge status-done img-badge">{{ t('status_translated') }}</div>
                    </div>
                    <div class="gallery-card-info">
                        <div class="img-path">{{ img.rel_path }}</div>
                        <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
                            <button class="icon-text-btn" @click="openImgFolder(img.original_path)" style="margin-left: 0; padding-left: 0;">[{{ t('open_folder') }}]</button>
                            <button v-if="img.is_translated" class="icon-text-btn" style="color: var(--error-text); padding-right: 0;" @click="revertImage(img)" :title="t('revert_img')">[{{ t('revert') }}]</button>
                        </div>
                    </div>
                </div>
            </div>

            <div v-if="filteredGallery.length === 0" style="text-align: center; color: var(--text-muted); padding: 40px;">
                {{ t('no_images_found') }}
            </div>

            <div v-if="paginatedGallery.length < filteredGallery.length" class="load-more-container">
                <button class="btn btn-secondary" @click="loadMoreImages">
                    Загрузить ещё (Показано {{ paginatedGallery.length }} из {{ filteredGallery.length }})
                </button>
            </div>
        </div>
    </div>

    <!-- === EDITOR === -->
    <div class="workspace" v-if="currentMode === 'editor'">
      <div v-if="showRawView || parsedBlocks.length === 0" class="raw-preview">
        <div class="raw-preview-header">
          <h3 v-if="parsedBlocks.length === 0">{{ t('raw_no_blocks') }}</h3>
          <h3 v-else>{{ t('raw_view_mode') }}</h3>
        </div>
        <pre class="raw-code">{{ rawFileText }}</pre>
      </div>

      <template v-else>
        <aside class="sidebar">
          <div class="sidebar-title">{{ t('file_structure') }}</div>
          <div class="sidebar-list">
            <div class="sidebar-item" v-for="(block, index) in parsedBlocks" :key="'nav-' + block.id" @click="scrollToBlock(block.id)">
              <span class="status-dot" :class="getBlockStatus(block)"></span>
              <span class="sidebar-index">{{ index + 1 }}</span>
              <span class="sidebar-id">{{ block.id }}</span>
            </div>
          </div>
        </aside>

        <main class="editor-panel">
          <div class="live-reload-hint">
            <span>[ INFO ]</span>
            <span>{{ t('live_reload_hint') }}</span>
          </div>

          <div class="translation-block" 
               v-for="(block, index) in parsedBlocks" 
               :key="block.id" 
               :id="'block-' + block.id" 
               :class="['status-' + getBlockStatus(block)]"
               v-show="!hideTranslated || getBlockStatus(block) !== 'translated' || focusedBlockId === block.id">
            <div class="block-header">
              <span class="block-id">{{ t('line_num') }} #{{ index + 1 }} | ID: {{ block.id }}</span>
            </div>
            
            <div class="original-text">
              <span v-if="block.prefix" class="char-prefix original-prefix">
                <span class="char-mapping-name" v-if="charMap[block.prefix.trim()]">
                  {{ charMap[block.prefix.trim()] }}
                </span>
                <span v-else>{{ block.prefix.trim() }}</span>
                <span v-if="charMap[block.prefix.trim()]" class="char-raw">({{ block.prefix.trim() }})</span>
              </span>
              <span v-html="highlightGlossary(block.original)"></span>
            </div>

            <div class="fake-input-wrapper">
              <div v-if="block.prefix" class="char-prefix translated-prefix">{{ block.prefix.trim() }}</div>
              <input type="text" class="transparent-input" 
                     v-model="block.translation" 
                     :placeholder="t('input_placeholder')"
                     @focus="focusedBlockId = block.id"
                     @blur="focusedBlockId = null" />
            </div>
            <div class="tag-error" v-if="getMissingTags(block).length > 0">
              <strong>{{ t('tag_error') }}</strong>
              <span class="missing-tag" v-for="tag in getMissingTags(block)" :key="tag">{{ tag }}</span>
            </div>
          </div>
        </main>

        <aside class="assistant-sidebar">
          <div class="sidebar-title glossary-title">{{ t('glossary') }}</div>
          <div class="glossary-content">
            <div class="glossary-add-form">
              <input type="text" v-model="newTerm.original" :placeholder="t('glos_orig')" />
              <input type="text" v-model="newTerm.translation" :placeholder="t('glos_tran')" @keyup.enter="addGlossaryTerm" />
              <button class="btn btn-primary" @click="addGlossaryTerm">{{ t('glos_add') }}</button>
            </div>
            
            <div class="glossary-list">
              <div class="glossary-card" v-for="(term, i) in glossary" :key="i">
                <div class="glos-terms">
                  <div class="glos-original">{{ term.original }}</div>
                  <div class="glos-translation">{{ term.translation }}</div>
                </div>
                <button class="glos-del-btn" @click="removeGlossaryTerm(i)">[ DEL ]</button>
              </div>
            </div>
          </div>
        </aside>
      </template>
    </div>

    <!-- === SETTINGS MODAL === -->
    <div v-if="isSettingsOpen" class="modal-overlay" @click.self="isSettingsOpen = false">
      <div class="modal-content" style="max-width: 400px;">
        <div class="modal-header">
            <h2>{{ t('settings') }}</h2>
            <button class="close-btn" @click="isSettingsOpen = false">{{ t('close') }}</button>
        </div>
        <div class="modal-body">
            <div class="setting-row">
                <label>{{ t('ui_theme') }}</label>
                <select class="settings-select" v-model="uiTheme" @change="saveSettings">
                    <option value="dark">{{ t('theme_dark') }}</option>
                    <option value="light">{{ t('theme_light') }}</option>
                </select>
            </div>
            <div class="setting-row">
                <label>{{ t('autosave') }}</label>
                <select class="settings-select" v-model="autosaveEnabled" @change="handleAutosaveToggle">
                    <option :value="true">{{ t('on') }}</option>
                    <option :value="false">{{ t('off') }}</option>
                </select>
            </div>
            <div class="setting-row">
                <label>{{ t('ui_lang') }}</label>
                <select class="settings-select" v-model="uiLang" @change="saveSettings">
                    <option value="ru">RU</option>
                    <option value="en">EN</option>
                </select>
            </div>
            <div class="setting-row" v-if="currentMode !== 'editor'">
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
                  <input v-if="targetLangSelect === 'custom'" 
                         type="text" 
                         class="settings-select" 
                         v-model="targetLang" 
                         @change="saveSettings" 
                         :placeholder="t('custom_lang_placeholder')" 
                         style="width: 100%; box-sizing: border-box;" />
                </div>
            </div>
        </div>
      </div>
    </div>

    <!-- === AI MODAL === -->
    <div v-if="isAiModalOpen" class="modal-overlay" @click.self="isAiModalOpen = false">
        <div class="modal-content">
            <div class="modal-header">
                <h2>{{ t('modal_ai_title') }}</h2>
                <button class="close-btn" @click="isAiModalOpen = false">{{ t('close') }}</button>
            </div>
            
            <div class="modal-body">
                <div class="tabs-header">
                    <button :class="['tab-btn', { active: aiTab === 'ollama' }]" @click="aiTab = 'ollama'">{{ t('ai_tabs_local') }}</button>
                    <button :class="['tab-btn', { active: aiTab === 'manual' }]" @click="aiTab = 'manual'">{{ t('ai_tabs_manual') }}</button>
                </div>

                <div class="step-box">
                    <h3>{{ t('ai_lines_range') }}</h3>
                    <div class="turbo-controls">
                        {{ t('from') }} <input type="number" v-model="aiStart" min="1" :max="parsedBlocks.length" />
                        {{ t('to') }} <input type="number" v-model="aiEnd" min="1" :max="parsedBlocks.length" />
                    </div>
                </div>

                <template v-if="aiTab === 'manual'">
                    <div class="step-box">
                        <h3>{{ t('step_1') }}</h3>
                        <p>{{ t('step_1_desc') }}</p>
                        <button class="btn btn-primary" @click="prepareAiBatch(false)">{{ t('copy_ai') }}</button>
                    </div>
                    <div class="step-box">
                        <h3>{{ t('step_2') }}</h3>
                        <p>{{ t('step_2_desc') }}</p>
                        <textarea v-model="aiInput" placeholder="1. Hello, world!&#10;2. How are you?"></textarea>
                        <button class="btn btn-primary apply-ai-btn" @click="importAiBatch" :disabled="!aiInput.trim()">
                            {{ t('apply_ai') }}
                        </button>
                    </div>
                </template>

                <template v-if="aiTab === 'ollama'">
                    <div class="step-box">
                        <h3>{{ t('ai_ollama_settings') }}</h3>
                        <div class="turbo-controls">
                            URL: <input type="text" v-model="ollamaUrl" style="width: 180px;" />
                            Model: <input type="text" v-model="ollamaModel" style="width: 120px;" />
                        </div>
                    </div>
                    <div class="step-box">
                        <h3>{{ t('ai_action') }}</h3>
                        <button class="btn btn-primary apply-ai-btn" @click="runLocalLLM" :disabled="isOllamaTranslating">
                            {{ isOllamaTranslating ? t('ai_processing') : t('ai_translate_btn') }}
                        </button>
                    </div>
                </template>
            </div>
        </div>
    </div>

    <!-- === HELP MODAL === -->
    <div v-if="showHelp" class="modal-overlay" @click.self="showHelp = false">
      <div class="modal-content" style="max-width: 550px;">
        <div class="modal-header">
            <h2>{{ t('help_title') }}</h2>
            <button class="close-btn" @click="showHelp = false">{{ t('close') }}</button>
        </div>
        <div class="modal-body" style="line-height: 1.6; font-size: 14px;">
          <ol style="margin-top: 0; padding-left: 20px;">
            <li style="margin-bottom: 10px;" v-html="t('help_step1')"></li>
            <li style="margin-bottom: 10px;" v-html="t('help_step2')"></li>
            <li style="margin-bottom: 10px;" v-html="t('help_step3')"></li>
            <li style="margin-bottom: 10px;" v-html="t('help_step4')"></li>
            <li v-html="t('help_step5')"></li>
          </ol>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

// ============================================================================
// Core State & Settings Management
// ============================================================================

const uiLang = ref(localStorage.getItem('renforge_ui_lang') || 'ru');
const targetLang = ref(localStorage.getItem('renforge_target_lang') || 'russian');
const uiTheme = ref(localStorage.getItem('renforge_ui_theme') || 'dark');
const autosaveEnabled = ref(localStorage.getItem('renforge_autosave') !== 'false');

const predefinedLangs =['russian', 'english', 'spanish', 'french', 'german'];
const targetLangSelect = ref(predefinedLangs.includes(targetLang.value) ? targetLang.value : 'custom');

const isSettingsOpen = ref(false);

/**
 * Handles target language selection changes.
 */
function onTargetLangSelect() {
  if (targetLangSelect.value !== 'custom') {
    targetLang.value = targetLangSelect.value;
  } else if (predefinedLangs.includes(targetLang.value)) {
    targetLang.value = '';
  }
  saveSettings();
}

/**
 * Persists user configuration to localStorage.
 */
function saveSettings() {
  localStorage.setItem('renforge_ui_lang', uiLang.value);
  localStorage.setItem('renforge_target_lang', targetLang.value);
  localStorage.setItem('renforge_ui_theme', uiTheme.value);
}

/**
 * Handles toggling of the autosave feature.
 */
function handleAutosaveToggle() {
  localStorage.setItem('renforge_autosave', autosaveEnabled.value);
  setupAutosave();
}

const locales = {
  ru: {
    settings: "Настройки",
    ui_theme: "Тема",
    theme_light: "Светлая",
    theme_dark: "Тёмная",
    autosave: "Автосохранение",
    on: "Вкл",
    off: "Выкл",
    close: "Закрыть",
    ui_lang: "Интерфейс",
    target_lang: "Перевод на",
    custom_lang: "Свой вариант (ввод)...",
    custom_lang_placeholder: "Напр., ukrainian",
    help_title: "Инструкция",
    help_step1: "Укажите корневую папку игры (там, где находится .exe файл).",
    help_step2: "В левой и центральной колонках нажмите <i>Распаковать всё</i> и <i>Вскрыть всё</i>.",
    help_step3: "Если оригиналы вскрыты, нажмите синюю кнопку <i>Сгенерировать переводы</i>.",
    help_step4: "В правой колонке нажмите <b>Внедрить патч</b>. Это заставит игру сразу запускаться с выбранным языком перевода.",
    help_step5: "Открывайте файлы, переводите строки и жмите <b>Сохранить</b>. Если в этот момент запущена игра, нажмите <b>Shift+R</b> в её окне — текст обновится без перезапуска игры!",
    show_hidden: "Показывать скрытые",
    select_folder: "Выбрать корневую папку",
    gallery: "Галерея ресурсов",
    loading_gallery: "Загрузка изображений, пожалуйста подождите...",
    drop_here: "Кликните и выберите отредактированное изображение",
    open_folder: "Открыть папку",
    img_copied: "Картинка успешно локализована!",
    no_images_found: "Изображения не найдены (возможно, нужно распаковать архивы RPA)",
    status_translated: "Переведено",
    ai_assistant: "Ассистент перевода",
    export_csv: "Экспорт CSV",
    import_csv: "Импорт CSV",
    back: "Назад",
    visual_editor: "Визуальный редактор",
    raw_code: "Сырой код",
    save: "Сохранить",
    empty_state_title: "Выберите папку с игрой для начала работы",
    empty_state_desc1: "Нажмите «Выбрать папку» в правом верхнем углу.",
    empty_state_desc2: "Внимание: Выбирайте именно КОРНЕВУЮ папку, где лежит .exe файл игры.",
    search_placeholder: "Поиск...",
    no_translation_found: "Структура перевода не найдена",
    unpack_first: "Сначала распакуйте и вскройте оригинальные скрипты в колонках ниже.",
    ready_to_gen: "Оригиналы готовы! Теперь можно безопасно сгенерировать файлы перевода.",
    gen_btn: "Сгенерировать переводы",
    col_archives: "Архивы (.rpa)",
    col_scripts: "Скрипты (.rpyc)",
    col_translations: "Переводы",
    unpack_all: "Распаковать всё",
    decompile_all: "Вскрыть всё",
    apply_patch: "Внедрить патч",
    fix_squares: "Лечить шрифты",
    fix_squares_hint: "Включить стандартный шрифт с кириллицей, если в игре вместо текста отображаются квадраты",
    no_archives: "Нет архивов",
    no_scripts: "Нет rpyc файлов",
    no_translations: "Сгенерируйте переводы выше",
    btn_hide: "Скрыть",
    btn_show: "Показать",
    btn_translate: "Перевод",
    status_done: "Готово",
    raw_no_blocks: "В этом файле не найдено блоков для перевода.",
    raw_view_mode: "Режим просмотра исходного кода",
    file_structure: "Структура файла",
    live_reload_hint: "После сохранения нажмите Shift + R в окне игры для обновления текста.",
    line_num: "Строка",
    input_placeholder: "Введите перевод...",
    tag_error: "ОШИБКА: Потеряны теги:",
    glossary: "Глоссарий",
    glos_orig: "Оригинал...",
    glos_tran: "Перевод...",
    glos_add: "Добавить",
    modal_ai_title: "Пакетный AI Ассистент",
    ai_tabs_local: "Локальная LLM (Ollama)",
    ai_tabs_manual: "Ручной (Copy-Paste)",
    ai_lines_range: "Интервал строк",
    ai_ollama_settings: "Настройки Ollama",
    ai_action: "Действие",
    ai_processing: "Обработка через Ollama...",
    ai_translate_btn: "Перевести через Ollama",
    step_1: "Подготовка промпта",
    step_1_desc: "Скопировать интервал строк в буфер обмена.",
    from: "От:",
    to: "До:",
    copy_ai: "Скопировать для AI",
    step_2: "Вставка ответа",
    step_2_desc: "Вставьте ответ нейросети (строки должны начинаться с цифр 1, 2, 3...):",
    apply_ai: "Применить",
    hide_translated: "Скрыть переведенное",
    saved_at: "Сохранено:",
    msg_engine_working: "Движок собирает текст... Ждите.",
    msg_patch_applied: "Патч внедрен! Запускайте игру, перевод применен. Также работает Shift+R.",
    msg_unpack_done: "Готово! Распаковано архивов:",
    msg_decomp_done: "Готово! Вскрыто файлов:",
    msg_file_saved: "Файл сохранен!",
    msg_copy_success: "Пакет строк успешно скопирован!",
    msg_copy_err: "Ошибка копирования в буфер обмена.",
    msg_ai_applied: "Успешно применено переводов:",
    msg_csv_exported: "Файл успешно экспортирован!",
    msg_csv_imported: "Импортировано строк:",
    revert: "Откатить",
    revert_img: "Удалить переведенную картинку",
    confirm_revert: "Вы уверены, что хотите удалить переведенную картинку?",
    img_reverted: "Переведенная картинка удалена!",
    unpacking: "Распаковка:",
    decompiling: "Вскрытие:",
    msg_cannot_save_errors: "Невозможно сохранить: есть ошибки в тегах."
  },
  en: {
    settings: "Settings",
    ui_theme: "Theme",
    theme_light: "Light",
    theme_dark: "Dark",
    autosave: "Autosave",
    on: "On",
    off: "Off",
    close: "Close",
    ui_lang: "Interface",
    target_lang: "Translate to",
    custom_lang: "Custom...",
    custom_lang_placeholder: "e.g., ukrainian",
    help_title: "How to use",
    help_step1: "Select the root game folder (where the .exe file is located).",
    help_step2: "In the left and center columns, click <i>Unpack All</i> and <i>Decompile All</i>.",
    help_step3: "Once the originals are decompiled, click the blue <i>Generate Translations</i> button.",
    help_step4: "In the right column, click <b>Apply Patch</b>. This will force the game to start with the selected translation language.",
    help_step5: "Open files, translate lines, and press <b>Save</b>. If the game is running, press <b>Shift+R</b> in its window to update the text without restarting!",
    show_hidden: "Show hidden",
    select_folder: "Select Root Folder",
    gallery: "Assets Gallery",
    loading_gallery: "Loading images, please wait...",
    drop_here: "Click and select the edited image",
    open_folder: "Open folder",
    img_copied: "Image successfully localized!",
    no_images_found: "No images found (you might need to unpack RPA archives first)",
    status_translated: "Translated",
    ai_assistant: "AI Assistant",
    export_csv: "Export CSV",
    import_csv: "Import CSV",
    back: "Back",
    visual_editor: "Visual Editor",
    raw_code: "Raw Code",
    save: "Save",
    empty_state_title: "Select game folder to start",
    empty_state_desc1: "Click «Select Root Folder» in the top right corner.",
    empty_state_desc2: "Warning: Select the ROOT folder where the .exe file is located.",
    search_placeholder: "Search...",
    no_translation_found: "Translation structure not found",
    unpack_first: "First unpack and decompile the original scripts in the columns below.",
    ready_to_gen: "Originals are ready! You can now generate translation files.",
    gen_btn: "Generate Translations",
    col_archives: "Archives (.rpa)",
    col_scripts: "Scripts (.rpyc)",
    col_translations: "Translations",
    unpack_all: "Unpack All",
    decompile_all: "Decompile All",
    apply_patch: "Apply Patch",
    fix_squares: "Fix Fonts",
    fix_squares_hint: "Force default font with Cyrillic support if the game shows squares instead of text",
    no_archives: "No archives",
    no_scripts: "No rpyc files",
    no_translations: "Generate translations above",
    btn_hide: "Hide",
    btn_show: "Show",
    btn_translate: "Translate",
    status_done: "Done",
    raw_no_blocks: "No translation blocks found in this file.",
    raw_view_mode: "Raw source code view mode",
    file_structure: "File Structure",
    live_reload_hint: "After saving, press Shift + R in the game window to update the text.",
    line_num: "Line",
    input_placeholder: "Enter translation...",
    tag_error: "ERROR: Missing tags:",
    glossary: "Glossary",
    glos_orig: "Original...",
    glos_tran: "Translation...",
    glos_add: "Add",
    modal_ai_title: "Batch AI Assistant",
    ai_tabs_local: "Local LLM (Ollama)",
    ai_tabs_manual: "Manual Copy-Paste",
    ai_lines_range: "Lines Range",
    ai_ollama_settings: "Ollama Settings",
    ai_action: "Action",
    ai_processing: "Processing via Ollama...",
    ai_translate_btn: "Translate with Ollama",
    step_1: "Prompt Preparation",
    step_1_desc: "Copy interval of lines to clipboard.",
    from: "From:",
    to: "To:",
    copy_ai: "Copy for AI",
    step_2: "Paste Response",
    step_2_desc: "Paste the neural network response (lines must start with numbers 1, 2, 3...):",
    apply_ai: "Apply",
    hide_translated: "Hide translated",
    saved_at: "Saved:",
    msg_engine_working: "Engine is processing text... Please wait.",
    msg_patch_applied: "Patch applied! Run the game to see the translation.",
    msg_unpack_done: "Done! Unpacked archives:",
    msg_decomp_done: "Done! Decompiled files:",
    msg_file_saved: "File saved!",
    msg_copy_success: "Batch copied successfully!",
    msg_copy_err: "Failed to copy to clipboard.",
    msg_ai_applied: "Translations applied successfully:",
    msg_csv_exported: "File exported successfully!",
    msg_csv_imported: "Rows imported:",
    revert: "Revert",
    revert_img: "Delete translated image",
    confirm_revert: "Are you sure you want to delete the translated image?",
    img_reverted: "Translated image deleted!",
    unpacking: "Unpacking:",
    decompiling: "Decompiling:",
    msg_cannot_save_errors: "Cannot save: there are tag errors."
  }
};

const t = (key) => locales[uiLang.value]?.[key] || key;

// ============================================================================
// Core State
// ============================================================================

const currentMode = ref('dashboard');
const showRawView = ref(false);
const projectPath = ref('');
const isProcessing = ref(false);
const forceFontFix = ref(false);
const error = ref('');
const successMsg = ref('');

const projectFiles = ref({ rpa_files: [], rpyc_files: [], rpy_files:[] });
const fileStats = ref({});
const parsedBlocks = ref([]);
const currentFilePath = ref('');
const rawFileText = ref('');
const charMap = ref({});
const showHelp = ref(false);

const searchQuery = ref('');
const searchResults = ref([]);

const hiddenFiles = ref(JSON.parse(localStorage.getItem('renforge_hidden') || '[]'));
const showHidden = ref(false);
const completedFiles = ref(JSON.parse(localStorage.getItem('renforge_completed') || '[]'));
const glossary = ref(JSON.parse(localStorage.getItem('renforge_glossary') || '[]'));
const newTerm = ref({ original: '', translation: '' });

const hideTranslated = ref(false);
const focusedBlockId = ref(null);

// ============================================================================
// Security Utilities
// ============================================================================

/**
 * Escapes unsafe characters in a string to prevent Cross-Site Scripting (XSS).
 * 
 * @param {string} unsafe - The raw string to escape.
 * @returns {string} The HTML-escaped safe string.
 */
function escapeHtml(unsafe) {
  if (!unsafe) return '';
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

// ============================================================================
// Asset Gallery Manager
// ============================================================================

const galleryImages = ref([]);
const gallerySearch = ref('');
const isGalleryLoading = ref(false);
const galleryLimit = ref(50);

/**
 * Initializes and opens the asset gallery view.
 */
async function openGallery() {
    currentMode.value = 'gallery';
    gallerySearch.value = '';
    galleryLimit.value = 50;
    await loadGallery();
}

/**
 * Loads the list of available images from the backend.
 */
async function loadGallery() {
    if (!projectPath.value) return;
    isGalleryLoading.value = true;
    try {
        galleryImages.value = await invoke('get_images_list', { 
            projectPath: projectPath.value, 
            targetLang: targetLang.value 
        });
    } catch(e) { showMsg('error', e.toString()); } 
    finally { isGalleryLoading.value = false; }
}

const filteredGallery = computed(() => {
    let result = galleryImages.value;
    if (gallerySearch.value) {
        const s = gallerySearch.value.toLowerCase();
        result = result.filter(img => img.rel_path.toLowerCase().includes(s));
    }
    return result;
});

const paginatedGallery = computed(() => {
    return filteredGallery.value.slice(0, galleryLimit.value);
});

function loadMoreImages() {
    galleryLimit.value += 50;
}

function getImgSrc(img) {
    const path = img.is_translated && img.translated_path ? img.translated_path : img.original_path;
    return convertFileSrc(path);
}

/**
 * Opens OS file dialog to select a localized image to import.
 */
async function importImageDialog(img) {
    try {
        const selected = await open({
            multiple: false,
            filters: [{ name: 'Images', extensions:['png', 'jpg', 'jpeg', 'webp'] }]
        });
        if (selected) { await processImageImport(img, selected); }
    } catch(e) { showMsg('error', e.toString()); }
}

/**
 * Triggers backend process to copy and map the image to the localization directory.
 */
async function processImageImport(img, sourcePath) {
    try {
        const translated_path = await invoke('import_localized_image', {
            projectPath: projectPath.value,
            targetLang: targetLang.value,
            relPath: img.rel_path,
            sourceFilePath: sourcePath
        });
        img.is_translated = true;
        img.translated_path = translated_path;
        showMsg('success', t('img_copied'));
    } catch(e) { showMsg('error', e.toString()); }
}

/**
 * Reverts an image to its unlocalized original state by deleting the localized copy.
 */
async function revertImage(img) {
    if (!confirm(t('confirm_revert'))) return;
    try {
        await invoke('delete_localized_image', {
            projectPath: projectPath.value,
            targetLang: targetLang.value,
            relPath: img.rel_path
        });
        img.is_translated = false;
        img.translated_path = null;
        showMsg('success', t('img_reverted'));
    } catch(e) { showMsg('error', e.toString()); }
}

async function openImgFolder(path) {
    try { await invoke('open_in_explorer', { path }); } 
    catch(e) { showMsg('error', e.toString()); }
}

// ============================================================================
// Autosave Feature
// ============================================================================

let autosaveTimer = null;
const lastSavedTime = ref('');

/**
 * Initializes the background autosave timer.
 */
function setupAutosave() {
  clearAutosave();
  if (autosaveEnabled.value) {
    autosaveTimer = setInterval(() => {
      if (currentMode.value === 'editor' && !hasErrors.value && parsedBlocks.value.length > 0) {
        saveFile(true);
      }
    }, 120000); // 2 minutes interval
  }
}

function clearAutosave() {
  if (autosaveTimer) clearInterval(autosaveTimer);
}

// ============================================================================
// AI Assistant Integration
// ============================================================================

const isAiModalOpen = ref(false);
const aiTab = ref('ollama');
const aiStart = ref(1);
const aiEnd = ref(30);
const aiInput = ref('');
const currentAiBatch = ref([]);

const ollamaUrl = ref(localStorage.getItem('renforge_ollama_url') || 'http://localhost:11434');
const ollamaModel = ref(localStorage.getItem('renforge_ollama_model') || 'llama3');
const isOllamaTranslating = ref(false);

function openAiModal() {
    isAiModalOpen.value = true;
    aiInput.value = '';
    aiStart.value = 1;
    aiEnd.value = Math.min(30, parsedBlocks.value.length);
}

/**
 * Prepares a batch of strings for the AI model, constructing a strict prompt 
 * with context and glossary adherence rules.
 * 
 * @param {boolean} returnPrompt - If true, returns the string instead of copying to clipboard.
 * @returns {Promise<string|null>} The generated prompt.
 */
async function prepareAiBatch(returnPrompt = false) {
    let startIdx = parseInt(aiStart.value) - 1;
    let endIdx = parseInt(aiEnd.value);
    
    if (startIdx < 0) startIdx = 0;
    if (endIdx > parsedBlocks.value.length) endIdx = parsedBlocks.value.length;
    if (startIdx >= endIdx) {
        if (!returnPrompt) showMsg('error', 'Invalid interval!');
        return null;
    }

    const batch = parsedBlocks.value.slice(startIdx, endIdx);
    currentAiBatch.value = batch.map(b => b.id);

    let prompt = `You are a professional visual novel translator. Translate the following ${batch.length} lines into ${targetLang.value} language.\n\n`;

    if (glossary.value.length > 0) {
        prompt += `--- GLOSSARY (Strictly use these terms!) ---\n`;
        glossary.value.forEach(t => { prompt += `${t.original} = ${t.translation}\n`; });
        prompt += `--------------------------------------------\n\n`;
    }

    prompt += `CRITICAL RULES:\n`;
    prompt += `1. PRESERVE ALL TAGS AND VARIABLES! If the original has [name], [player], {b}, {color=#f00}, \\n, or similar, they MUST remain exactly the same in the translation.\n`;
    prompt += `2. NEVER translate the variable name itself (e.g.[name] stays [name], do NOT translate to [имя]).\n`;
    prompt += `3. Output ONLY the numbered list with translations. No introductory text, no explanations.\n`;
    prompt += `4. You MUST translate exactly ${batch.length} lines and keep their exact original numbering.\n\n`;
    
    prompt += `--- TEXT TO TRANSLATE ---\n`;
    
    batch.forEach((b, i) => {
        let charInfo = b.prefix ? b.prefix.replace(/[^a-zA-Z0-9_]/g, '') : 'Narrator';
        if (!charInfo) charInfo = 'Narrator';
        if (charMap.value[charInfo]) charInfo = charMap.value[charInfo];
        prompt += `${i + 1}.[${charInfo}]: ${b.original}\n`;
    });

    if (returnPrompt) return prompt;

    try {
        await navigator.clipboard.writeText(prompt);
        showMsg('success', t('msg_copy_success'));
    } catch(e) { showMsg('error', t('msg_copy_err')); }
}

/**
 * Parses numeric lists from AI responses and injects the text into blocks.
 */
function parseAiResponseAndApply(text) {
    const lines = text.split('\n')
        .map(l => l.replace(/^\d+[\.\)]\s*/, '').trim())
        .filter(l => l.length > 0);

    let appliedCount = 0;
    currentAiBatch.value.forEach((id, index) => {
        if (lines[index]) {
            const block = parsedBlocks.value.find(b => b.id === id);
            if (block) {
                block.translation = lines[index];
                appliedCount++;
            }
        }
    });
    return appliedCount;
}

function importAiBatch() {
    const applied = parseAiResponseAndApply(aiInput.value);
    showMsg('success', `${t('msg_ai_applied')} ${applied}`);
    isAiModalOpen.value = false;
}

/**
 * Triggers a REST call to a local Ollama instance for autonomous AI translation.
 */
async function runLocalLLM() {
    localStorage.setItem('renforge_ollama_url', ollamaUrl.value);
    localStorage.setItem('renforge_ollama_model', ollamaModel.value);
    
    const prompt = await prepareAiBatch(true);
    if (!prompt) return;

    isOllamaTranslating.value = true;
    try {
        const res = await fetch(`${ollamaUrl.value}/api/generate`, {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({
                model: ollamaModel.value,
                prompt: prompt,
                stream: false,
                options: {
                    num_predict: -1, 
                    num_ctx: 8192    
                }
            })
        });
        
        if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
        
        const data = await res.json();
        const applied = parseAiResponseAndApply(data.response);
        showMsg('success', `${t('msg_ai_applied')} ${applied}`);
        isAiModalOpen.value = false;
    } catch (e) {
        showMsg('error', 'Ollama API error: ' + e.message);
    } finally {
        isOllamaTranslating.value = false;
    }
}

// ============================================================================
// Data Export & Import (CSV)
// ============================================================================

/**
 * Exports current parsed blocks to CSV format, injecting protection against formula execution.
 */
async function exportCSV() {
    let csvContent = "ID;Original;Translation\n";
    parsedBlocks.value.forEach(b => {
        const orig = b.original.replace(/"/g, '""').replace(/\n/g, "[BR]");
        let tran = (b.translation || "").replace(/"/g, '""').replace(/\n/g, "[BR]");
        
        // Prevent CSV Formula Injection
        if (/^[=+\-@]/.test(tran)) {
            tran = "'" + tran;
        }

        csvContent += `"${b.id}";"${orig}";"${tran}"\n`;
    });
    
    try {
        const savePath = await save({ filters: [{ name: 'CSV', extensions:['csv'] }] });
        if (savePath) {
            await invoke('write_text_file', { path: savePath, content: csvContent });
            showMsg('success', t('msg_csv_exported'));
        }
    } catch (e) { showMsg('error', `Error: ${e}`); }
}

/**
 * Imports translations from a user-selected CSV file.
 */
async function importCSV() {
    try {
        const selected = await open({ multiple: false, filters:[{ name: 'CSV', extensions: ['csv'] }] });
        if (!selected) return;
        
        const csvContent = await invoke('read_text_file', { path: selected });
        const lines = csvContent.split('\n');
        let updatedCount = 0;
        
        for (let i = 1; i < lines.length; i++) {
            const line = lines[i].trim();
            if (!line) continue;
            
            const match = line.match(/^"([^"]+)";"([^"]*)";"([^"]*)"$/);
            if (match) {
                const id = match[1];
                let tran = match[3].replace(/\[BR\]/g, "\n").replace(/""/g, '"');
                if (tran.startsWith("'") && /^[=+\-@]/.test(tran.substring(1))) {
                    tran = tran.substring(1);
                }
                
                const block = parsedBlocks.value.find(b => b.id === id);
                if (block && tran) {
                    block.translation = tran;
                    updatedCount++;
                }
            }
        }
        showMsg('success', `${t('msg_csv_imported')} ${updatedCount}.`);
    } catch (e) { showMsg('error', `Error: ${e}`); }
}

// ============================================================================
// Utility Methods
// ============================================================================

let msgTimeoutId = null;

function showMsg(type, text, timeout = 8000) {
  if (msgTimeoutId) clearTimeout(msgTimeoutId);
  if (type === 'error') { error.value = text; successMsg.value = ''; }
  else { successMsg.value = text; error.value = ''; }
  
  if (timeout > 0) {
    msgTimeoutId = setTimeout(() => { closeMsg(); }, timeout);
  }
}

function closeMsg() {
  if (msgTimeoutId) clearTimeout(msgTimeoutId);
  error.value = ''; 
  successMsg.value = '';
}

function getFileName(fullPath) { return fullPath.split(/[/\\]/).pop(); }
function getRelativePath(fullPath) {
  if (!projectPath.value) return fullPath;
  const normalizedFull = fullPath.replace(/\\/g, '/');
  const normalizedProj = projectPath.value.replace(/\\/g, '/');
  return normalizedFull.replace(normalizedProj, '').replace(/^\//, '');
}

function toggleHide(filePath) {
  if (hiddenFiles.value.includes(filePath)) { hiddenFiles.value = hiddenFiles.value.filter(p => p !== filePath); } 
  else { hiddenFiles.value.push(filePath); }
  localStorage.setItem('renforge_hidden', JSON.stringify(hiddenFiles.value));
}

function toggleFileCompleted(filePath) {
  if (completedFiles.value.includes(filePath)) { completedFiles.value = completedFiles.value.filter(p => p !== filePath); } 
  else { completedFiles.value.push(filePath); }
  localStorage.setItem('renforge_completed', JSON.stringify(completedFiles.value));
}

function addGlossaryTerm() {
  if (!newTerm.value.original.trim() || !newTerm.value.translation.trim()) return;
  glossary.value.push({ original: newTerm.value.original.trim(), translation: newTerm.value.translation.trim() });
  localStorage.setItem('renforge_glossary', JSON.stringify(glossary.value));
  newTerm.value = { original: '', translation: '' };
}

function removeGlossaryTerm(index) {
  glossary.value.splice(index, 1);
  localStorage.setItem('renforge_glossary', JSON.stringify(glossary.value));
}

/**
 * Highlights glossary terms within the text. Safe from XSS due to pre-escaping.
 * 
 * @param {string} text - The original unescaped text.
 * @returns {string} HTML string with glossary terms wrapped in interactive spans.
 */
function highlightGlossary(text) {
  if (!text) return '';
  let res = escapeHtml(text);
  
  const sortedTerms = [...glossary.value].sort((a, b) => b.original.length - a.original.length);
  for (const term of sortedTerms) {
    if (!term.original) continue;
    const escapedTerm = escapeHtml(term.original).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`(${escapedTerm})`, 'gi');
    res = res.replace(regex, `<span class="glossary-word" title="${escapeHtml(term.translation)}">$1</span>`);
  }
  return res;
}

const filteredFiles = computed(() => {
  const filterFn = (files) => files.filter(f => showHidden.value || !hiddenFiles.value.includes(f));
  return {
    rpa: filterFn(projectFiles.value.rpa_files),
    rpyc: filterFn(projectFiles.value.rpyc_files),
    rpy: filterFn(projectFiles.value.rpy_files),
  };
});

const translationFiles = computed(() => filteredFiles.value.rpy.filter(f => f.includes(`/tl/${targetLang.value}`) || f.includes(`\\tl\\${targetLang.value}`)));
const originalRpyFiles = computed(() => filteredFiles.value.rpy.filter(f => !f.includes('/tl/') && !f.includes('\\tl\\')));

function isAlreadyDecompiled(rpycPath) {
  const expectedRpyPath = rpycPath.replace(/\.rpyc$/, '.rpy');
  return projectFiles.value.rpy_files.includes(expectedRpyPath);
}

// ============================================================================
// Core Project Logic
// ============================================================================

async function handleSearch() {
  if (searchQuery.value.length < 3) { searchResults.value =[]; return; }
  try { searchResults.value = await invoke('search_in_db', { projectPath: projectPath.value, query: searchQuery.value }); } catch (e) { console.error(e); }
}

async function jumpToFile(result) {
    await openEditor(result.file_path);
    searchQuery.value = ''; searchResults.value =[];
    setTimeout(() => scrollToBlock(result.id), 500);
}

async function openProjectFolder() {
  try {
    const selectedPath = await open({ multiple: false, directory: true });
    if (!selectedPath) return; 
    projectPath.value = selectedPath; 
    await refreshProject();
  } catch (e) { showMsg('error', `Error: ${e}`); }
}

async function refreshProject() {
  try { 
      isProcessing.value = true; 
      projectFiles.value = await invoke('scan_project', { path: projectPath.value }); 
      charMap.value = await invoke('get_character_mapping', { projectPath: projectPath.value });
      fileStats.value = await invoke('get_translation_stats', { projectPath: projectPath.value });
  } 
  catch (e) { showMsg('error', `Error: ${e}`); } 
  finally { isProcessing.value = false; }
}

async function generateTranslations() {
  try {
    isProcessing.value = true; showMsg('success', t('msg_engine_working'), 0);
    const res = await invoke('generate_translations', { path: projectPath.value, targetLang: targetLang.value });
    showMsg('success', res); await refreshProject();
  } catch (e) { showMsg('error', e, 15000); } finally { isProcessing.value = false; }
}

async function applyPatch() {
  try {
    isProcessing.value = true;
    await invoke('apply_renforge_patch', { 
        projectPath: projectPath.value, 
        targetLang: targetLang.value,
        forceFont: forceFontFix.value 
    });
    showMsg('success', t('msg_patch_applied'), 5000);
  } catch (e) { showMsg('error', `Error: ${e}`); } finally { isProcessing.value = false; }
}

async function extractAllRpa() {
  const files = filteredFiles.value.rpa;
  if (files.length === 0) return;
  isProcessing.value = true; 
  let successCount = 0;
  
  showMsg('success', `${t('unpacking')} 0 / ${files.length}...`, 0);
  
  for (let i = 0; i < files.length; i++) {
    try { 
        await invoke('run_unrpa', { filePath: files[i] }); 
        successCount++; 
        showMsg('success', `${t('unpacking')} ${successCount} / ${files.length}...`, 0);
    } catch (e) { console.error(`Failed to unpack ${files[i]}:`, e); }
  }
  
  showMsg('success', `${t('msg_unpack_done')} ${successCount}`, 8000); 
  await refreshProject();
}

async function decompileAllRpyc() {
  const files = filteredFiles.value.rpyc.filter(f => !isAlreadyDecompiled(f));
  if (files.length === 0) return;
  isProcessing.value = true; 
  let successCount = 0;
  
  showMsg('success', `${t('decompiling')} 0 / ${files.length}...`, 0);
  
  for (let i = 0; i < files.length; i++) {
    try { 
        await invoke('run_unrpyc', { filePath: files[i] }); 
        successCount++; 
        showMsg('success', `${t('decompiling')} ${successCount} / ${files.length}...`, 0);
    } catch (e) { console.error(`Failed to decompile ${files[i]}:`, e); }
  }
  
  showMsg('success', `${t('msg_decomp_done')} ${successCount}`, 8000); 
  await refreshProject();
}

// ============================================================================
// Parsing & Editor Engine
// ============================================================================

function extractTags(text) { return text.match(/(\[.*?\]|\{.*?\})/g) ||[]; }
function getMissingTags(block) { return extractTags(block.original).filter(tag => !block.translation.includes(tag)); }

function getBlockStatus(block) {
  if (getMissingTags(block).length > 0) return 'error'; 
  if (!block.translation.trim() || block.translation === block.original) return 'untranslated'; 
  return 'translated'; 
}

const hasErrors = computed(() => parsedBlocks.value.some(block => getBlockStatus(block) === 'error'));

function scrollToBlock(id) { 
    const el = document.getElementById('block-' + id); 
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' }); 
}

/**
 * Extracts character prefixes, dialogue text, and suffixes robustly.
 * Supports escaped quotes (\") inside strings.
 */
function extractDialogueParts(line) {
  const match = line.match(/^([^"]*)"((?:\\.|[^"\\])*)"(.*)$/);
  if (match) return { prefix: match[1], content: match[2], suffix: match[3] };
  return { prefix: '', content: line, suffix: '' };
}

/**
 * Main parser for Ren'Py (.rpy) files. Extracts text blocks for localization.
 * 
 * @param {string} rawText - Raw source code of the .rpy file.
 * @param {string} filePath - Path of the file being processed.
 * @returns {Array<Object>} Array of block objects available for translation.
 */
function parseRpy(rawText, filePath) {
  const blocks =[];
  const lines = rawText.split('\n');
  let currentId = null;
  let isStrings = false;
  let tempOriginal = '';

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimLine = line.trim();

    const idMatch = line.match(/^translate\s+\S+\s+([^:]+):/);
    if (idMatch) {
      currentId = idMatch[1];
      isStrings = (currentId === 'strings');
      tempOriginal = '';
      continue;
    }

    if (!currentId) continue;

    if (isStrings) {
        if (trimLine.startsWith('old ') && line.includes('"')) {
            const parts = extractDialogueParts(trimLine);
            tempOriginal = parts.content.replace(/\\"/g, '"');
        } else if (trimLine.startsWith('new ') && line.includes('"')) {
            const p = extractDialogueParts(line);
            const block = { 
              id: `string_${i}`, original: tempOriginal, translation: p.content.replace(/\\"/g, '"'), 
              prefix: p.prefix, suffix: p.suffix, lineIndex: i
            };
            blocks.push(block);
            invoke('upsert_translation', { projectPath: projectPath.value, entry: { id: block.id, file_path: filePath, original: block.original, translation: block.translation, status: getBlockStatus(block) } });
        }
    } else {
        let cleanLine = trimLine.startsWith('#') ? trimLine.substring(1).trim() : trimLine;
        if (/^(voice|play|stop|scene|show|hide|window|pause|\$|jump|call|return)(?:\s|\(|$)/.test(cleanLine)) continue;

        if (trimLine.startsWith('#') && line.includes('"')) {
            const p = extractDialogueParts(cleanLine);
            tempOriginal = p.content.replace(/\\"/g, '"');
        } else if (trimLine !== '' && !trimLine.startsWith('#') && line.includes('"')) {
            const p = extractDialogueParts(line);
            const block = { 
              id: currentId, original: tempOriginal, translation: p.content.replace(/\\"/g, '"'), 
              prefix: p.prefix, suffix: p.suffix, lineIndex: i
            };
            blocks.push(block);
            invoke('upsert_translation', { projectPath: projectPath.value, entry: { id: block.id, file_path: filePath, original: block.original, translation: block.translation, status: getBlockStatus(block) } });
            currentId = null; 
        }
    }
  }
  return blocks;
}

/**
 * Loads and parses a specific .rpy file into the visual editor.
 */
async function openEditor(filePath) {
  try {
    const text = await invoke('read_rpy_file', { projectPath: projectPath.value, filePath: filePath });
    rawFileText.value = text.replace(/\r\n/g, '\n'); 
    currentFilePath.value = filePath;
    parsedBlocks.value = parseRpy(rawFileText.value, filePath); 
    showRawView.value = false; currentMode.value = 'editor';
    hideTranslated.value = false;
    setupAutosave();
  } catch (e) { showMsg('error', `Error: ${e}`); }
}

function closeEditor() { 
  currentMode.value = 'dashboard'; 
  parsedBlocks.value =[]; 
  currentFilePath.value = ''; 
  showRawView.value = false; 
  focusedBlockId.value = null;
  clearAutosave();
  refreshProject();
}

/**
 * Saves modifications back to the disk and database.
 */
async function saveFile(isAuto = false) {
  if (!currentFilePath.value || hasErrors.value) return;
  try {
    const lines = rawFileText.value.split('\n');
    for (const block of parsedBlocks.value) {
        const escaped = block.translation.replace(/"/g, '\\"');
        lines[block.lineIndex] = `${block.prefix}"${escaped}"${block.suffix}`;
        invoke('upsert_translation', { projectPath: projectPath.value, entry: { id: block.id, file_path: currentFilePath.value, original: block.original, translation: block.translation, status: getBlockStatus(block) } });
    }
    const newFileContent = lines.join('\n');
    await invoke('write_rpy_file', { projectPath: projectPath.value, filePath: currentFilePath.value, content: newFileContent });
    rawFileText.value = newFileContent; 
    
    const d = new Date();
    lastSavedTime.value = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`;
    
    if (!isAuto) showMsg('success', t('msg_file_saved'));
  } catch (e) { showMsg('error', `Error: ${e}`); }
}
</script>

<style>
/* === GLOBAL === */
*, *::before, *::after {
  box-sizing: border-box;
}

/* === THEME VARS === */
:root {
  --bg-app: #ffffff;
  --bg-base: #f3f4f6;
  --bg-panel: #f9fafb;
  --bg-input: #f9fafb;
  --bg-input-focus: #ffffff;
  
  --text-main: #1f2937;
  --text-secondary: #4b5563;
  --text-muted: #6b7280;
  
  --border-main: #e5e7eb;
  --border-input: #d1d5db;
  
  --accent: #2563eb;
  --accent-hover: #1d4ed8;
  
  --btn-sec-bg: #f3f4f6;
  --btn-sec-hover: #e5e7eb;
  --btn-outline-text: #374151;
  --btn-outline-hover: #f9fafb;
  
  --error-bg: #fef2f2;
  --error-text: #b91c1c;
  --error-border: #fecaca;
  
  --success-bg: #f0fdf4;
  --success-text: #15803d;
  --success-border: #bbf7d0;
  
  --code-bg: #f8fafc;
  --code-text: #334155;
  
  --shadow-color: rgba(0,0,0,0.05);
}[data-theme="dark"] {
  --bg-app: #1e1e1e;
  --bg-base: #121212;
  --bg-panel: #252526;
  --bg-input: #1e1e1e;
  --bg-input-focus: #1a1a1a;
  
  --text-main: #e0e0e0;
  --text-secondary: #a0a0a0;
  --text-muted: #858585;
  
  --border-main: #333333;
  --border-input: #444444;
  
  --accent: #007acc;
  --accent-hover: #005999;
  
  --btn-sec-bg: #333333;
  --btn-sec-hover: #444444;
  --btn-outline-text: #d4d4d4;
  --btn-outline-hover: #2a2d31;
  
  --error-bg: #451a1a;
  --error-text: #ff8a8a;
  --error-border: #5a1d1d;
  
  --success-bg: #1a3a24;
  --success-text: #8aff9e;
  --success-border: #2e7d32;
  
  --code-bg: #252526;
  --code-text: #cecece;
  
  --shadow-color: rgba(0,0,0,0.3);
}

body { 
  margin: 0; padding: 0; 
  background-color: var(--bg-base); 
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; 
  color: var(--text-main); 
  overflow: hidden; 
}
.glossary-word { border-bottom: 2px dashed var(--accent); color: var(--accent); cursor: help; padding-bottom: 1px; transition: background 0.2s; }
.glossary-word:hover { background-color: var(--bg-hover); }
</style>

<style scoped>
.app-container { display: flex; flex-direction: column; height: 100vh; background-color: var(--bg-app); color: var(--text-main); }

/* === HEADER === */
.header { display: flex; justify-content: space-between; align-items: center; padding: 12px 25px; background-color: var(--bg-app); border-bottom: 1px solid var(--border-main); box-shadow: 0 1px 2px var(--shadow-color); z-index: 10; }
.logo { font-size: 20px; font-weight: 700; color: var(--accent); }
.version { font-size: 12px; color: var(--text-muted); font-weight: normal; }

.actions { display: flex; gap: 10px; align-items: center; }
.info-btn { background: transparent; border: 1px solid var(--border-input); color: var(--text-muted); font-weight: bold; font-size: 12px; padding: 4px 8px; border-radius: 4px; cursor: pointer; transition: 0.2s; margin-right: 10px; }
.info-btn:hover { border-color: var(--accent); color: var(--accent); background: var(--bg-panel); }
.csv-group { display: flex; gap: 6px; border-left: 1px solid var(--border-main); padding-left: 10px; margin-left: 4px; }
.autosave-indicator { font-size: 12px; color: var(--text-muted); margin-right: 10px; font-weight: 600; border-right: 1px solid var(--border-main); padding-right: 10px;}

.btn { padding: 7px 14px; font-size: 13px; border-radius: 6px; cursor: pointer; font-weight: 500; transition: all 0.2s; outline: none; border: 1px solid transparent; }
.btn-primary { background-color: var(--accent); color: #ffffff; border-color: var(--accent); }
.btn-primary:hover:not(:disabled) { background-color: var(--accent-hover); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-secondary { background-color: var(--btn-sec-bg); color: var(--text-main); border-color: var(--border-input); }
.btn-secondary:hover:not(:disabled) { background-color: var(--btn-sec-hover); border-color: var(--text-muted); }
.btn-outline { background-color: transparent; border-color: var(--border-input); color: var(--btn-outline-text); }
.btn-outline:hover { background-color: var(--btn-outline-hover); border-color: var(--text-muted); }

.icon-text-btn { background: none; border: none; font-size: 12px; color: var(--text-muted); cursor: pointer; padding: 4px 6px; transition: 0.2s; }
.icon-text-btn:hover { color: var(--text-main); background: var(--bg-panel); border-radius: 4px; }
.toggle-hidden { font-size: 13px; color: var(--text-secondary); display: flex; align-items: center; gap: 5px; margin-right: 15px; cursor: pointer; font-weight: 500; }

.global-msg { padding: 10px 25px; font-weight: 500; font-size: 14px; border-bottom: 1px solid transparent; display: flex; justify-content: space-between; align-items: center;}
.error-msg { background-color: var(--error-bg); color: var(--error-text); border-color: var(--error-border); }
.success-msg { background-color: var(--success-bg); color: var(--success-text); border-color: var(--success-border); }

.msg-close-btn { background: none; border: none; font-size: 18px; color: inherit; cursor: pointer; opacity: 0.6; transition: 0.2s; font-weight: bold; }
.msg-close-btn:hover { opacity: 1; }

/* === DASHBOARD === */
.dashboard { padding: 25px; flex: 1; overflow-y: auto; background-color: var(--bg-base); }
.empty-state { text-align: center; margin-top: 10vh; color: var(--text-secondary); }
.empty-state h2 { color: var(--text-main); }
.empty-hint { color: var(--text-muted); font-size: 13px; margin-top: 10px; }

.dashboard-content { display: flex; flex-direction: column; gap: 20px; height: 100%; }

.search-section { position: relative; }
.search-input { width: 100%; padding: 12px 20px; background: var(--bg-app); border: 1px solid var(--border-input); border-radius: 8px; color: var(--text-main); font-size: 15px; outline: none; transition: 0.2s; box-shadow: 0 1px 2px var(--shadow-color); }
.search-input:focus { border-color: var(--accent); }
.search-results { position: absolute; top: 50px; left: 0; right: 0; background: var(--bg-app); border: 1px solid var(--border-main); border-radius: 8px; z-index: 100; max-height: 400px; overflow-y: auto; box-shadow: 0 10px 15px -3px var(--shadow-color); }
.search-res-item { padding: 12px 15px; border-bottom: 1px solid var(--border-main); cursor: pointer; display: flex; flex-direction: column; gap: 4px; transition: background 0.1s; }
.search-res-item:hover { background: var(--bg-panel); }
.res-file { font-size: 11px; color: var(--text-muted); font-weight: 600; }
.res-text { font-size: 14px; color: var(--text-main); }
.res-tran { font-size: 14px; color: var(--success-text); font-style: italic; }

.translation-banner { background-color: var(--bg-app); border: 1px solid var(--border-main); border-left: 4px solid var(--accent); border-radius: 8px; padding: 15px 25px; display: flex; justify-content: space-between; align-items: center; gap: 20px; box-shadow: 0 1px 2px var(--shadow-color); }
.banner-content h3 { margin: 0 0 6px 0; color: var(--text-main); font-size: 16px; font-weight: 600; }
.banner-content p { margin: 0; color: var(--text-secondary); font-size: 13px; line-height: 1.5; }

.dashboard-grid { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 20px; flex: 1; min-height: 0; }
.file-column { background-color: var(--bg-app); border-radius: 8px; display: flex; flex-direction: column; overflow: hidden; border: 1px solid var(--border-main); box-shadow: 0 1px 2px var(--shadow-color); }
.column-header { padding: 12px 15px; background-color: var(--bg-panel); font-weight: 600; font-size: 14px; border-bottom: 1px solid var(--border-main); color: var(--text-main); }
.column-header-flex { display: flex; justify-content: space-between; align-items: center; }
.tl-header { background-color: var(--success-bg); color: var(--success-text); border-bottom-color: var(--success-border); }
.apply-patch-btn { background-color: var(--success-text); color: var(--bg-app); border: none; padding: 6px 12px; border-radius: 4px; font-size: 12px; cursor: pointer; font-weight: 600; }
.apply-patch-btn:hover { opacity: 0.9; }

.column-content { flex: 1; overflow-y: auto; padding: 8px; }
.column-content::-webkit-scrollbar { width: 6px; }
.column-content::-webkit-scrollbar-thumb { background: var(--border-input); border-radius: 4px; }

.file-item { display: flex; justify-content: space-between; align-items: center; padding: 8px 10px; border-radius: 6px; margin-bottom: 4px; background-color: var(--bg-app); border: 1px solid transparent; transition: 0.1s;}
.file-item:hover { border-color: var(--border-main); background-color: var(--bg-panel); }
.file-item.is-hidden { opacity: 0.5; filter: grayscale(1); }
.file-item.is-completed { background-color: var(--success-bg); border-color: var(--success-border); }

.file-info { display: flex; flex-direction: column; gap: 4px; overflow: hidden; flex: 1; }
.file-name { font-size: 14px; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.file-path { font-size: 11px; color: var(--text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 90%; }
.file-actions { display: flex; align-items: center; gap: 8px; flex-shrink: 0; margin-left: 10px; }

.file-stats-wrapper { display: flex; flex-direction: column; gap: 4px; padding-right: 10px;}
.progress-bar-bg { width: 100%; height: 3px; background-color: var(--border-input); border-radius: 2px; overflow: hidden; }
.progress-bar-fill { height: 100%; background-color: var(--accent); transition: width 0.3s ease; }

.action-btn { background-color: var(--btn-sec-bg); border: 1px solid var(--border-input); color: var(--text-main); padding: 4px 10px; border-radius: 4px; cursor: pointer; font-size: 12px; font-weight: 500;}
.action-btn:hover:not(:disabled) { background-color: var(--btn-sec-hover); }
.action-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.edit-btn { background-color: var(--accent); color: #fff; border-color: var(--accent); }
.edit-btn:hover { background-color: var(--accent-hover); }
.bulk-btn { border: none; background-color: var(--border-main); }
.status-badge { font-size: 11px; padding: 2px 6px; border-radius: 4px; font-weight: 600; }
.status-done { background: var(--success-bg); color: var(--success-text); }
.no-files { text-align: center; padding: 20px; color: var(--text-muted); font-size: 13px; }

/* === GALLERY === */
.gallery-workspace { flex: 1; display: flex; flex-direction: column; overflow: hidden; background: var(--bg-base); }
.gallery-header { padding: 15px 25px; background: var(--bg-app); border-bottom: 1px solid var(--border-main); display: flex; justify-content: space-between; align-items: center; box-shadow: 0 1px 2px var(--shadow-color); z-index: 5;}
.gallery-header h2 { margin: 0; color: var(--text-main); font-size: 18px; font-weight: 600;}

.gallery-scroll-container { flex: 1; overflow-y: auto; padding: 25px; }

.gallery-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 20px; grid-auto-rows: max-content; }
.gallery-card { background: var(--bg-app); border: 1px solid var(--border-main); border-radius: 8px; overflow: hidden; display: flex; flex-direction: column; box-shadow: 0 1px 3px var(--shadow-color); transition: 0.2s; min-height: 240px; }
.gallery-card:hover { border-color: var(--accent); box-shadow: 0 4px 6px -1px var(--shadow-color);}
.gallery-img-container { height: 160px; position: relative; background: var(--bg-panel); display: flex; justify-content: center; align-items: center; cursor: pointer; overflow: hidden; }
.gallery-img { max-width: 100%; max-height: 100%; object-fit: contain; }
.gallery-img-overlay { position: absolute; inset: 0; background: rgba(0,0,0,0.6); color: white; display: flex; justify-content: center; align-items: center; opacity: 0; transition: 0.2s; font-size: 13px; font-weight: 600; text-align: center; padding: 10px; }
.gallery-img-container:hover .gallery-img-overlay { opacity: 1; }
.img-badge { position: absolute; top: 8px; right: 8px; z-index: 2; box-shadow: 0 2px 4px rgba(0,0,0,0.3); }
.gallery-card-info { padding: 12px; display: flex; flex-direction: column; gap: 8px; background: var(--bg-app); border-top: 1px solid var(--border-main);}
.img-path { font-size: 11px; color: var(--text-muted); word-break: break-all; font-weight: 500; }

.load-more-container { text-align: center; margin-top: 30px; padding-bottom: 20px; }
.load-more-container .btn { padding: 10px 20px; font-size: 14px; font-weight: 600; }

/* === EDITOR (Raw) === */
.raw-preview { flex: 1; display: flex; flex-direction: column; padding: 20px; background: var(--bg-app); overflow: hidden; }
.raw-preview-header { margin-bottom: 10px; color: var(--text-muted); font-size: 14px; }
.raw-code { flex: 1; background: var(--code-bg); padding: 15px; border-radius: 8px; border: 1px solid var(--border-main); color: var(--code-text); font-family: 'Consolas', monospace; font-size: 13px; overflow: auto; white-space: pre-wrap; word-break: break-all; }

/* === EDITOR (Visual) === */
.workspace { display: flex; flex: 1; overflow: hidden; background: var(--bg-app); }

.sidebar { width: 240px; background-color: var(--bg-panel); border-right: 1px solid var(--border-main); display: flex; flex-direction: column; }
.sidebar-title { padding: 15px; font-size: 12px; font-weight: 600; text-transform: uppercase; color: var(--text-muted); border-bottom: 1px solid var(--border-main); letter-spacing: 0.5px;}
.sidebar-list { flex: 1; overflow-y: auto; padding: 8px 0; }
.sidebar-list::-webkit-scrollbar { width: 6px; }
.sidebar-list::-webkit-scrollbar-thumb { background: var(--border-input); border-radius: 4px; }
.sidebar-item { padding: 6px 15px; font-size: 13px; color: var(--text-secondary); cursor: pointer; display: flex; align-items: center; gap: 10px; transition: 0.1s;}
.sidebar-item:hover { background-color: var(--bg-app); color: var(--text-main); }
.sidebar-index { color: var(--text-muted); font-size: 11px; width: 16px; text-align: right; }
.sidebar-id { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }

.status-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.untranslated { background-color: #ef4444; } .translated { background-color: #22c55e; } .error { background-color: #f59e0b; }

.editor-panel { flex: 1; overflow-y: auto; padding: 25px; display: flex; flex-direction: column; gap: 20px; background: var(--bg-app); scroll-behavior: smooth; }
.editor-panel::-webkit-scrollbar { width: 8px; }
.editor-panel::-webkit-scrollbar-thumb { background: var(--border-input); border-radius: 4px; }

.live-reload-hint { background: var(--bg-panel); border: 1px solid var(--border-main); padding: 12px 15px; border-radius: 6px; font-size: 13px; color: var(--accent); display: flex; align-items: center; gap: 10px; font-weight: 600;}

.translation-block { background-color: var(--bg-app); padding: 15px 20px; border-radius: 8px; border: 1px solid var(--border-main); border-left: 4px solid; box-shadow: 0 1px 3px var(--shadow-color); }
.status-untranslated { border-left-color: #ef4444; } .status-translated { border-left-color: #22c55e; } .status-error { border-left-color: #f59e0b; }

.block-header { margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center; }
.block-id { font-size: 11px; color: var(--text-muted); font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px;}

.original-text { font-size: 15px; color: var(--text-main); margin-bottom: 15px; line-height: 1.5; display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.char-prefix { font-size: 12px; font-weight: 600; padding: 2px 6px; border-radius: 4px; font-style: normal; }
.original-prefix { background-color: var(--bg-panel); color: var(--text-secondary); border: 1px solid var(--border-main);}
.char-mapping-name { color: var(--accent); font-weight: 700; font-size: 13px; }
.char-raw { color: var(--text-muted); font-size: 10px; margin-left: 4px; font-weight: normal; }

.fake-input-wrapper { display: flex; align-items: center; background-color: var(--bg-input); border: 1px solid var(--border-input); border-radius: 6px; padding: 4px 8px; transition: all 0.2s; }
.fake-input-wrapper:focus-within { border-color: var(--accent); background-color: var(--bg-input-focus); }
.translated-prefix { background-color: var(--accent); color: #fff; margin-right: 8px; flex-shrink: 0; }
.transparent-input { flex: 1; width: 100%; padding: 6px 0; background: transparent; border: none; color: var(--text-main); font-size: 15px; outline: none; }
.transparent-input::placeholder { color: var(--text-muted); }

.tag-error { margin-top: 12px; padding: 10px 12px; background-color: var(--error-bg); border: 1px solid var(--error-border); border-radius: 6px; font-size: 13px; color: var(--error-text); display: flex; align-items: center; gap: 8px;}
.missing-tag { background-color: #ef4444; color: white; padding: 2px 6px; border-radius: 4px; font-family: monospace; font-weight: bold;}

/* === RIGHT SIDEBAR (GLOSSARY) === */
.assistant-sidebar { width: 280px; background-color: var(--bg-panel); border-left: 1px solid var(--border-main); display: flex; flex-direction: column; }
.glossary-content { padding: 15px; flex: 1; overflow-y: auto; }
.glossary-content::-webkit-scrollbar { width: 6px; }
.glossary-content::-webkit-scrollbar-thumb { background: var(--border-input); border-radius: 4px; }

.glossary-add-form { margin-bottom: 20px; background: var(--bg-app); padding: 12px; border-radius: 8px; border: 1px solid var(--border-main); box-shadow: 0 1px 2px var(--shadow-color);}
.glossary-add-form input { width: 100%; box-sizing: border-box; background: var(--bg-input); border: 1px solid var(--border-input); border-radius: 4px; color: var(--text-main); padding: 8px 10px; margin-bottom: 8px; outline: none; font-size: 13px; transition: 0.2s;}
.glossary-add-form input:focus { border-color: var(--accent); background: var(--bg-input-focus);}
.glossary-add-form .btn { width: 100%; justify-content: center; }

.glossary-list { display: flex; flex-direction: column; gap: 8px; }
.glossary-card { background: var(--bg-app); padding: 10px 12px; border-radius: 6px; border: 1px solid var(--border-main); display: flex; align-items: center; justify-content: space-between; box-shadow: 0 1px 2px var(--shadow-color);}
.glos-terms { flex: 1; overflow: hidden; }
.glos-original { color: var(--text-secondary); font-weight: 600; font-size: 12px; margin-bottom: 2px; }
.glos-translation { color: var(--success-text); font-size: 13px; font-weight: 500; }
.glos-del-btn { background: transparent; border: none; color: #ef4444; font-size: 11px; cursor: pointer; padding: 4px; opacity: 0.6; transition: 0.2s; font-weight: bold; }
.glos-del-btn:hover { opacity: 1; background: var(--error-bg); border-radius: 4px; }

/* === MODALS === */
.modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.6); backdrop-filter: blur(2px); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal-content { background: var(--bg-app); width: 600px; border-radius: 12px; border: 1px solid var(--border-main); overflow: hidden; box-shadow: 0 20px 25px -5px rgba(0,0,0,0.2);}
.modal-header { padding: 16px 20px; background: var(--bg-panel); display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border-main); }
.modal-header h2 { margin: 0; font-size: 16px; color: var(--text-main); font-weight: 600; }
.close-btn { background: none; border: 1px solid var(--border-input); border-radius: 4px; padding: 4px 12px; font-size: 13px; color: var(--text-muted); cursor: pointer; transition: 0.2s; font-weight: 600;}
.close-btn:hover { color: var(--text-main); background: var(--bg-base); }

.modal-body { padding: 20px; display: flex; flex-direction: column; gap: 20px; }

.setting-row { display: flex; justify-content: space-between; align-items: center; padding: 10px 0; border-bottom: 1px solid var(--border-main); }
.setting-row:last-child { border-bottom: none; }
.setting-row label { font-size: 14px; font-weight: 500; color: var(--text-main); }
.settings-select { padding: 6px 10px; border: 1px solid var(--border-input); border-radius: 6px; background: var(--bg-app); color: var(--text-main); font-size: 13px; outline: none; cursor: pointer; min-width: 140px; }
.settings-select:focus { border-color: var(--accent); }

.tabs-header { display: flex; gap: 10px; border-bottom: 1px solid var(--border-main); padding-bottom: 10px; }
.tab-btn { background: none; border: none; font-size: 14px; font-weight: 600; color: var(--text-muted); cursor: pointer; padding: 6px 12px; border-radius: 4px; transition: 0.2s; }
.tab-btn:hover { color: var(--text-main); background: var(--bg-base); }
.tab-btn.active { color: var(--accent); background: rgba(37,99,235,0.1); }

.step-box { background: var(--bg-panel); padding: 15px; border-radius: 8px; border: 1px solid var(--border-main); }
.step-box h3 { margin: 0 0 8px 0; font-size: 13px; color: var(--accent); text-transform: uppercase; letter-spacing: 0.5px; font-weight: 700; }
.step-box p { font-size: 13px; color: var(--text-secondary); margin-bottom: 12px; }

.turbo-controls { display: flex; align-items: center; gap: 10px; font-size: 13px; color: var(--text-main);}
.turbo-controls input { background: var(--bg-app); border: 1px solid var(--border-input); color: var(--text-main); padding: 6px 8px; width: 60px; border-radius: 4px; outline: none;}
.turbo-controls input:focus { border-color: var(--accent); }

textarea { width: 100%; height: 140px; background: var(--bg-app); border: 1px solid var(--border-input); color: var(--text-main); padding: 12px; border-radius: 6px; font-family: 'Consolas', monospace; resize: none; box-sizing: border-box; margin-bottom: 10px; outline: none; font-size: 13px; line-height: 1.5;}
textarea:focus { border-color: var(--accent); box-shadow: 0 0 0 2px rgba(37,99,235,0.1);}
.apply-ai-btn { width: 100%; padding: 10px; font-size: 14px; font-weight: 600;}
</style>