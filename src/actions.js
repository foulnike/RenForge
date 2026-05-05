import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';
import { 
  projectPath, targetLang, isProcessing, projectFiles, charMap, fileStats, 
  showMsg, currentMode, isEditorLoading, showRawView, currentFilePath, rawFileText, 
  parsedBlocks, fallbackLines, fallbackRelPath, fallbackIsEditMode, hideTranslated 
} from './store.js';
import { t } from './locales.js';

export async function refreshProject() {
  try { 
      isProcessing.value = true; 
      projectFiles.value = await invoke('scan_project', { path: projectPath.value, targetLang: targetLang.value }); 
      charMap.value = await invoke('get_character_mapping', { projectPath: projectPath.value });
      fileStats.value = await invoke('get_translation_stats', { projectPath: projectPath.value });
  } 
  catch (e) { showMsg('error', `Error: ${e}`); } 
  finally { isProcessing.value = false; }
}

export async function prepareProject() {
    isProcessing.value = true;
    const rpa = projectFiles.value.rpa_files;
    let unpackCount = 0;
    for (let i = 0; i < rpa.length; i++) {
        showMsg('success', `${t('unpacking')} ${i+1} / ${rpa.length}...`, 0);
        try { await invoke('run_unrpa', { filePath: rpa[i] }); unpackCount++; } catch (e) { console.error(e); }
    }
    if (unpackCount > 0) { projectFiles.value = await invoke('scan_project', { path: projectPath.value, targetLang: targetLang.value }); }

    const newRpyc = projectFiles.value.rpyc_files;
    let decompCount = 0;
    for (let i = 0; i < newRpyc.length; i++) {
        const expectedRpyPath = newRpyc[i].replace(/\.rpyc$/, '.rpy');
        if (!projectFiles.value.rpy_files.includes(expectedRpyPath)) {
            showMsg('success', `${t('decompiling')} ${i+1} / ${newRpyc.length}...`, 0);
            try { await invoke('run_unrpyc', { filePath: newRpyc[i] }); decompCount++; } catch (e) { console.error(e); }
        }
    }
    
    if (decompCount > 0) showMsg('success', `${t('msg_decomp_done')} ${decompCount}`, 5000);
    else if (unpackCount > 0) showMsg('success', `${t('msg_unpack_done')} ${unpackCount}`, 5000);
    else showMsg('success', t('status_done'), 3000); 
    
    await refreshProject();
}

export async function generateTranslations() {
  try {
    isProcessing.value = true; showMsg('success', t('msg_engine_working'), 0);
    const res = await invoke('generate_translations', { path: projectPath.value, targetLang: targetLang.value });
    showMsg('success', res); await refreshProject();
  } catch (e) { showMsg('error', e, 15000); } finally { isProcessing.value = false; }
}

// -- Редактор: Парсинг и сохранение --
function extractTags(text) { return text.match(/(\[.*?\]|\{.*?\})/g) ||[]; }
export function getMissingTags(block) { return extractTags(block.original).filter(tag => !block.translation.includes(tag)); }

export function getBlockStatus(block) {
  if (getMissingTags(block).length > 0) return 'error'; 
  if (!block.translation.trim() || block.translation === block.original) return 'untranslated'; 
  return 'translated'; 
}

