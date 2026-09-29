// Every user-facing string lives here so a Ukrainian translation only touches this file.
const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`;

export const t = {
  name: 'openlofree',
  tabs: { keys: 'Keys', lighting: 'Lighting', profiles: 'Profiles', device: 'Device' },
  chip: { none: 'No keyboard', demo: 'Demo keyboard', searching: 'Looking for a keyboard' },
  theme: {
    label: 'Theme',
    system: 'Match system',
    light: 'Light',
    dark: 'Dark',
  },

  state: {
    notFound: {
      title: 'No keyboard found',
      body: 'Connect the Flow 2 with a USB-C cable and set its switch to wired mode. Bluetooth cannot be configured.',
      hint: 'Looking again every few seconds.',
    },
    timeout: {
      title: 'The keyboard is not answering',
      body: 'Press a key to wake it. If it stays silent, check the wired-mode switch and close other tools that use it, such as VIA in a browser.',
      hint: 'Looking again every few seconds.',
    },
    generic: { title: 'Something went wrong' },
    loading: {
      title: 'Reading the keyboard',
      body: (layers: number) => `Reading all ${plural(layers, 'layer', 'layers')} and the backlight.`,
    },
    disconnected: 'The keyboard was disconnected.',
    retry: 'Try again',
    demo: 'Try demo',
    demoNote: 'Demo needs no keyboard and never writes to a real one.',
  },

  keys: {
    layer: (n: number) => `Layer ${n}`,
    layers: 'Layers',
    selectPrompt: 'Select a key on the keyboard to change what it types.',
    selected: (label: string, hex: string) => `${label} (${hex})`,
    now: 'Now',
    changedMark: 'changed',
    keyLabel: (row: number, col: number, legend: string, changed: boolean) =>
      `Row ${row + 1}, column ${col + 1}, ${legend}${changed ? ', changed' : ''}`,
  },

  picker: {
    title: 'Assign a key',
    search: 'Search keys',
    noMatch: (q: string) => `Nothing matches "${q}".`,
    customCode: 'Custom code',
    customHint: 'Hex keycode, for example 0x0029',
    set: 'Set',
    invalid: 'Enter a hex value from 0x0000 to 0xFFFF.',
  },

  bar: {
    summary: (keys: number, layers: number) =>
      `${plural(keys, 'key', 'keys')} changed on ${plural(layers, 'layer', 'layers')}`,
    none: 'No unsaved changes',
    apply: 'Apply to keyboard',
    discard: 'Discard',
    writing: 'Writing to the keyboard',
    done: (n: number) => `Wrote ${plural(n, 'key', 'keys')}. Each one was read back and matched.`,
    failed: 'Not applied',
  },

  lighting: {
    title: 'Backlight',
    mode: 'Mode',
    modes: { off: 'Off', steady: 'Steady', breathing: 'Breathing' },
    brightness: 'Brightness',
    note: 'Preview changes the light now. Apply saves it to the keyboard, so it stays after a power cycle.',
    apply: 'Apply',
    reset: 'Reset',
    applied: 'Saved to the keyboard.',
    previewFailed: 'Preview failed',
    whiteOnly: 'The Flow 2 backlight is white only, there are no colors to pick.',
  },

  profiles: {
    title: 'Profiles',
    saveAs: 'Save current as…',
    import: 'Import…',
    empty: {
      title: 'No profiles yet',
      body: 'Save the keymap and lighting you see now as a profile. Then switch setups here or from the tray.',
    },
    loading: 'Loading profiles',
    unapplied: (keys: number) => `Includes ${plural(keys, 'unapplied change', 'unapplied changes')}.`,
    meta: (model: string, layers: number) => `${model}, ${plural(layers, 'layer', 'layers')}`,
    apply: 'Apply',
    duplicate: 'Duplicate',
    export: 'Export',
    delete: 'Delete',
    needKeyboard: 'Connect a keyboard to apply profiles.',
    otherModel: 'Made for another model.',
    applied: (name: string, n: number) => `Applied ${name}: ${plural(n, 'key', 'keys')} changed.`,
    saved: (name: string) => `Saved ${name}.`,
    exported: (name: string) => `Exported ${name}.`,
    imported: 'Imported.',
    deleted: (name: string) => `Deleted ${name}.`,
    nameLabel: 'Profile name',
    saveTitle: 'Save profile',
    duplicateTitle: (name: string) => `Duplicate ${name}`,
    confirmDelete: (name: string) => `Delete ${name}? This cannot be undone.`,
    overwrite: (name: string) => `A profile named ${name} exists. Saving replaces it.`,
    cancel: 'Cancel',
    save: 'Save',
    confirm: 'Delete',
  },

  device: {
    title: 'Device',
    model: 'Model',
    connection: 'Connection',
    usb: 'USB, wired',
    demo: 'Demo, no hardware',
    product: 'Product name',
    ids: 'USB ID',
    protocol: 'VIA protocol',
    layers: 'Layers',
    battery: 'Battery',
    batteryUnknown: 'Not available',
    batteryWhy: 'Read over Bluetooth on macOS. Windows and Linux are not supported yet.',
    backup: 'Factory backup',
    backupMissing: 'Created before the first write.',
    backupDemo: 'Not used in demo.',
    unverified: 'Not tested on real hardware yet.',
    refresh: 'Read again',
    disconnect: 'Disconnect',
    leaveDemo: 'Leave demo',
    offline: 'No keyboard connected.',
  },

  errors: {
    verify: 'The keyboard did not keep the change. Nothing else was written.',
  },
};
