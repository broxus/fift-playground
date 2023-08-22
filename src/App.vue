<script setup lang="ts">
import { provide, watchEffect } from 'vue'

import { ReplStore } from './store';

import SplitPane from './SplitPane.vue';
import EditorContainer from './EditorContainer.vue';
import Output from './Output.vue';

const query = new URLSearchParams(location.search);

const store = new ReplStore({
  serializedState: location.hash.slice(1),
  showOutput: query.has('so'),
});
store.init();

provide('store', store);

watchEffect(() => history.replaceState({}, '', store.serialize()));
</script>

<template>
  <main>
    <SplitPane>
      <template #left>
        <EditorContainer />
      </template>
      <template #right>
        <Output />
      </template>
    </SplitPane>
  </main>
</template>

<style>
main {
  height: 100%;
  margin: 0;
  overflow: hidden;
  font-size: 13px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
    Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  background-color: var(--bg-soft);
}

button {
  border: none;
  outline: none;
  cursor: pointer;
  margin: 0;
  background-color: transparent;
}
</style>
