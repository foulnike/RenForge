<template>
  <div class="dashboard">
    <div v-if="!projectPath" class="empty-state">
      <h2>{{ t('empty_state_title') }}</h2>
      <p>{{ t('empty_state_desc1') }}</p>
      <p class="empty-hint">{{ t('empty_state_desc2') }}</p>
    </div>
    
    <div v-else class="dashboard-content">
      <!-- TOP ROW: SEARCH & TRANSLATION BANNER -->
      <div class="dashboard-header-row">
          <div class="search-section" style="flex: 1;">
              <input type="text" v-model="searchQuery" @input="handleSearch" :placeholder="t('search_placeholder')" class="search-input" />
              <div v-if="searchResults.length > 0" class="search-results">
                  <div v-for="res in searchResults" :key="res.id" class="search-res-item" @click="jumpToFile(res)">
                      <span class="res-file">{{ getFileName(res.file_path) }}</span>
                      <span class="res-text"><b>{{ res.original }}</b></span>
                      <span class="res-tran">{{ res.translation || '...' }}</span>
                  </div>
              </div>
          </div>
          
          <div class="translation-banner" v-if="filteredFiles.tl.length === 0" style="flex: 2;">
              <div class="banner-content">
                  <h3>{{ t('no_translation_found') }}</h3>
                  <p v-if="filteredFiles.rpy.length === 0">{{ t('unpack_first') }}</p>
                  <p v-else>{{ t('ready_to_gen') }}</p>
              </div>
              <button class="btn btn-primary generate-btn" :disabled="filteredFiles.rpy.length === 0 || isProcessing" @click="generateTranslations">
                  {{ t('gen_btn') }}
              </button>
          </div>
      </div>

      <div class="dashboard-grid">
        <!-- LEFT: VUE FLOW CANVAS -->
        <div class="canvas-column">
          <div class="column-header column-header-flex">
            <span>{{ t('col_project_map') }}</span>
            <div style="display:flex; gap:10px; align-items:center;">
              <label class="toggle-hidden" v-if="hiddenGraphNodes.length > 0" style="margin: 0; font-size: 12px;">
                <input type="checkbox" v-model="showHiddenGraphNodes" @change="buildFlowGraph">
                {{ t('show_hidden_nodes') }} ({{ hiddenGraphNodes.length }})
              </label>
              <div style="width: 1px; height: 16px; background: var(--border-main); margin: 0 5px;"></div>
              <button v-if="flowNodes.length > 0" class="btn btn-secondary" style="padding: 4px 10px;" @click="resetGraphPositions" :title="t('reset_graph')">{{ t('reset') }}</button>
              <button v-if="projectFiles.rpa_files.length > 0 || projectFiles.rpyc_files.length > 0" class="btn btn-primary" style="padding: 6px 15px;" @click="prepareProject" :disabled="isProcessing">
                 {{ t('prepare_project') }}
              </button>
            </div>
          </div>
          <div class="canvas-container">
            <VueFlow v-model:nodes="flowNodes" :edges="flowEdges" class="renforge-flow" @node-drag-stop="saveGraphPositions" @move-end="onGraphMoveEnd">
              <Background pattern-color="var(--border-input)" />
              <Controls />
              <MiniMap :pannable="true" :zoomable="true" />

              <!-- Node: RPA Archives -->
              <template #node-rpa="{ data }">
                <div class="custom-node node-rpa">
                  <div class="node-title">{{ t('archives_node') }}</div>
                  <div class="node-content nowheel">
                    <div class="node-file" v-for="file in data.files" :key="file">📦 {{ getFileName(file) }}</div>
                    <div v-if="data.files.length === 0" class="node-empty">{{ t('no_archives') }}</div>
                  </div>
                </div>
              </template>

              <!-- Node: Original Scripts Folder -->
              <template #node-orig="{ data }">
                <div class="custom-node node-orig" :class="{'node-faded': data.isHidden}">
                  <div class="node-title" style="display:flex; justify-content:space-between; align-items:center;">
                    <span style="flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-right: 10px;" :title="'📁 ' + data.dirName">📁 {{ data.dirName }}</span>
                    <button class="icon-text-btn" @click="toggleHideGraphNode(data.dirName)">[{{ data.isHidden ? t('btn_show') : t('btn_hide') }}]
                    </button>
                  </div>
                  <div class="node-content nowheel">
                    <div class="node-file interactive-node" v-for="f in data.files" :key="f.name">
                      <span style="display: flex; align-items: center; gap: 6px; flex: 1; overflow: hidden; cursor: pointer;"
                            :class="{ 'file-warning': !f.hasRpy }"
                            @click="f.hasRpy ? viewOriginalScript(f.rpyPath) : null"
                            :title="f.hasRpy ? t('tt_view_orig') : t('tt_needs_decompile')">
                        <span class="file-tag">{{ f.hasRpy ? 'rpy' : 'rpyc' }}</span>
                        <span style="white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">{{ f.name }}</span>
                        <span v-if="!f.hasRpy" class="warning-icon">⚠️</span>
                      </span>
                      
                      <button v-if="f.hasRpy && !f.isSynced && projectFiles.tl_files.length > 0" 
                              class="icon-text-btn" 
                              style="padding: 2px 6px; border: 1px dashed var(--accent); color: var(--accent); font-weight: bold; flex-shrink: 0;"
                              @click.stop="openFallbackEditor(f)" 
                              :title="t('tt_prepare_manual')">[ 📝 ]
                      </button>
                    </div>
                  </div>
                </div>
              </template>

              <!-- Node: TL Scripts Folder -->
              <template #node-tl="{ data }">
                <div class="custom-node node-tl" :class="{'node-faded': data.isHidden}">
                  <div class="node-title" :class="{ 'title-sync': data.isSynced, 'title-warn': !data.isSynced }" style="display:flex; justify-content:space-between; align-items:center;">
                    <span style="flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-right: 10px;" :title="'🌐 tl/' + targetLang + '/' + data.dirName">🌐 tl/{{ targetLang }}/{{ data.dirName }}</span>
                    <button class="icon-text-btn" @click="toggleHideGraphNode(data.dirName)" :style="{ color: data.isSynced ? 'var(--success-text)' : '#b45309' }">[{{ data.isHidden ? t('btn_show') : t('btn_hide') }}]
                    </button>
                  </div>
                  <div class="node-content nowheel">
                    <div class="node-file interactive-node" v-for="f in data.files" :key="f.name" 
                         :class="{ 'file-success': f.isSynced, 'file-error': !f.isSynced }"
                         :title="f.isSynced ? t('tt_synced_tl') : t('tt_missing_tl')"
                         @click="f.isSynced ? openEditor(f.tlPath) : null">
                      <span class="file-tag tag-tl">tl</span>
                      <span style="flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; cursor: pointer;">{{ f.name }}</span>
                      
                      <span v-if="!f.isSynced" class="warning-icon">❌</span>
                      <span v-else-if="f.isManual" class="success-icon" style="cursor:help;" :title="t('tt_manual_gen')">❔</span>
                      <span v-else class="success-icon">✔️</span>

                      <button v-if="f.isSynced" 
                              class="icon-text-btn" 
                              style="margin-left: 5px; color: #f59e0b; padding: 2px 4px; font-weight: bold; flex-shrink: 0;" 
                              @click.stop="openFallbackEditor(f)" 
                              :title="t('tt_fix_manual')">[ ⚠️ ]
                      </button>
                    </div>
                    <div v-if="data.files.length === 0" class="node-empty">{{ t('no_translations') }}</div>
                  </div>
                </div>
              </template>

            </VueFlow>
          </div>
        </div>

        <!-- RIGHT: TRANSLATIONS LIST -->
        <div class="file-column tl-column">
          <div class="column-header column-header-flex tl-header">
            <span>{{ t('col_translations') }}</span>
            <div v-if="filteredFiles.tl.length > 0" style="display:flex; align-items:center; gap:10px;">
              <button class="btn btn-secondary" style="padding: 4px 10px; font-size: 12px;" @click="showFontPanel = !showFontPanel" :class="{active: showFontPanel}">
                {{ t('fonts') }}
              </button>
              <button class="btn apply-patch-btn" @click="applyPatch" :disabled="isProcessing">
                {{ t('apply_patch') }}
              </button>
            </div>
          </div>
          
          <div v-if="showFontPanel" class="font-settings-panel">
              <label style="font-size: 13px; cursor: pointer; display: flex; align-items: center; gap: 6px;">
                  <input type="checkbox" v-model="forceFontFix" style="margin:0;">
                  <strong>{{ t('fix_squares') }}</strong>
              </label>
              <p style="margin: 4px 0 0 0; font-size: 11px; color: var(--text-muted);">{{ t('fix_squares_hint') }}</p>
              
              <div v-if="forceFontFix" style="display: flex; gap: 8px; margin-top: 10px; padding-top: 10px; border-top: 1px solid var(--border-main);">
                  <button class="btn btn-secondary" style="flex: 1;" @click="selectCustomFont">{{ t('select_font') }}</button>
                  <div v-if="customFontName" style="display: flex; flex: 2; justify-content: space-between; align-items: center; background: var(--bg-base); padding: 4px 8px; border-radius: 4px;">
                      <span style="font-size: 11px; color: var(--accent); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" :title="customFontName">{{ customFontName }}</span>
                      <button class="icon-text-btn" style="color: var(--error-text); padding: 2px;" @click="clearCustomFont" :title="t('reset_font')">×</button>
                  </div>
              </div>
          </div>

          <div class="column-content">
            <div class="file-item rpy-item" v-for="file in filteredFiles.tl" :key="file" :class="{ 'is-hidden': hiddenFiles.includes(file), 'is-completed': completedFiles.includes(file) }">
              <input type="checkbox" :checked="completedFiles.includes(file)" @change="toggleFileCompleted(file)" class="done-checkbox"/>
              <div class="file-info">
                <span class="file-name">
                  {{ getFileName(file) }}
                  <span v-if="projectFiles.manual_tl_files?.includes(file)" 
                        style="color: var(--accent); font-size: 12px; font-weight: 800; cursor: help;" 
                        :title="t('tt_manual_gen')">[❔]
                  </span>
                </span>
                <div class="file-stats-wrapper" v-if="fileStats[file]">
                  <span class="file-path">{{ fileStats[file].translated }} / {{ fileStats[file].total }}</span>
                  <div class="progress-bar-bg">
                    <div class="progress-bar-fill" :style="{ width: (fileStats[file].translated / fileStats[file].total * 100) + '%' }"></div>
                  </div>
                </div>
                <span v-else class="file-path">{{ getRelativePath(file) }}</span>
              </div>
              <div class="file-actions">
                <button class="icon-text-btn" @click="toggleHide(file)">[{{ hiddenFiles.includes(file) ? t('btn_show') : t('btn_hide') }}]</button>
                <button class="btn btn-primary" style="padding: 4px 10px; font-size: 12px;" @click="openEditor(file)" :disabled="isProcessing">{{ t('btn_translate') }}</button>
              </div>
            </div>
            <div v-if="filteredFiles.tl.length === 0" class="no-files">{{ t('no_translations') }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';

// Vue Flow Imports
import { VueFlow, useVueFlow } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { Controls } from '@vue-flow/controls';
import { MiniMap } from '@vue-flow/minimap';
import '@vue-flow/core/dist/style.css';
import '@vue-flow/core/dist/theme-default.css';
import '@vue-flow/controls/dist/style.css';
import '@vue-flow/minimap/dist/style.css'; 

import { t } from '../locales.js';
import { 
    projectPath, isProcessing, projectFiles, targetLang, hiddenFiles, completedFiles, 
    showHidden, showFontPanel, getFileName, getRelativePath, fileStats, 
    hiddenGraphNodes, showMsg, getProjectKey, searchQuery, searchResults
} from '../store.js';

import { prepareProject, generateTranslations, openEditor, viewOriginalScript, openFallbackEditor } from '../actions.js';

const showHiddenGraphNodes = ref(false);
const forceFontFix = ref(false);
const customFontPath = ref(localStorage.getItem('renforge_custom_font_path') || '');
const customFontName = ref(localStorage.getItem('renforge_custom_font_name') || '');

const { fitView, setViewport, getViewport } = useVueFlow();
const flowNodes = ref([]);
const flowEdges = ref([]);
let sessionViewport = null;

const filteredFiles = computed(() => {
  const filterFn = (files) => files.filter(f => showHidden.value || !hiddenFiles.value.includes(f));
  return {
    rpa: filterFn(projectFiles.value.rpa_files ||[]),
    rpyc: filterFn(projectFiles.value.rpyc_files ||[]),
    rpy: filterFn(projectFiles.value.rpy_files ||[]),
    tl: filterFn(projectFiles.value.tl_files ||[]),
  };
});

watch(projectFiles, () => { buildFlowGraph(); }, { deep: true });

async function handleSearch() {
  if (searchQuery.value.length < 3) { searchResults.value =[]; return; }
  try { searchResults.value = await invoke('search_in_db', { projectPath: projectPath.value, query: searchQuery.value }); } catch (e) { console.error(e); }
}

async function jumpToFile(result) {
    await openEditor(result.file_path);
    searchQuery.value = ''; searchResults.value =[];
    setTimeout(() => {
        const el = document.getElementById('block-' + result.id); 
        if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' }); 
    }, 500);
}

function clearCustomFont() {
    customFontPath.value = ''; customFontName.value = '';
    localStorage.removeItem('renforge_custom_font_path');
    localStorage.removeItem('renforge_custom_font_name');
}

async function selectCustomFont() {
    try {
        const selected = await openDialog({ multiple: false, filters:[{ name: 'Fonts', extensions:['ttf', 'otf', 'woff', 'woff2'] }] });
        if (selected) {
            customFontPath.value = selected; customFontName.value = getFileName(selected);
            localStorage.setItem('renforge_custom_font_path', selected);
            localStorage.setItem('renforge_custom_font_name', customFontName.value);
        }
    } catch (e) { showMsg('error', e.toString()); }
}

async function applyPatch() {
  try {
    isProcessing.value = true;
    await invoke('apply_renforge_patch', { 
        projectPath: projectPath.value, targetLang: targetLang.value, 
        forceFont: forceFontFix.value, customFontPath: customFontPath.value || null, customFontName: customFontName.value || null
    });
    showMsg('success', t('msg_patch_applied'), 5000);
  } catch (e) { showMsg('error', `Error: ${e}`); } finally { isProcessing.value = false; }
}

function toggleHide(filePath) {
  if (hiddenFiles.value.includes(filePath)) hiddenFiles.value = hiddenFiles.value.filter(p => p !== filePath);
  else hiddenFiles.value.push(filePath);
}

function toggleFileCompleted(filePath) {
  if (completedFiles.value.includes(filePath)) completedFiles.value = completedFiles.value.filter(p => p !== filePath);
  else completedFiles.value.push(filePath);
}

function toggleHideGraphNode(dirName) {
    if (hiddenGraphNodes.value.includes(dirName)) hiddenGraphNodes.value = hiddenGraphNodes.value.filter(n => n !== dirName);
    else hiddenGraphNodes.value.push(dirName);
    buildFlowGraph();
}

function onGraphMoveEnd(event) {
    if (event && event.x !== undefined) sessionViewport = { x: event.x, y: event.y, zoom: event.zoom };
    else sessionViewport = getViewport();
}

function buildFlowGraph() {
    const nodes = []; const edges =[];
    nodes.push({ id: 'node_rpa', type: 'rpa', position: { x: 900, y: 50 }, data: { files: projectFiles.value.rpa_files ||[] } });

    const folders = {};
    const extractDir = (path) => {
        let n = getRelativePath(path); let parts = n.split('/');
        parts.pop(); return parts.length > 0 ? parts.join('/') : 'game';
    };

    (projectFiles.value.rpyc_files ||[]).forEach(f => {
        const dir = extractDir(f); if(!folders[dir]) folders[dir] = {};
        const name = getFileName(f).replace('.rpyc', '');
        if(!folders[dir][name]) folders[dir][name] = { name, hasRpyc: true, hasRpy: false, isSynced: false };
    });
    (projectFiles.value.rpy_files ||[]).forEach(f => {
        const dir = extractDir(f); if(!folders[dir]) folders[dir] = {};
        const name = getFileName(f).replace('.rpy', '');
        if(!folders[dir][name]) folders[dir][name] = { name, rpyPath: f, hasRpyc: false, hasRpy: true, isSynced: false };
        else { folders[dir][name].hasRpy = true; folders[dir][name].rpyPath = f; }
    });
    (projectFiles.value.tl_files ||[]).forEach(f => {
        let n = getRelativePath(f); let match = n.match(new RegExp(`tl/${targetLang.value}/(.*)`));
        if (match) {
            let origPath = 'game/' + match[1];
            if (!match[1].includes('/')) origPath = 'game'; 
            else origPath = 'game/' + match[1].substring(0, match[1].lastIndexOf('/'));
            const name = getFileName(f).replace('.rpy', '');
            if (folders[origPath] && folders[origPath][name]) {
                folders[origPath][name].isSynced = true;
                folders[origPath][name].isManual = (projectFiles.value.manual_tl_files || []).includes(f);
                folders[origPath][name].tlPath = f;
            }
        }
    });

    let colY =[250, 250, 250]; 
    const sortedDirNames = Object.keys(folders).sort();

    sortedDirNames.forEach((dirName, index) => {
        const isHidden = hiddenGraphNodes.value.includes(dirName);
        const fileObjects = Object.values(folders[dirName]).sort((a,b) => a.name.localeCompare(b.name));
        const allSynced = fileObjects.every(f => f.isSynced);
        const tlFiles = fileObjects.filter(f => f.hasRpy);

        let c = index % 3; let startX = 50 + c * 850; let startY = colY[c];

        nodes.push({
            id: `orig_${dirName}`, type: 'orig', position: { x: startX, y: startY },
            hidden: !showHiddenGraphNodes.value && isHidden,
            data: { dirName, files: fileObjects, isHidden: isHidden }
        });

        nodes.push({
            id: `tl_${dirName}`, type: 'tl', position: { x: startX + 350, y: startY },
            hidden: !showHiddenGraphNodes.value && isHidden,
            data: { dirName, files: tlFiles, isSynced: allSynced, isHidden: isHidden }
        });

        edges.push({
            id: `e_${dirName}`, source: `orig_${dirName}`, target: `tl_${dirName}`,
            hidden: !showHiddenGraphNodes.value && isHidden, animated: !allSynced,
            style: { stroke: allSynced ? '#22c55e' : '#f59e0b', strokeWidth: 3 }
        });

        if (!(!showHiddenGraphNodes.value && isHidden)) colY[c] += 100 + (fileObjects.length * 28) + 50;
    });

    const key = getProjectKey('renforge_graph');
    if (key) {
        const savedRaw = localStorage.getItem(key);
        if (savedRaw) {
            try {
                const savedPos = JSON.parse(savedRaw);
                nodes.forEach(n => { if (savedPos[n.id]) n.position = savedPos[n.id]; });
            } catch(e) {}
        }
    }

    flowNodes.value = nodes; flowEdges.value = edges;
    setTimeout(() => { if (sessionViewport) setViewport(sessionViewport); else fitView({ padding: 0.2 }); }, 200);
}

function saveGraphPositions() {
    const key = getProjectKey('renforge_graph'); if (!key) return;
    const posMap = {};
    flowNodes.value.forEach(n => { posMap[n.id] = { x: n.position.x, y: n.position.y }; });
    localStorage.setItem(key, JSON.stringify(posMap));
}

function resetGraphPositions() {
    const key = getProjectKey('renforge_graph'); if (key) localStorage.removeItem(key);
    sessionViewport = null; buildFlowGraph();
}

onMounted(() => { buildFlowGraph(); });
</script>