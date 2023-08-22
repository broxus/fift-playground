<script lang="ts" setup>
import { inject, ref, watchEffect } from 'vue';
import * as monaco from 'monaco-editor-core';
import * as fift from '@fift'

import { useFift } from './providers/useFift'
import { Store } from './store';

const OWNER = "Fift";

type Tab = 'stdout' | 'stderr' | 'backtrace';
const OK_TABS: Tab[] = ['stdout'];
const ERR_TABS_SHORT: Tab[] = ['stdout', 'stderr'];
const ERR_TABS_FULL: Tab[] = ['stdout', 'stderr', 'backtrace'];

type OutputState = {
  stdout: string;
  stderr: string;
  backtrace: string[];
  exitCode?: number;
};

function makeDefaultState(): OutputState {
  return { stdout: '', stderr: '', backtrace: [] };
}

const store = inject('store') as Store

const { fiftModule } = useFift();
const state = ref<OutputState>(makeDefaultState())

const tabs = ref<Tab[]>();
const selectedTab = ref<Tab>();

watchEffect(() => {
  const fift = fiftModule.value;
  if (fift == null) {
    return;
  }

  const activeFile = store.state.activeFile;
  const uri = monaco.Uri.parse(`file:///${activeFile.filename}`);
  const model = monaco.editor.getModel(uri);

  state.value = makeDefaultState();

  let fiftState: fift.FiftState;
  try {
    fiftState = new fift.FiftState();
    const res = fiftState.run(activeFile.code, store.state.includeStdlib);

    state.value.stdout = res.stdout;
    if (res.success == true) {
      state.value.exitCode = res.exitCode;
      tabs.value = OK_TABS;
      selectedTab.value = 'stdout';

      if (model != null) {
        monaco.editor.setModelMarkers(model, OWNER, []);
      }
    } else {
      state.value.stderr = res.stderr;
      tabs.value = res.backtrace != null ? ERR_TABS_FULL : ERR_TABS_SHORT;
      selectedTab.value = 'stderr';

      if (res.errorPosition != null && model != null) {
        monaco.editor.setModelMarkers(model, OWNER, [{
          severity: monaco.MarkerSeverity.Error,
          message: res.stderr,
          source: res.errorPosition.blockName,
          startLineNumber: res.errorPosition.lineNumber,
          startColumn: res.errorPosition.wordStart + 1,
          endLineNumber: res.errorPosition.lineNumber,
          endColumn: res.errorPosition.wordEnd + 1,
        }])
      }

      if (res.backtrace) {
        state.value.backtrace = res.backtrace;
      }
    }
  } catch (e: any) {
    state.value.stderr = e.toString();
  } finally {
    fiftState?.free();
  }
});
</script>

<template>
  <div class="tab-buttons">
    <button v-for="t of tabs" :class="{ active: selectedTab === t }" @click="selectedTab = t">
      <span>{{ t }}</span>
    </button>

    <span v-if="state.exitCode != null" class="exit-code">code: {{ state.exitCode }}</span>
  </div>

  <div class="output-container">
    <pre class="stdout" v-show="selectedTab === 'stdout'">{{ state.stdout }}</pre>
    <pre class="stderr" v-show="selectedTab === 'stderr'">{{ state.stderr }}</pre>
    <ol class="backtrace" v-show="selectedTab === 'backtrace'">
      <li v-for="item of state.backtrace">{{ item }}</li>
    </ol>
  </div>
</template>

<style scoped>
.output-container {
  height: calc(100% - var(--header-height));
  overflow: hidden;
  position: relative;
}

.tab-buttons {
  display: flex;
  flex-direction: row;
  box-sizing: border-box;
  border-bottom: 1px solid var(--border);
  background-color: var(--bg);
  height: var(--header-height);
  overflow: hidden;
}

.tab-buttons button {
  padding: 0;
  box-sizing: border-box;
  display: flex;
  flex-direction: row;
}

.tab-buttons span {
  font-size: 13px;
  font-family: var(--font-code);
  text-transform: uppercase;
  color: var(--text-light);
  display: inline-block;
  padding: 8px 16px 6px;
  line-height: 20px;
}

.tab-buttons span.exit-code {
  margin-left: auto;
  color: var(--color-branding);
}

button.active {
  color: var(--color-branding-dark);
  border-bottom: 3px solid var(--color-branding-dark);
}

.stdout,
.stderr {
  margin: 1em;
  display: flex;
  flex-direction: column;
  position: relative;
  max-height: 100%;
}

.backtrace {
  margin: 1em;
  padding: 0;
  list-style-position: inside;
}

.backtrace li {
  padding: 1em;
  background-color: var(--bg);
  margin-bottom: 2px;
  font-family: var(--font-code);
}

pre {
  flex: 0 0 auto;

  padding: 1em;
  color: var(--text-light);
  background-color: var(--bg);

  overflow: scroll;
}
</style>
