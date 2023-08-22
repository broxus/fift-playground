<script lang="ts" setup>
import { inject } from 'vue';

import FileSelector from './FileSelector.vue';
import Monaco from './monaco/Monaco.vue';

import { debounce } from './util'
import { Store } from './store';

const store = inject('store') as Store;

const onChange = debounce((code: string) => {
  store.state.activeFile.code = code
}, 250);

</script>

<template>
  <FileSelector />
  <div class="editor-container">
    <Monaco @change="onChange" :value="store.state.activeFile.code" :filename="store.state.activeFile.filename" />
  </div>
</template>

<style>
.editor-container {
  height: calc(100% - var(--header-height));
  overflow: hidden;
  position: relative;
}
</style>
