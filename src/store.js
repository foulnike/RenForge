import { ref, watch } from 'vue';

// Настройки приложения
export const uiLang = ref(localStorage.getItem('renforge_ui_lang') || 'ru');
export const targetLang = ref(localStorage.getItem('renforge_target_lang') || 'russian');
export const uiTheme = ref(localStorage.getItem('renforge_ui_theme') || 'dark');
export const currentMode = ref('dashboard');
export const activePopover = ref(null);
export const showFontPanel = ref(false);
export const isAiModalOpen = ref(false);

// Уведомления
export const error = ref('');
export const successMsg = ref('');
let msgTimeoutId = null;

export function showMsg(type, text, timeout = 8000) {
  if (msgTimeoutId) clearTimeout(msgTimeoutId);
  if (type === 'error') { error.value = text; successMsg.value = ''; }
  else { successMsg.value = text; error.value = ''; }
  if (timeout > 0) msgTimeoutId = setTimeout(closeMsg, timeout);
}

export function closeMsg() {
  if (msgTimeoutId) clearTimeout(msgTimeoutId);
  error.value = ''; successMsg.value = '';
}

// Данные проекта
export const projectPath = ref('');
export const isProcessing = ref(false);
export const projectFiles = ref({ rpa_files:[], rpyc_files:[], rpy_files:[], tl_files:[], manual_tl_files:[] });
export const fileStats = ref({});
export const charMap = ref({});

// Редактор переводов
export const parsedBlocks = ref([]);
export const currentFilePath = ref('');
export const rawFileText = ref('');
export const showRawView = ref(false);
export const isEditorLoading = ref(false);
export const hideTranslated = ref(false);
export const focusedBlockId = ref(null);
export const newTerm = ref({ original: '', translation: '' });

// Редактор Fallback-скриптов
export const fallbackLines = ref([]);
export const fallbackRelPath = ref('');
export const fallbackIsEditMode = ref(false);

// Локально сохраненные списки (в LocalStorage)
export const hiddenFiles = ref([]);
export const completedFiles = ref([]);
export const glossary = ref([]);
export const hiddenImages = ref([]);
export const hiddenAudio = ref([]);
export const hiddenFolders = ref([]); 
export const hiddenGraphNodes = ref([]);
export const showHidden = ref(false);
export const showHiddenMedia = ref(false);
export const searchQuery = ref('');
export const searchResults = ref([]);

// Утилиты
export function getProjectKey(baseKey) {
  if (!projectPath.value) return null;
  return `${baseKey}_${projectPath.value.replace(/[^a-zA-Z0-9]/g, '_')}`;
}

function safeParseJSON(key, defaultVal = '[]') {
  try { return JSON.parse(localStorage.getItem(key) || defaultVal); } 
  catch (e) { return JSON.parse(defaultVal); }
}

export function loadProjectSettings() {
  if (!projectPath.value) return;
  hiddenFiles.value = safeParseJSON(getProjectKey('renforge_hidden'));
  completedFiles.value = safeParseJSON(getProjectKey('renforge_completed'));
  glossary.value = safeParseJSON(getProjectKey('renforge_glossary'));
  hiddenImages.value = safeParseJSON(getProjectKey('renforge_hidden_img'));
  hiddenAudio.value = safeParseJSON(getProjectKey('renforge_hidden_aud'));
  hiddenFolders.value = safeParseJSON(getProjectKey('renforge_hidden_folders'));
  hiddenGraphNodes.value = safeParseJSON(getProjectKey('renforge_hidden_graph_nodes'));
}

// Авто-сохранение списков
const watchConfig = { deep: true };
watch(hiddenFiles, (val) => { const k = getProjectKey('renforge_hidden'); if(k) localStorage.setItem(k, JSON.stringify(val)); }, watchConfig);
watch(completedFiles, (val) => { const k = getProjectKey('renforge_completed'); if(k) localStorage.setItem(k, JSON.stringify(val)); }, watchConfig);
watch(glossary, (val) => { const k = getProjectKey('renforge_glossary'); if(k) localStorage.setItem(k, JSON.stringify(val)); }, watchConfig);
watch(hiddenImages, (val) => { const k = getProjectKey('renforge_hidden_img'); if(k) localStorage.setItem(k, JSON.stringify(val)); }, watchConfig);
watch(hiddenAudio, (val) => { const k = getProjectKey('renforge_hidden_aud'); if(k) localStorage.setItem(k, JSON.stringify(val)); }, watchConfig);
watch(hiddenFolders, (val) => { const k = getProjectKey('renforge_hidden_folders'); if(k) localStorage.setItem(k, JSON.stringify(val)); }, watchConfig);
watch(hiddenGraphNodes, (val) => { const k = getProjectKey('renforge_hidden_graph_nodes'); if(k) localStorage.setItem(k, JSON.stringify(val)); }, watchConfig);

export function getFileName(fullPath) { 
  if (!fullPath) return '';
  return fullPath.split(/[/\\]/).pop(); 
}

export function getRelativePath(fullPath) {
  if (!projectPath.value) return fullPath;
  const normalizedFull = fullPath.replace(/\\/g, '/');
  const normalizedProj = projectPath.value.replace(/\\/g, '/');
  return normalizedFull.replace(normalizedProj, '').replace(/^\//, '');
}

export function getFolderFromPath(relPath) {
  let normalized = relPath.replace(/\\/g, '/');
  const parts = normalized.split('/');
  if (parts.length > 1) { parts.pop(); return parts.join('/'); }
  return '/';
}