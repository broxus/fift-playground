import { reactive } from 'vue';
import type { editor } from 'monaco-editor-core';

// import { useFift } from './providers/useFift';
import { atou, utoa } from './util';

const DEFAULT_MAIN_FILE = 'main.fif';
const WELCOME_CODE = '"Hello world!" type cr';

export class File {
  filename: string;
  code: string;
  hidden: boolean;
  editorViewState: editor.ICodeEditorViewState | null = null;

  constructor(filename: string, code = '', hidden = false) {
    this.filename = filename;
    this.code = code;
    this.hidden = hidden;
  }

  get language() {
    if (this.filename.endsWith('.fif')) {
      return 'fift';
    }
    return 'text';
  }
}

export interface StoreState {
  mainFile: string;
  includeStdlib: boolean;
  files: Record<string, File>;
  activeFile: File;
  errors: (string | Error)[];
  resetFlip: boolean;
}

export interface Store {
  state: StoreState;
  initialShowOutput: boolean;

  init: () => void;
  setActive: (filename: string) => void;
  addFile: (filename: string | File) => void;
  deleteFile: (filename: string) => void;
  renameFile: (oldFilename: string, newFilename: string) => void;
}

export class ReplStore implements Store {
  state: StoreState;
  initialShowOutput: boolean;

  constructor(serializedState = '', showOutput = false) {
    const files: StoreState['files'] = {};

    if (serializedState) {
      const saved = JSON.parse(atou(serializedState));
      for (const filename in saved) {
        setFile(files, filename, saved[filename]);
      }
    } else {
      setFile(files, DEFAULT_MAIN_FILE, WELCOME_CODE);
    }

    this.initialShowOutput = showOutput;

    let mainFile = DEFAULT_MAIN_FILE;
    if (!files[mainFile]) {
      mainFile = Object.keys(files)[0];
    }

    this.state = reactive({
      mainFile,
      includeStdlib: true,
      files,
      activeFile: files[mainFile],
      errors: [],
      resetFlip: true
    });
  }

  init(): void {
    this.state.errors = [];
  }

  setActive(filename: string): void {
    this.state.activeFile = this.state.files[filename];
  }

  addFile(filename: string | File): void {
    const file = typeof filename === 'string' ? new File(filename) : filename;
    this.state.files[file.filename] = file;
    if (!file.hidden) {
      this.setActive(file.filename);
    }
  }

  deleteFile(filename: string): void {
    if (confirm(`Are you sure to delete ${filename}?`)) {
      if (this.state.activeFile.filename === filename) {
        this.state.activeFile = this.state.files[this.state.mainFile];
      }
      delete this.state.files[filename];
    }
  }

  renameFile(oldFilename: string, newFilename: string): void {
    const { files } = this.state;
    const file = files[oldFilename];

    if (!file) {
      this.state.errors = [`Could not rename "${oldFilename}", file not found`];
      return;
    }

    if (!newFilename || oldFilename === newFilename) {
      this.state.errors = [`Cannot rename "${oldFilename}" to "${newFilename}"`];
      return;
    }

    file.filename = newFilename;

    const newFiles: Record<string, File> = {};
    for (const name in files) {
      if (name === oldFilename) {
        newFiles[newFilename] = file;
      } else {
        newFiles[name] = files[name];
      }
    }

    this.state.files = newFiles;

    if (this.state.mainFile === oldFilename) {
      this.state.mainFile = newFilename;
    }

    // TODO: compile
  }

  serialize(): string {
    const files = this.getFiles();
    return '#' + utoa(JSON.stringify(files));
  }

  getFiles(): Record<string, string> {
    const exported: Record<string, string> = {};
    for (const filename in this.state.files) {
      // TODO: normalize filename
      exported[filename] = this.state.files[filename].code;
    }
    return exported;
  }
}

function setFile(files: Record<string, File>, filename: string, content: string) {
  // TODO: normalize
  files[filename] = new File(filename, content);
}
