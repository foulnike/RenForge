<template>
  <div class="modal-overlay" @click.self="isAiModalOpen = false">
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
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { isAiModalOpen, parsedBlocks, targetLang, glossary, charMap, showMsg, currentFilePath, fileStats } from '../store.js';
import { getBlockStatus } from '../actions.js';
import { t } from '../locales.js';

const aiTab = ref('ollama');
const aiStart = ref(1);
const aiEnd = ref(30);
const aiInput = ref('');
const currentAiBatch = ref([]);

const ollamaUrl = ref(localStorage.getItem('renforge_ollama_url') || 'http://localhost:11434');
const ollamaModel = ref(localStorage.getItem('renforge_ollama_model') || 'llama3');
const isOllamaTranslating = ref(false);

onMounted(() => {
    aiEnd.value = Math.min(30, parsedBlocks.value.length);
});

function updateStats() {
    if (!currentFilePath.value) return;
    fileStats.value[currentFilePath.value] = { 
        total: parsedBlocks.value.length, 
        translated: parsedBlocks.value.filter(b => getBlockStatus(b) === 'translated').length 
    };
}

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
    prompt += `1. PRESERVE ALL TAGS AND VARIABLES! If the original has[name],[player], {b}, {color=#f00}, \\n, or similar, they MUST remain exactly the same in the translation.\n`;
    prompt += `2. NEVER translate the variable name itself (e.g.[name] stays [name], do NOT translate to[имя]).\n`;
    prompt += `3. Output ONLY the numbered list with translations. No introductory text, no explanations.\n`;
    prompt += `4. You MUST translate exactly ${batch.length} lines and keep their exact original numbering.\n\n`;
    prompt += `5. Speaker names or tags (like[new]:,[Narrator]:) are for context ONLY. DO NOT include them in your output! Just output the translated string.\n\n`;
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

function parseAiResponseAndApply(text) {
    const lines = text.split('\n').map(l => l.replace(/^\d+[\.\)]\s*/, '').trim()).filter(l => l.length > 0);
    let appliedCount = 0;
    currentAiBatch.value.forEach((id, index) => {
        if (lines[index]) {
            const block = parsedBlocks.value.find(b => b.id === id);
            if (block) { block.translation = lines[index]; appliedCount++; }
        }
    });
    return appliedCount;
}

function importAiBatch() {
    const applied = parseAiResponseAndApply(aiInput.value);
    showMsg('success', `${t('msg_ai_applied')} ${applied}`);
    updateStats();
    isAiModalOpen.value = false;
}

async function runLocalLLM() {
    localStorage.setItem('renforge_ollama_url', ollamaUrl.value);
    localStorage.setItem('renforge_ollama_model', ollamaModel.value);
    
    const prompt = await prepareAiBatch(true);
    if (!prompt) return;

    isOllamaTranslating.value = true;
    try {
        const res = await fetch(`${ollamaUrl.value}/api/generate`, {
            method: 'POST', headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({ model: ollamaModel.value, prompt: prompt, stream: false, options: { num_predict: -1, num_ctx: 8192 } })
        });
        if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
        const data = await res.json();
        const applied = parseAiResponseAndApply(data.response);
        showMsg('success', `${t('msg_ai_applied')} ${applied}`);
        updateStats();
        isAiModalOpen.value = false;
    } catch (e) { showMsg('error', 'Ollama API error: ' + e.message); } 
    finally { isOllamaTranslating.value = false; }
}
</script>