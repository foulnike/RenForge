<template>
  <div class="workspace">
    <div v-if="isEditorLoading" class="editor-loading">
      <div class="spinner"></div>
      <p style="font-weight: 600;">{{ t('loading_editor') }}</p>
    </div>

    <template v-else>
      <div v-if="showRawView || parsedBlocks.length === 0" class="raw-preview">
        <div class="raw-preview-header">
          <h3 v-if="showRawView">{{ t('raw_view_mode') }}</h3>
          <h3 v-else>{{ t('raw_no_blocks') }}</h3>
        </div>
        <pre class="raw-code">{{ rawFileText }}</pre>
      </div>

      <template v-else>
        <!-- SIDEBAR -->
        <aside class="sidebar">
          <div class="sidebar-title">{{ t('file_structure') }}</div>
          <div style="padding: 0 15px 10px; border-bottom: 1px solid var(--border-main);">
            <label class="toggle-hidden" style="margin: 0;">
              <input type="checkbox" v-model="hideTranslated" />
              {{ t('hide_translated') }}
            </label>
          </div>
          <div class="sidebar-list">
            <template v-for="(block, index) in parsedBlocks" :key="'nav-' + block.id">
              <div class="sidebar-item" 
                   @click="scrollToBlock(block.id)"
                   v-show="!hideTranslated || getBlockStatus(block) !== 'translated' || focusedBlockId === block.id">
                <span class="status-dot" :class="getBlockStatus(block)"></span>
                <span class="sidebar-index">{{ index + 1 }}</span>
                <span class="sidebar-id">{{ block.id }}</span>
              </div>
            </template>
          </div>
        </aside>

        <!-- MAIN EDITOR -->
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

        <!-- GLOSSARY SIDEBAR -->
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
    </template>
  </div>
</template>

<script setup>
import { 
    isEditorLoading, showRawView, parsedBlocks, rawFileText, hideTranslated, 
    focusedBlockId, charMap, glossary, newTerm 
} from '../store.js';
import { getBlockStatus, getMissingTags } from '../actions.js';
import { t } from '../locales.js';

function scrollToBlock(id) { 
    const el = document.getElementById('block-' + id); 
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' }); 
}

function addGlossaryTerm() {
    if (!newTerm.value.original.trim() || !newTerm.value.translation.trim()) return;
    glossary.value.push({ original: newTerm.value.original.trim(), translation: newTerm.value.translation.trim() });
    newTerm.value = { original: '', translation: '' };
}

function removeGlossaryTerm(index) {
    glossary.value.splice(index, 1);
}

function escapeHtml(unsafe) {
    if (!unsafe) return '';
    return unsafe.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#039;");
}

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
</script>