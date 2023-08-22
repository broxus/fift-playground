<script lang="ts" setup>
import { watch, computed, inject, onMounted, onBeforeUnmount, ref, shallowRef } from 'vue';
import * as monaco from 'monaco-editor-core'
import { Registry } from 'monaco-textmate';
import { wireTmGrammars } from 'monaco-editor-textmate';

import { Store } from '../store';
import { initMonaco } from './env';
import { getOrCreateModel } from './utils';

import fiftGrammar from './fift.tmLanguage.json';

const registry = new Registry({
  getGrammarDefinition: async _ => {
    return {
      format: 'json',
      content: fiftGrammar
    };
  }
});

const grammars = new Map();
grammars.set('fift', 'source.fif');

const props = withDefaults(
  defineProps<{
    filename: string
    value?: string
    readonly?: boolean
  }>(),
  { readonly: false }
);

const emit = defineEmits<{
  (e: 'change', value: string): void
}>();

const containerRef = ref<HTMLDivElement>();
const editor = shallowRef<monaco.editor.IStandaloneCodeEditor>();
const store = inject<Store>('store')!;

const initPromise = initMonaco(store);

const lang = computed(() => 'fift');

onMounted(async () => {
  if (!containerRef.value) {
    throw new Error('Cannot find containerRef');
  }

  const editorInstance = monaco.editor.create(containerRef.value, {
    ...(props.readonly
      ? { value: props.value, language: lang.value, wordWrap: 'on' }
      : { model: null }),
    fontSize: 13,
    readOnly: props.readonly,
    automaticLayout: true,
    scrollBeyondLastLine: false,
    minimap: {
      enabled: false,
    },
    inlineSuggest: {
      enabled: false,
    },
    fixedOverflowWidgets: true,
    theme: 'vs-dark',
  })
  editor.value = editorInstance;

  watch(
    () => props.value,
    (value) => {
      if (editorInstance.getValue() === value) {
        return;
      }
      editorInstance.setValue(value || '');
    },
    { immediate: true },
  );

  watch(lang, (lang) =>
    monaco.editor.setModelLanguage(editorInstance.getModel()!, lang)
  );

  if (!props.readonly) {
    watch(
      () => props.filename,
      (_, oldFilename) => {
        if (!editorInstance) return
        const file = store.state.files[props.filename]
        if (!file) return null
        const model = getOrCreateModel(
          monaco.Uri.parse(`file:///${props.filename}`),
          file.language,
          file.code
        )

        const oldFile = oldFilename ? store.state.files[oldFilename] : null
        if (oldFile) {
          oldFile.editorViewState = editorInstance.saveViewState()
        }

        editorInstance.setModel(model)

        if (file.editorViewState) {
          editorInstance.restoreViewState(file.editorViewState)
          editorInstance.focus()
        }
      },
      { immediate: true }
    )
  }

  initPromise.then(() => wireTmGrammars(monaco as any, registry, grammars, editorInstance as any));

  editorInstance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
    // ignore save event
  });

  editorInstance.onDidChangeModelContent(() => {
    emit('change', editorInstance.getValue())
  });
});

onBeforeUnmount(() => {
  editor.value?.dispose()
});
</script>

<template>
  <div class="editor" ref="containerRef"></div>
</template>

<style>
.editor {
  position: relative;
  height: 100%;
  width: 100%;
  overflow: hidden;
}
</style>
