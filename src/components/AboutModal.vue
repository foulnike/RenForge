<template>
  <div class="modal-overlay" @click.self="close">
    <div class="modal-content about-modal">
      <div class="modal-header">
        <h2><img :src="appLogo" class="about-logo" alt="" /> {{ t('about_title') }}</h2>
        <button class="icon-close-btn" @click="close" :title="t('close')"><Icon name="x" :size="18" /></button>
      </div>

      <div class="about-body">
        <div class="about-app">
          <div class="about-app-head">
            <div class="about-app-name"><span class="logo-ren">Ren</span><span class="logo-forge">Forge</span> <span class="about-ver">v{{ APP_VERSION }}</span></div>
            <button class="btn btn-secondary about-update-btn" @click="checkUpdates" :disabled="checking">{{ checking ? t('app_update_checking') : t('app_update_check') }}</button>
          </div>
          <div class="about-app-sub">GPL-3.0 · © foulnike</div>
          <a class="about-link" href="https://github.com/foulnike/RenForge" target="_blank" rel="noopener">github.com/foulnike/RenForge</a>
        </div>

        <div class="about-section-title">{{ t('about_third_party') }}</div>
        <pre class="about-notices">{{ notices }}</pre>
      </div>

      <div class="about-footer">
        <span class="about-foot-note">{{ t('about_licenses_note') }}</span>
        <span class="about-foot-spacer"></span>
        <button class="btn btn-secondary" @click="openFolder"><Icon name="file" :size="14" /> {{ t('about_open_licenses') }}</button>
        <button class="btn btn-primary" @click="close">{{ t('close') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { resolveResource } from '@tauri-apps/api/path';
import { showAboutModal, showMsg } from '../store.js';
import { t } from '../locales.js';
import Icon from './Icon.vue';
import appLogo from '../assets/app-logo.png';
import { checkForAppUpdate } from '../updater.js';

const APP_VERSION = '1.3.2';

// Английская атрибуция-фолбэк (на случай, если ресурс не прочитался, напр. в dev).
const FALLBACK = `RenForge v${APP_VERSION} — GPL-3.0 — (c) foulnike
https://github.com/foulnike/RenForge

Third-party components:
- unrpa 2.3.0 — GPL-3.0 — (c) Gareth Latty (Lattyware)
  https://github.com/Lattyware/unrpa
- unrpyc — MIT (+ BSD-3 for codegen.py) — (c) Yuri K. Schlesner, CensoredUsername, Jackmcbarn
  https://github.com/CensoredUsername/unrpyc

Full license texts are in the application's licenses/ folder.`;

const notices = ref(FALLBACK);
const checking = ref(false);

onMounted(async () => {
  try {
    const p = await resolveResource('licenses/THIRD_PARTY_NOTICES.txt');
    const txt = await invoke('read_text_file', { path: p });
    if (txt && txt.trim()) notices.value = txt;
  } catch (e) {
    // оставляем FALLBACK (dev / ресурс недоступен)
  }
});

async function openFolder() {
  try {
    const dir = await resolveResource('licenses');
    await invoke('open_in_explorer', { path: dir });
  } catch (e) {
    showMsg('error', (e && e.toString) ? e.toString() : String(e));
  }
}

async function checkUpdates() {
  checking.value = true;
  try {
    const update = await checkForAppUpdate({ interactive: true });
    if (!update) showMsg('success', t('app_update_latest'), 4000);
  } catch (e) {
    showMsg('error', t('app_update_failed'), 6000);
  } finally {
    checking.value = false;
  }
}

function close() { showAboutModal.value = false; }
</script>

<style scoped>
.about-modal { width: 600px; max-width: 92vw; display: flex; flex-direction: column; max-height: 86vh; }
.about-logo { width: 22px; height: 22px; border-radius: 5px; vertical-align: middle; margin-right: 4px; }
.about-body { padding: 18px 20px; display: flex; flex-direction: column; gap: 14px; overflow-y: auto; }
.about-app { display: flex; flex-direction: column; gap: 4px; }
.about-app-head { display: flex; align-items: center; gap: 14px; flex-wrap: wrap; min-height: 30px; }
.about-app-name { display: inline-flex; align-items: center; height: 30px; font-size: 22px; line-height: 30px; font-weight: 800; letter-spacing: -0.02em; }
.about-update-btn { display: inline-flex; align-items: center; justify-content: center; height: 30px; box-sizing: border-box; padding: 0 14px; white-space: nowrap; }
.about-ver { margin-left: 5px; font-size: 13px; line-height: 1; font-weight: 700; color: var(--accent); align-self: center; }
.about-app-sub { font-size: 13px; color: var(--text-secondary); }
.about-link { font-size: 13px; color: var(--accent); text-decoration: none; }
.about-link:hover { text-decoration: underline; }
.about-section-title { font-size: 11px; font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; color: var(--text-muted); }
.about-notices {
  margin: 0; white-space: pre-wrap; word-break: break-word;
  font-family: ui-monospace, 'Cascadia Code', Consolas, monospace; font-size: 12px; line-height: 1.55;
  color: var(--code-text); background: var(--code-bg);
  border: 1px solid var(--border-input); border-radius: var(--radius-md, 8px); padding: 12px 14px;
  max-height: 46vh; overflow: auto;
}
.about-footer { display: flex; align-items: center; gap: 10px; padding: 13px 20px; border-top: 1px solid var(--border-main); background: var(--bg-panel); border-bottom-left-radius: inherit; border-bottom-right-radius: inherit; }
.about-foot-note { font-size: 12px; color: var(--text-muted); }
.about-foot-spacer { flex: 1; }
</style>
