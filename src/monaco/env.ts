import * as onigasm from 'onigasm';
import onigasmWasm from 'onigasm/lib/onigasm.wasm?url';
import { editor, languages, Uri } from 'monaco-editor-core';
import editorWorker from 'monaco-editor-core/esm/vs/editor/editor.worker?worker';
import { watchEffect } from 'vue';

import { Store } from '../store';
import { getOrCreateModel } from './utils';

const URI_PREFIX_LEN = 'file:///'.length;

let initted = false;
let wasmPromise = Promise.resolve();
export async function initMonaco(store: Store) {
  if (initted) {
    return wasmPromise;
  }
  loadMonacoEnv();
  wasmPromise = loadWasm();

  watchEffect(() => {
    for (const filename in store.state.files) {
      const uri = Uri.parse(`file:///${filename}`);
      const file = store.state.files[filename];
      if (editor.getModel(uri)) {
        continue;
      }
      getOrCreateModel(uri, file.language, file.code);
    }

    for (const model of editor.getModels()) {
      const uri = model.uri.toString();
      if (store.state.files[uri.substring(URI_PREFIX_LEN)]) continue;
      if (uri.startsWith('inmemory://')) continue;

      model.dispose();
    }
  });

  initted = true;
  return wasmPromise;
}

export function loadWasm() {
  return onigasm.loadWASM(onigasmWasm);
}

export function loadMonacoEnv() {
  (self as any).MonacoEnvironment = {
    async getWorker(_: any, _label: string) {
      return new editorWorker();
    }
  };

  languages.register({ id: 'fift', extensions: ['.fif'] });
}
