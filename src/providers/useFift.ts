import { shallowRef } from 'vue';
import init, * as wasm from '@fift';

let startedLoading = false;
let fiftModule = shallowRef<typeof wasm>();

export function useFift() {
  if (!startedLoading) {
    startedLoading = true;
    init().then(() => {
      fiftModule.value = wasm;
    });
  }

  return {
    fiftModule
  };
}
