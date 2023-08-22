<script lang="ts" setup>
import { computed, inject, ref, VNode, Ref } from 'vue'

import { Store } from './store'

const DEFAULT_FILE_NAME = 'File.fif';

const store = inject('store') as Store

const pending = ref<boolean | string>(false)
const pendingFilename = ref(DEFAULT_FILE_NAME)

const files = computed(() =>
  Object.entries(store.state.files)
    .filter(([_name, file]) => !file.hidden)
    .map(([name]) => name)
);

function startAddFile() {
  let i = 0;
  let name = DEFAULT_FILE_NAME;

  while (true) {
    let hasConflict = false
    for (const filename in store.state.files) {
      if (filename === name) {
        hasConflict = true
        name = `File${++i}.fif`
        break
      }
    }
    if (!hasConflict) {
      break
    }
  }

  pendingFilename.value = name
  pending.value = true
}

function cancelNameFile() {
  pending.value = false
}

function focus({ el }: VNode) {
  (el as HTMLInputElement).focus();
}

function doneNameFile() {
  if (!pending.value) {
    return;
  }
  const filename = pendingFilename.value
  const oldFilename = pending.value === true ? '' : pending.value

  if (filename !== oldFilename && filename in store.state.files) {
    store.state.errors = [`File "${filename}" already exists.`]
    return
  }

  store.state.errors = []
  cancelNameFile()

  if (filename === oldFilename) {
    return
  }

  if (oldFilename) {
    store.renameFile(oldFilename, filename)
  } else {
    store.addFile(filename)
  }
}

function editFileName(file: string) {
  pendingFilename.value = file
  pending.value = file
}

const fileSel = ref(null)
function horizontalScroll(e: WheelEvent) {
  e.preventDefault()
  const el = fileSel.value! as HTMLElement
  const direction =
    Math.abs(e.deltaX) >= Math.abs(e.deltaY) ? e.deltaX : e.deltaY
  const distance = 30 * (direction > 0 ? 1 : -1)
  el.scrollTo({
    left: el.scrollLeft + distance,
  })
}
</script>

<template>
  <div class="file-selector" @wheel="horizontalScroll" ref="fileSel">
    <div class="stdlib-toggle" @click="store.state.includeStdlib = !store.state.includeStdlib">
      <span>Stdlib</span>
      <div class="toggle" :class="[{ active: store.state.includeStdlib }]">
        <div class="indicator" />
      </div>
    </div>

    <template v-for="(file, i) in files">
      <div v-if="pending !== file" class="file" :class="{ active: store.state.activeFile.filename === file }"
        @click="store.setActive(file)" @dblclick="i > 0 && editFileName(file)">
        <span class="label">{{ file }}</span>
        <span v-if="i > 0" class="remove" @click.stop="store.deleteFile(file)">
          <svg class="icon" width="12" height="12" viewBox="0 0 24 24">
            <line stroke="#999" x1="18" y1="6" x2="6" y2="18"></line>
            <line stroke="#999" x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </span>
      </div>
      <div v-if="(pending === true && i === files.length - 1) || pending === file" class="file pending">
        <input v-model="pendingFilename" spellcheck="false" @blur="doneNameFile" @keyup.enter="doneNameFile"
          @keyup.esc="cancelNameFile" @vue:mounted="focus" />
      </div>
    </template>
    <button class="add" @click="startAddFile">+</button>
  </div>
</template>

<style scoped>
.file-selector {
  display: flex;
  box-sizing: border-box;
  border-bottom: 1px solid var(--border);
  background-color: var(--bg);
  overflow-y: hidden;
  overflow-x: auto;
  white-space: nowrap;
  position: relative;
  height: var(--header-height);
}

.file-selector::-webkit-scrollbar {
  height: 1px;
}

.file-selector::-webkit-scrollbar-track {
  background-color: var(--border);
}

.file-selector::-webkit-scrollbar-thumb {
  background-color: var(--color-branding);
}

.file {
  display: inline-block;
  font-size: 13px;
  font-family: var(--font-code);
  cursor: pointer;
  color: var(--text-light);
  box-sizing: border-box;
}

.file.active {
  color: var(--color-branding);
  border-bottom: 3px solid var(--color-branding);
  cursor: text;
}

.file span {
  display: inline-block;
  padding: 8px 10px 6px;
  line-height: 20px;
}

.file.pending input {
  width: 90px;
  height: 30px;
  line-height: 30px;
  outline: none;
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0 0 0 10px;
  margin-top: 2px;
  margin-left: 6px;
  font-family: var(--font-code);
  font-size: 12px;
}

.file .remove {
  display: inline-block;
  vertical-align: middle;
  line-height: 12px;
  cursor: pointer;
  padding-left: 0;
}

.add {
  font-size: 18px;
  color: #999;
  vertical-align: middle;
  margin-left: 6px;
  position: relative;
  top: -1px;
}

.add:hover {
  color: var(--color-branding);
}

.icon {
  margin-top: -1px;
}

.stdlib-toggle {
  display: flex;
  align-items: center;
  background-color: var(--bg);
  color: var(--text-light);
  font-family: var(--font-code);
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 2px;
  user-select: none;
}

.stdlib-toggle .toggle {
  display: inline-block;
  margin-left: 4px;
  width: 32px;
  height: 18px;
  border-radius: 12px;
  position: relative;
  background-color: var(--border);
}

.stdlib-toggle .indicator {
  font-size: 12px;
  background-color: var(--text-light);
  width: 14px;
  height: 14px;
  border-radius: 50%;
  transition: transform ease-in-out 0.2s;
  position: absolute;
  left: 2px;
  top: 2px;
  color: var(--bg);
  text-align: center;
}

.stdlib-toggle .active .indicator {
  background-color: var(--color-branding);
  transform: translateX(14px);
  color: white;
}
</style>
