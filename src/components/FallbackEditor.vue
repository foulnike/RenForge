<template>
  <div class="workspace fallback-workspace">
    <div class="fallback-scroll-area">
      <div v-for="line in fallbackLines" :key="line.index" class="fallback-line">
        <span class="fallback-line-num">{{ line.index }}</span>
        <span class="fallback-line-content">
          <template v-for="(part, idx) in line.parts" :key="idx">
            <span v-if="part.type === 'code'" class="token-code">{{ part.text }}</span>
            <span v-else-if="part.type === 'string'" 
                  class="token-string" 
                  :class="{ 
                      'selected': part.selected, 
                      'candidate': part.canAuto && !part.selected, 
                      'suspicious': part.suspicious && !part.selected 
                  }"
                  :title="part.canAuto ? t('tt_candidate') : (part.suspicious ? t('tt_suspicious') : t('tt_system'))"
                  @click="toggleFallbackToken(part)">
              {{ part.text }}
            </span>
          </template>
        </span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { fallbackLines } from '../store.js';
import { t } from '../locales.js';

function toggleFallbackToken(clickedPart) {
    if (!clickedPart.groupId) return;
    const newState = !clickedPart.selected;
    fallbackLines.value.forEach(line => {
        line.parts.forEach(p => {
            if (p.groupId === clickedPart.groupId) p.selected = newState;
        });
    });
}
</script>