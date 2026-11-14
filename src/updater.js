import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { appUpdateInfo, showAppUpdateModal } from './store.js';
import { editorDirty, showMsg } from './store.js';
import { t } from './locales.js';
let pendingUpdate = null;
export async function checkForAppUpdate({ interactive = false } = {}) { try { const update = await check({ timeout: 10000 }); if (!update) return null; pendingUpdate = update; appUpdateInfo.value = { version: update.version, currentVersion: update.currentVersion, body: update.body || '', installing: false }; showAppUpdateModal.value = true; return update; } catch (e) { if (interactive) throw e; console.debug('Update check failed:', e); return null; } }
export async function installAppUpdate() { if (!pendingUpdate) return; if (editorDirty.value) { showMsg('error', t('app_update_unsaved')); return; } appUpdateInfo.value.installing = true; await pendingUpdate.downloadAndInstall(); await relaunch(); }
