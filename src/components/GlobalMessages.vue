<template>
  <div v-if="error" class="global-msg error-msg">
    <span style="white-space: pre-wrap;">{{ error }}</span>
    <button class="icon-text-btn" style="color: inherit;" @click="closeMsg">×</button>
  </div>
  <div v-if="successMsg" class="global-msg success-msg">
    <span>{{ successMsg }}</span>
    <button class="icon-text-btn" style="color: inherit;" @click="closeMsg">×</button>
  </div>
</template>

<script setup>
import { error, successMsg, closeMsg } from '../store.js';
</script>