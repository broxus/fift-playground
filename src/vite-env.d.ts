/// <reference types="vite/client" />

declare module '*.vue' {
  import { ComponentOptions } from 'vue';
  const comp: ComponentOptions;
  export default comp;
}

interface ImportMetaEnv {
  readonly BYTIE_CURRENCY: string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