function extractDialogueParts(line) {
  const match = line.match(/^([^"]*)"((?:\\.|[^"\\])*)"(.*)$/);
  if (match) return { prefix: match[1], content: match[2], suffix: match[3] };
  return { prefix: '', content: line, suffix: '' };
}

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
    
    if (idMatch) { currentId = idMatch[1]; isStrings = (currentId === 'strings'); tempOriginal = ''; continue; }
    if (!currentId) continue;

    if (isStrings) {
        if (trimLine.startsWith('old ') && line.includes('"')) {
            tempOriginal = extractDialogueParts(trimLine).content.replace(/\\"/g, '"');
        } else if (trimLine.startsWith('new ') && line.includes('"')) {
            const p = extractDialogueParts(line);
            const block = { id: `string_${i}`, original: tempOriginal, translation: p.content.replace(/\\"/g, '"'), prefix: p.prefix, suffix: p.suffix, lineIndex: i };
            blocks.push(block);
        }
    } else {
        let cleanLine = trimLine.startsWith('#') ? trimLine.substring(1).trim() : trimLine;
        if (/^(voice|play|stop|scene|show|hide|window|pause|\$|jump|call|return)(?:\s|\(|$)/.test(cleanLine)) continue;

        if (trimLine.startsWith('#') && line.includes('"')) {
            tempOriginal = extractDialogueParts(cleanLine).content.replace(/\\"/g, '"');
        } else if (trimLine !== '' && !trimLine.startsWith('#') && line.includes('"')) {
            const p = extractDialogueParts(line);
            const block = { id: currentId, original: tempOriginal, translation: p.content.replace(/\\"/g, '"'), prefix: p.prefix, suffix: p.suffix, lineIndex: i };
            blocks.push(block);
            currentId = null; 
        }
    }
  }
  return blocks;
}

export async function openEditor(filePath) {
  try {
    isEditorLoading.value = true;
    currentMode.value = 'editor';
    showRawView.value = false;
    hideTranslated.value = false;
    currentFilePath.value = filePath;
    const text = await invoke('read_rpy_file', { projectPath: projectPath.value, filePath: filePath });
    rawFileText.value = text.replace(/\r\n/g, '\n'); 
    parsedBlocks.value = parseRpy(rawFileText.value, filePath); 
  } catch (e) { showMsg('error', `Error: ${e}`); currentMode.value = 'dashboard'; } 
  finally { isEditorLoading.value = false; }
}

export async function viewOriginalScript(filePath) {
  try {
    isEditorLoading.value = true;
    currentMode.value = 'editor';
    showRawView.value = true;
    currentFilePath.value = filePath;
    const text = await invoke('read_rpy_file', { projectPath: projectPath.value, filePath: filePath });
    rawFileText.value = text.replace(/\r\n/g, '\n'); 
    parsedBlocks.value =[]; 
  } catch (e) { showMsg('error', `Error: ${e}`); } 
  finally { isEditorLoading.value = false; }
}

export async function saveFile() {
  if (!currentFilePath.value) return;
  const hasErrors = parsedBlocks.value.some(block => getBlockStatus(block) === 'error');
  if (hasErrors) { showMsg('error', t('msg_cannot_save_errors')); return; }
  
  try {
    const lines = rawFileText.value.split('\n');
    const entriesToSave =[];
    
    for (const block of parsedBlocks.value) {
        const escaped = block.translation.replace(/"/g, '\\"');
        lines[block.lineIndex] = `${block.prefix}"${escaped}"${block.suffix}`;
        entriesToSave.push({ id: block.id, file_path: currentFilePath.value, original: block.original, translation: block.translation, status: getBlockStatus(block) });
    }
    
    await invoke('upsert_translations_batch', { projectPath: projectPath.value, entries: entriesToSave });
    const newFileContent = lines.join('\n');
    await invoke('write_rpy_file', { projectPath: projectPath.value, filePath: currentFilePath.value, content: newFileContent });
    
    fileStats.value[currentFilePath.value] = { 
        total: parsedBlocks.value.length, 
        translated: parsedBlocks.value.filter(b => getBlockStatus(b) === 'translated').length 
    };
    showMsg('success', t('msg_file_saved'));
  } catch (e) { showMsg('error', `Error: ${e}`); }
}

// -- Fallback Editor --
export async function openFallbackEditor(f) {
    try {
        isProcessing.value = true;
        fallbackIsEditMode.value = f.isSynced || false;
        
        let origRelPath = f.name + '.rpy';
        if (f.rpyPath) {
            const rel = f.rpyPath.replace(/\\/g, '/').replace(projectPath.value.replace(/\\/g, '/'), '').replace(/^\//, '');
            origRelPath = rel.startsWith('game/') ? rel.substring(5) : rel;
        } else if (f.tlPath) {
            const match = f.tlPath.replace(/\\/g, '/').match(new RegExp(`tl/${targetLang.value}/(.*)`));
            if (match) origRelPath = match[1];
        }
        fallbackRelPath.value = origRelPath;
        
        const targetRpyPath = f.rpyPath || (projectPath.value + '/game/' + origRelPath);
        const text = await invoke('read_rpy_file', { projectPath: projectPath.value, filePath: targetRpyPath });
        
        const parsed = parseFallbackFile(text.replace(/\r\n/g, '\n'));

        if (fallbackIsEditMode.value) {
            try {
                const tlFilePath = projectPath.value + '/game/tl/' + targetLang.value + '/' + origRelPath;
                const tlText = await invoke('read_rpy_file', { projectPath: projectPath.value, filePath: tlFilePath });
                if (tlText) {
                    parsed.forEach(line => {
                        line.parts.forEach(part => { if (part.type === 'string' && part.fullRaw && tlText.includes(part.fullRaw)) part.selected = true; });
                    });
                }
            } catch(e) {}
        }
        fallbackLines.value = parsed;
        currentMode.value = 'fallback-editor';
    } catch(e) { showMsg('error', 'Ошибка: ' + e.toString()); } finally { isProcessing.value = false; }
}

function parseFallbackFile(text) {
    const tokenRegex = /("[^"\\]*(?:\\.[^"\\]*)*"|'[^'\\]*(?:\\.[^'\\]*)*')/g;
    const matches =[];
    let match;
    while ((match = tokenRegex.exec(text)) !== null) { matches.push({ start: match.index, end: match.index + match[0].length, raw: match[0], id: 'str_' + match.index }); }

    const textLines = text.split('\n');
    let currentLineStart = 0; let matchIdx = 0;

    return textLines.map((lineStr, idx) => {
        const lineEnd = currentLineStart + lineStr.length;
        let parts =[]; let cursor = currentLineStart;

        while (matchIdx < matches.length && matches[matchIdx].start < lineEnd) {
            const m = matches[matchIdx];
            if (m.start > cursor) { parts.push({ type: 'code', text: text.substring(cursor, m.start) }); cursor = m.start; }

            const partEnd = Math.min(m.end, lineEnd);
            const partText = text.substring(cursor, partEnd);
            const innerText = m.raw.slice(1, -1);
            let canAuto = false; let suspicious = false;
            
            if (innerText.trim() !== '') {
                const textClean = innerText.replace(/<\/?[^>]+>|\{\/?[^}]+\}|\[[^\]]+\]/g, '');
                const isExt = /\.(png|jpg|jpeg|webp|ogg|mp3|wav|ttf|otf|rpy|rpyc|webm|mp4|txt|csv)$/i.test(innerText);
                const isHex = /^#[0-9a-fA-F]{3,8}$/.test(innerText);
                const hasLetters = /\p{L}/u.test(textClean);
                const isTech = /^[a-z_][a-zA-Z0-9_]*$/.test(innerText) || /%[sdefgi]/.test(innerText);
                const isPath = textClean.includes('/') || textClean.includes('\\');
                const isShort = textClean.trim().length <= 1;

                if (isExt || isHex || !hasLetters) { canAuto = false; suspicious = false; } 
                else if (isTech || isPath || isShort) { canAuto = false; suspicious = true; } 
                else { canAuto = true; suspicious = false; }
            }
            parts.push({ type: 'string', text: partText, fullRaw: m.raw, groupId: m.id, selected: false, canAuto, suspicious });
            cursor = partEnd;
            if (m.end <= lineEnd) matchIdx++; else break;
        }
        if (cursor < lineEnd) parts.push({ type: 'code', text: text.substring(cursor, lineEnd) });
        currentLineStart = lineEnd + 1;
        return { index: idx + 1, parts };
    });
}

// -- Экспорт / Импорт --
export async function exportCSV() {
    let csvContent = "ID;Original;Translation\n";
    parsedBlocks.value.forEach(b => {
        const orig = b.original.replace(/"/g, '""').replace(/\n/g, "[BR]");
        let tran = (b.translation || "").replace(/"/g, '""').replace(/\n/g, "[BR]");
        if (/^[=+\-@]/.test(tran)) { tran = "'" + tran; }
        csvContent += `"${b.id}";"${orig}";"${tran}"\n`;
    });
    try {
        const savePath = await save({ filters:[{ name: 'CSV', extensions:['csv'] }] });
        if (savePath) { await invoke('write_text_file', { path: savePath, content: csvContent }); showMsg('success', t('msg_csv_exported')); }
    } catch (e) { showMsg('error', `Error: ${e}`); }
}

export async function importCSV() {
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
                if (tran.startsWith("'") && /^[=+\-@]/.test(tran.substring(1))) tran = tran.substring(1);
                const block = parsedBlocks.value.find(b => b.id === id);
                if (block && tran) { block.translation = tran; updatedCount++; }
            }
        }
        
        
        if (currentFilePath.value) {
            fileStats.value[currentFilePath.value] = { 
                total: parsedBlocks.value.length, 
                translated: parsedBlocks.value.filter(b => getBlockStatus(b) === 'translated').length 
            };
        }
        
        showMsg('success', `${t('msg_csv_imported')} ${updatedCount}.`);
    } catch (e) { showMsg('error', `Error: ${e}`); }
}

export async function exportJSON() {
    const data = parsedBlocks.value.map(b => ({ id: b.id, original: b.original, translation: b.translation }));
    try {
        const savePath = await save({ filters:[{ name: 'JSON', extensions:['json'] }] });
        if (savePath) { await invoke('write_text_file', { path: savePath, content: JSON.stringify(data, null, 2) }); showMsg('success', t('msg_json_exported')); }
    } catch (e) { showMsg('error', `Error: ${e}`); }
}

export async function importJSON() {
    try {
        const selected = await open({ multiple: false, filters:[{ name: 'JSON', extensions: ['json'] }] });
        if (!selected) return;
        const jsonContent = await invoke('read_text_file', { path: selected });
        const data = JSON.parse(jsonContent);
        let updatedCount = 0;
        data.forEach(item => {
            const block = parsedBlocks.value.find(b => b.id === item.id);
            if (block && item.translation) { block.translation = item.translation; updatedCount++; }
        });
        
        
        if (currentFilePath.value) {
            fileStats.value[currentFilePath.value] = { 
                total: parsedBlocks.value.length, 
                translated: parsedBlocks.value.filter(b => getBlockStatus(b) === 'translated').length 
            };
        }
        
        showMsg('success', `${t('msg_json_imported')} ${updatedCount}.`);
    } catch (e) { showMsg('error', `Error: ${e}`); }
}