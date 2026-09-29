# Lofree Flow 2 на macOS: чи можна писати власний софт і що вже існує

Дата дослідження: 2026-09-29. Усі GitHub-дані (зірки, дати комітів) зняті через api.github.com / raw.githubusercontent.com у день дослідження.
Позначки впевненості: **[В]** висока, **[С]** середня, **[Н]** низька. **[НЕ ПЕРЕВІРЕНО]** означає, що я не зміг підтвердити це першоджерелом.

## TL;DR

- **Так, це можливо, і базова версія вже існує.** Flow 2 працює на QMK-прошивці з підтримкою VIA (raw HID, usage page `0xFF60`, usage `0x61`, звіти по 32 байти). Окремого «драйвера» немає, а офіційний конфігуратор Lofree це VIA JSON плюс Windows-утиліта. [В]
- **Найближче до вашої задачі:** `linder3hs/lofree-flow-2` (Swift, MIT, macOS 13+): menu bar app і CLI. Показує батарею (через `system_profiler`, працює по Bluetooth) і керує підсвіткою (VIA raw HID, лише по USB). Перевірено автором на моделі 84, v0.1.0 від 2026-08-18. [С]
- **Переназначення клавіш, шари, макроси** робляться штатно через VIA у Chromium (WebHID) з ручним завантаженням JSON-визначення, бо в офіційній базі VIA цієї клавіатури немає. Safari і Firefox WebHID не підтримують. [В]
- **RGB немає:** підсвітка біла. Це видно зі специфікації Lofree («Backlight: White»). [В]
- **Bluetooth:** vendor-канал VIA по BT не доступний (за двома незалежними community-джерелами). Причину не встановлено. Для BT лишаються батарея (стандартний BLE-сервіс) і ремап на боці macOS. [С]
- **Фолбек без vendor-протоколу:** `hidutil` (Apple TN2450) і Karabiner-Elements (фільтр за `vendor_id`, `product_id`, `device_address`) працюють в усіх трьох режимах, зокрема по BT. [В]
- **Офіційного SDK, коду прошивки і документації немає.** Lofree письмово відмовила у QMK-коді, розпіновці матриці і VIAL-збірці (qmk/qmk_firmware#26062). Повний власний QMK-порт нереалістичний: є лише частковий WIP (`sameid/samkey84`) без BT, 2.4G, touch bar і батареї. [В]
- **Не плутати:** Flow (оригінал) і Flow Lite мають інші моделі та протоколи. Flow Lite не VIA, а власний vendor-протокол, і для нього вже є окремий WebHID-конфігуратор.

---

## 1. Що таке Flow 2

### 1.1 Назви та лінійка [В]

Lofree продає окремі продукти: **Flow** (оригінал), **Flow Lite** і **Flow 2**. Це видно з навігації сайту.
- Download Center (розділи Flow, Flow Lite, Flow 2 for Mac): https://www.lofree.co/pages/download-center
- Flow 2 68: https://www.lofree.co/products/flow-2-68-low-profile-mechanical-keyboard
- Flow 2 84 for Mac: https://www.lofree.co/products/flow-2-84-low-profile-mechanical-keyboard-for-mac
- Flow 2 100 for Mac: https://www.lofree.co/products/flow-2-100-low-profile-mechanical-keyboard-for-mac

Три розміри: 68 (65%), 84 (75%), 100 (96%). Сторінки 84 і 100 мають суфікс «for Mac» (макет Mac).

### 1.2 Специфікація (з офіційних сторінок) [В]

| Параметр | Значення | Джерело |
|---|---|---|
| Підключення | USB-C дріт, Bluetooth 5.3, 2.4 GHz донгл | сторінки продуктів вище |
| Polling (дріт / 2.4G) | до 1000 Hz | сторінки продуктів |
| Hot-swap | Так («Hot-swappable Support: Yes») | сторінки продуктів |
| Перемикачі | Lofree x Kailh Cloud Series (low-profile) | сторінки продуктів |
| Підсвітка | Біла («Backlight: White»), не RGB | сторінки продуктів |
| Сенсорна панель | Емкісна touch bar збоку (гучність, яскравість) | сторінки продуктів |
| Батарея | 68: 2000 mAh; 84 і 100: 3000 mAh | сторінки продуктів |
| BT-назва (68) | `Flow2-68@lofree` | сторінка Flow 2 68 |
| Заявлена кастомізація | «Fully compatible with VIA», «Reassign keys, set up advanced macros» | сторінки продуктів |

Уточнення: перемикач кількості клавіш «68 / 84 / 100» на сторінках частково спільний, тому цифри батареї беру з блоку Tech specs кожної сторінки, а не з маркетингового тексту. На сторінці 68-key є текст «Flow Lite's seamless Bluetooth connectivity», ймовірно копіпаст з іншого продукту. Це не впливає на висновки.

### 1.3 Прошивка та залізо [С]

- **Прошивка: QMK з VIA.** Офіційно заявлено «full VIA support». Community-дослідження показують, що VIA-протокол відповідає (версія 12 у README `linder3hs`). Джерела: https://github.com/linder3hs/lofree-flow-2 і https://github.com/andresmarpz/flow/blob/main/flow.md.
- **QMK-код не опублікований.** Лист підтримки Lofree (цитується в issue): «unable to provide the QMK source code, internal keymap source files, or hardware-level documentation such as matrix wiring, MCU pin mappings, or bootloader implementation details». Також «We also do not have an official VIAL-enabled firmware build». Джерело: https://github.com/qmk/qmk_firmware/issues/26062 [В щодо змісту листа; це переказ автора issue, оригінали були скріншотами].
- **MCU: Westberry WB32FQ95**, bootloader `wb32-dfu`. Джерело: `keyboard.json` у https://github.com/sameid/samkey84 (модель 84) [С]. Незалежне підтвердження для моделі 68: у нотатках прошивки 256 KB flash і 36 KB SRAM, «matches Westberry's WB32FQ95xx family closely» (https://github.com/seobaeksol/lofree-flow-spacefn-layout, `docs/firmware/flow2_firmware_notes.md`) [С].
- **Режим DFU:** `VID_342D PID_DFA0`, «WB Device in DFU Mode» (https://github.com/dsamjjlenk/lofree-flow2-100-backlight-60s). Прошивка через `wb32-dfu-updater_cli`, базова адреса `0x08000000`.
- **Кодові назви моделей (лише з community):** 100-key = `OE926`, 68-key = `OE928` (https://github.com/dsamjjlenk/lofree-flow2-100-backlight-60s, https://github.com/seobaeksol/lofree-flow-spacefn-layout). Для 84-key код не знайшов. [Н]

### 1.4 VID/PID [С, суперечливо]

| Значення | Що це | Джерело |
|---|---|---|
| `0x388D` | Vendor ID у всіх джерелах | усі нижче |
| `0x388D:0x0003` | Flow 2 100 (OE926), дротовий режим | dsamjjlenk, `docs/TECHNICAL_NOTES.md`, `tools/check_keyboard.py` |
| `0x388D:0x0001` | VIA-визначення Flow 2 68 (OE928) (`"name":"Flow2@Lofree"`) і водночас 2.4G донгл | seobaeksol `references/oe928_via_definition_20251114.json`; piperamirez/lofree-flow2-os9-driver README |
| `0x342D:0xDFA0` | Bootloader (DFU) | dsamjjlenk |

Висновок: PID залежить від моделі, тож на 84-key його треба зчитати з власного пристрою (`ioreg -p IOUSB -l` або `system_profiler SPUSBDataType`). `linder3hs` обходить це фільтром за usage page `0xFF60/0x61` і за підрядком «flow» у product string. Product string за VIA-визначенням: `Flow2@Lofree`. Для 84-key VID/PID у жодному джерелі, яке я відкрив, не наведено. [НЕ ПЕРЕВІРЕНО]

---

## 2. Офіційний софт

### 2.1 Що є [С]

- Lofree **не поширює нативну програму для macOS** для Flow 2. Автор `linder3hs` пише: «Lofree ships no desktop software» (https://github.com/linder3hs/lofree-flow-2). Автор `andresmarpz/flow`: «no proprietary software, all configuration is done through VIA» (https://github.com/andresmarpz/flow/blob/main/flow.md).
- **Що дає Lofree:** пакет з Dropbox (посилання на сторінках продуктів, розділ Downloads) з такими підписами:
  - на сторінках 84 і 100 «Lofree Flow 2 **Mac** VIA Configurator **For Windows**»;
  - на сторінці 68 «Lofree Flow 2 VIA Configurator For Windows».
  
  За описом в issue #26062, пакет містить скомпільований `.hex`, VIA JSON і утиліту «Lofree Flow 2 VIA Configurator» (https://github.com/qmk/qmk_firmware/issues/26062).
- **Вміст Dropbox-папок я не зміг переглянути** (сторінка потребує JavaScript, WebFetch не бачить списку). [НЕ ПЕРЕВІРЕНО] Тому не знаю, яка версія конфігуратора, чи є там macOS-збірка і чи це лише оновлювач прошивки.
- Практичний шлях для макОС: **usevia.app у Chrome/Edge**, вкладка Design (Settings, «Show Design tab»), завантажити JSON, після цього з'являється «Flow2@Lofree». Джерело: https://github.com/andresmarpz/flow/blob/main/flow.md [С, це переказ одного автора, я цього не відтворював].
- У офіційній базі VIA (`the-via/keyboards`, гілка `master`) немає жодного шляху зі словом «lofree». Перевірка по 6915 записах дерева (не обрізане): https://github.com/the-via/keyboards. Тобто JSON-визначення доводиться завантажувати вручну. [С: шукав лише за назвою шляху, не за вмістом]

### 2.2 Відомі обмеження і скарги [С]

- **68-key JSON має неправильну назву у пакеті Lofree.** Треба брати файл `FLOW2-100key-OE928-Json-20250905.json` (https://github.com/andresmarpz/flow/blob/main/flow.md). [С, один автор]
- **VIA не зберігає клавіші перемикання BT/2.4G.** Вони кодуються як сирі кейкоди `0x7793/0x7794/0x7795` (BT1-3) і `0x7785` (2.4G), а після `Save + Load` у VIA часто повертаються як `KC_NO` (https://github.com/seobaeksol/lofree-flow-spacefn-layout, `docs/guides/via_layouts.md`). [С]
- **Оновлення прошивки могло «зацегелити» пристрій.** Автор `sameid/samkey84` стверджує, що офіційне оновлення прошивки перетворило його 84-key на цеглу без можливості увійти в DFU, після чого Lofree замінила пристрій (https://github.com/sameid/samkey84). Це одне свідчення. [Н]
- **VIA-визначення для 68 не містить кастомних меню.** У завантаженому JSON немає ключа `menus`, лише `customKeycodes: [TRIGGER]`, матриця 5x15 (`references/oe928_via_definition_20251114.json`, репо seobaeksol). Отже, у вікні VIA керування підсвіткою може бути відсутнє: у `linder3hs` підсвітку знайшли зондуванням VIA-каналів, а не з JSON. [С]
- Про Flow Lite: `aaditagrawal/flow-control` зазначає, що Flow Lite 84 «does not ship with native VIA support» (https://github.com/aaditagrawal/flow-control). Це підтверджує, що лінійки різні.
- Обзори (Tom's Guide, Notebookcheck) знайшлися пошуком, але сторінки не відкрилися (обрізаний текст, HTTP 403), тому «huge drawback» з Tom's Guide я не читав. [НЕ ПЕРЕВІРЕНО] https://www.tomsguide.com/computing/keyboards/lofree-flow-2-review

---

## 3. Community / третя сторона

Усі дані з GitHub API на 2026-09-29. Зірки й дати «pushed» відносні: репозиторії дуже молоді та малі.

### 3.1 Flow 2 (релевантні)

| Репо | Що це | Стан | Джерело |
|---|---|---|---|
| **linder3hs/lofree-flow-2** | macOS menu bar app і CLI `flow2ctl` (Swift, IOKit, SwiftUI, MIT). Батарея, підсвітка (off/steady/breathing і яскравість), детекція USB/BT. Без ремапу і макросів (у design-spec явно «fuera de alcance»). | 1 зірка, створений 2026-08-07, останній push 2026-08-18, реліз v0.1.0 (arm64 zip). Тестовано автором на 84-key. Ad-hoc підпис, не нотаризовано. | https://github.com/linder3hs/lofree-flow-2 |
| **andresmarpz/flow** | Один файл `flow.md`: дослідження VIA-протоколу для Flow 2 68, план TUI. Коду немає. | 2 зірки, 2026-02-07..09 (3 коміти) | https://github.com/andresmarpz/flow |
| **sameid/samkey84** | Власна QMK-прошивка для Flow 2 84 (WB32FQ95). Працює: усі клавіші, Fn-шар, медіа, підсвітка лише on/off, Caps-LED. WIP: BLE, 2.4G, touch bar, індикатор батареї. Ліцензії немає. | 4 зірки, створений 2025-12-09, останній коміт 2026-02-08 («release binary») | https://github.com/sameid/samkey84 |
| **dsamjjlenk/lofree-flow2-100-backlight-60s** | Патч офіційної прошивки OE926 v14 (4 байти за `0x0800403E`): таймаут підсвітки у бездротовому режимі 10 с → 60 с. Скрипти Windows/Python: `enter_dfu_via_raw_hid.py` (VIA-команда `0x0B` bootloader jump), `flash_60s_wb32.ps1`. MIT. **Лише для Flow 2 100.** | 6 зірок, 2026-06-04 | https://github.com/dsamjjlenk/lofree-flow2-100-backlight-60s |
| **seobaeksol/lofree-flow-spacefn-layout** | Патч дефолтної розкладки в офіційній прошивці 68-key (OE928 v14), інструмент `oe928_firmware_tool.py` (dump/extract/patch), 6 шарів (3 Mac + 3 Win). Нотатки з реверсу прошивки. | 0 зірок, останній коміт 2026-04-21 | https://github.com/seobaeksol/lofree-flow-spacefn-layout |
| **kabolen/via-lofree-flow2** | Форк `the-via/app` «with changes for Flow 2». У п'яти останніх комітах і в гілках (лише `main`) бачу тільки апстрімні коміти (останній 2025-05-12); зірок 0. Тобто помітних Flow 2-змін я не знайшов. | створений 2025-12-24 | https://github.com/kabolen/via-lofree-flow2 |
| piperamirez/lofree-flow2-os9-driver | USB HID-драйвер для **2.4G донгла** під Mac OS 9 (VID/PID `0x388D/0x0001`, композитний пристрій з інтерфейсами 0, 1, 2). Корисно як опис донгла. | 0 зірок, 2026-04-20 | https://github.com/piperamirez/lofree-flow2-os9-driver |

Уточнення щодо `kabolen`: `linder3hs/docs/superpowers/plans/...` посилається на цей репо як на джерело JSON-визначення, але сам я в ньому Flow 2-специфічних файлів не побачив (див. вище). [С]

### 3.2 Не Flow 2, але схожі (не переносити напряму)

| Репо | Для чого | Джерело |
|---|---|---|
| aaditagrawal/flow-control | **Flow Lite 84** (`Air84@Lofree`, VID:PID `05AC:024F`, vendor collection `0xFF02:0x0002`, report ID `0x08`, 16 байт). Нативний macOS 14+ додаток (IOHID) і WebHID-компаньйон. Ключі, підсвітка, макроси. Явно «one observed variant». | https://github.com/aaditagrawal/flow-control |
| shikuiow/lofree-key-mapper-mac | Офлайн-обгортка (Electron/Node) офіційної веб-сторінки Lofree для **Flow Lite**; WebHID. Права на вбудовані ресурси Lofree не надано. | https://github.com/shikuiow/lofree-key-mapper-mac |
| DigitalEdens/lofree-mac-2-4ghz-dongle-battery | Батарея через 2.4G-донгл (HID) або BLE Battery Service; протестовано на **Flow Lite 100**. Потрібен Input Monitoring. | https://github.com/DigitalEdens/lofree-mac-2-4ghz-dongle-battery |
| dmtrKovalenko/lofree-hypace-reverse-engineer | Реверс протоколу прошивання миші **Hypace** (веб-флешер Lofree не підтримував частину функцій на macOS). Це показує, що Lofree теж використовує WebHID у своїх веб-інструментах. | https://github.com/dmtrKovalenko/lofree-hypace-reverse-engineer |
| alexeygumirov/lofree-flow-fn-fix | Linux, `hid_apple fnmode`, для оригінального **Flow**. Переїхав на Codeberg. | https://github.com/alexeygumirov/lofree-flow-fn-fix |

### 3.3 Що не знайшов

- Профілів Karabiner-Elements для Flow 2: пошук `lofree+karabiner` впирався в ліміт GitHub API, тож **результат порожній через збій, а не через відсутність**. [НЕ ПЕРЕВІРЕНО]
- Reddit / форумів з посиланнями на репо: прямого пошуку по Reddit не робив. README `DigitalEdens` згадує Reddit-гілку, але сторінку я не відкривав. [НЕ ПЕРЕВІРЕНО]
- Запису про Flow 2 у `qmk_firmware/keyboards` немає (у каталозі `keyboards` немає записів зі словом «lofree»). [В]
- Issue qmk/qmk_firmware#26062 («Lofree declined to provide QMK source...») відкрито 2026-03-13, закрито 2026-03-16, коментарів немає (дані GitHub API). Причину закриття не встановлено. https://github.com/qmk/qmk_firmware/issues/26062

---

## 4. Технічна здійсненність власного софту на macOS

### 4.1 Транспорти

| Транспорт | Vendor HID (VIA raw) | Джерела і впевненість |
|---|---|---|
| **USB-C дріт** | **Доступний.** Interface з usage page `0xFF60`, usage `0x61`, 32-байтові звіти. `linder3hs` працює через `IOHIDManager` (`IOHIDDeviceSetReport` + input report callback). | https://github.com/linder3hs/lofree-flow-2 (`Sources/Flow2Kit/HIDController.swift`, `ViaReport.swift`) [С: це лише код автора, я не запускав на залізі] |
| **2.4G донгл** | Community-твердження: VIA працює по донглу. Донгл композитний (HID-інтерфейси 0, 1, 2). | https://github.com/andresmarpz/flow/blob/main/flow.md; https://github.com/piperamirez/lofree-flow2-os9-driver [Н, з другої руки, мною не перевірено] |
| **Bluetooth** | **Недоступний на практиці.** README `linder3hs`: «VIA is only reachable over USB — over Bluetooth the app shows battery only». `andresmarpz`: «VIA/HID does NOT work over Bluetooth». Також при перемикачі в BT-режимі USB-кабель лише заряджає, HID-інтерфейс не експонується. | https://github.com/linder3hs/lofree-flow-2; https://github.com/andresmarpz/flow/blob/main/flow.md [С: два джерела, причину не встановлено] |

### 4.2 Доступ у macOS: Input Monitoring і WebHID

- Для raw HID через `IOHIDManager` macOS просить **Input Monitoring**: «macOS will ask for Input Monitoring permission ... required for raw HID access». Джерело: README https://github.com/linder3hs/lofree-flow-2. Те саме для `DigitalEdens` (донгл трактується як «protected keyboard-like HID device»): https://github.com/DigitalEdens/lofree-mac-2-4ghz-dongle-battery/blob/main/docs/HOW_IT_WORKS.md [С]
- API перевірки дозволу: `IOHIDCheckAccess(kIOHIDRequestTypeListenEvent)`, `Info.plist` ключ `NSInputMonitoringUsageDescription`, зміни діють після перезапуску додатка. Це з результатів пошуку та форумів (Apple Developer Forums thread 696673 і ін.), **офіційну сторінку документації Apple я не відкривав**. [Н] https://developer.apple.com/forums/thread/696673
- **`hidapi` на macOS** — це бекенд на `IOHidManager`. https://github.com/libusb/hidapi (README). Він працює з USB і Bluetooth HID-класу, але:
  - Composite-пристрій (клавіатура плюс vendor-інтерфейс) macOS подає як **один нероздільний пристрій**; за замовчуванням hidapi відкриває його ексклюзивно (`kIOHIDOptionsTypeSeizeDevice`), і тоді системі перестають надходити натискання; з прапорцем `0` і дозволом Input Monitoring це працювало. Це обговорення саме про BLE-пристрій, який є клавіатурою: https://github.com/libusb/hidapi/issues/344 [С, це досвід користувача в issue, не документація Apple].
  - Повідомлення в issue #266: із нестандартними usage pages поряд зі стандартними macOS іноді показує лише стандартні (у їхньому випадку допомогло прибрати інші usage pages): https://github.com/libusb/hidapi/issues/266 [Н]
- Для Flow 2 по USB, за даними `linder3hs`, це працює без хитрощів (є окрема raw HID collection). [С]
- **WebHID:** Chrome 89+, Edge (дзеркало Chrome), **Firefox: ні, Safari: ні** (MDN BCD: https://raw.githubusercontent.com/mdn/browser-compat-data/main/api/HID.json; сторінка MDN: https://developer.mozilla.org/en-US/docs/Web/API/WebHID_API). Тому VIA web потребує Chromium. [В]

### 4.3 Що реально можна робити через VIA raw HID [С]

Джерела: `ViaReport.swift` у https://github.com/linder3hs/lofree-flow-2 і `flow.md` у https://github.com/andresmarpz/flow; специфікація VIA: https://www.caniusevia.com/docs/specification, https://www.caniusevia.com/docs/custom_ui.

- Запит `0x01` версія протоколу. `custom_set_value` `0x07` / `custom_get_value` `0x08` / `custom_save` `0x09` у форматі `[cmd, channel, value_id, data...]`. Backlight = channel 1: brightness value id 1 (0-255), effect value id 2 (0 = fixed, 1 = breathing). Зроблено на 84-key, VIA-протокол v12.
- Keymap: `0x04` get_keycode, `0x05` set_keycode, `0x0C/0x0D` bulk get/set buffer (`flow.md`). Макроси та шари в стандартному VIA також через dynamic keymap, але **автор Flow 2-коду їх не реалізував**. [Н щодо повноти набору команд на цій прошивці]
- **Не доступно через VIA (за перевіркою `flow2ctl scan`):** таймаут вимкнення підсвітки в BT-режимі («not exposed via VIA (verified with flow2ctl scan)»). Патч прошивки для цього робить `dsamjjlenk`, але для 100-key і з ризиком. [С]
- **Батарея:** у BT-режимі береться з macOS: `system_profiler SPBluetoothDataType -json`, ключ `device_batteryLevelMain`; IOKit HID registry батарею Flow 2 не показує (за автором). Джерело: `BatteryMonitor.swift` у https://github.com/linder3hs/lofree-flow-2. [С] По USB-кабелю і по донглу батареї через VIA я не знайшов (для донгла Flow Lite 100 є HID-запит у DigitalEdens, але це інший протокол). [НЕ ПЕРЕВІРЕНО для Flow 2]
- **Touch bar:** працює як апаратна функція (гучність/яскравість), керування з хоста ніде не задокументовано. [НЕ ПЕРЕВІРЕНО]
- **RGB:** немає, підсвітка біла.

### 4.4 Реверс-інжиніринг (якщо VIA не вистачає)

- **Не потрібно для базового набору** (клавіші, підсвітка): протокол стандартний VIA. Потрібно лише для нестандартних функцій (touch bar, статус батареї за кабелем, BT-таймаут).
- **USB на macOS:** Wireshark має вбудований захоплювач (інтерфейс `XHC20`), але на Catalina і новіших треба **вимкнути SIP**. Джерела (результати пошуку, самі сторінки не відкривав): https://wiki.wireshark.org/CaptureSetup/USB, https://ask.wireshark.org/question/30854/how-do-i-capture-usb-traffic-on-macos/ [Н]. Зручніше захоплювати **VIA-трафік у Chrome на Windows з USBPcap**, якщо є Windows-машина; сам USBPcap мною не перевірявся. [НЕ ПЕРЕВІРЕНО]
- **Bluetooth:** PacketLogger входить у «Additional Tools for Xcode» (Hardware IO Tools) і показує HCI-трафік: https://developer.apple.com/bluetooth/ (з результату пошуку; сторінку не відкривав) [Н]. Оскільки по BT vendor-канал, найімовірніше, відсутній як такий, корисність невелика.
- **Аналіз прошивки:** репозиторії seobaeksol і dsamjjlenk показують, що офіційний `.hex` (ARM Cortex, `0x08000000`) можна дизасемблювати й патчити 4-байтовими змінами. Це працює, але ризик цегли реальний (див. samkey84), а bootloader Lofree окремий (`wb32-dfu`). [С]

### 4.5 Фолбек: ремап без vendor-протоколу

- **`hidutil`:** офіційна технотатка Apple TN2450 (https://developer.apple.com/library/archive/technotes/tn2450/_index.html): ремап через `hidutil property --set '{"UserKeyMapping":[...]}'`, ключі як `0x700000000 | usage`, «No special privileges are required», ремапи **зникають при перезавантаженні** або коли зникає keyboard service (наприклад, від'єднана остання клавіатура). Для постійності потрібен LaunchAgent. Обмеження за окремим пристроєм (`--matching`) у прочитаних мною джерелах не підтверджено. [В щодо переліченого; НЕ ПЕРЕВІРЕНО щодо per-device]
- **Karabiner-Elements:** умови `device_if` / `device_unless` з `vendor_id`, `product_id`, `is_keyboard` і **`device_address` (Bluetooth MAC, від 14.12.2)**: https://karabiner-elements.pqrs.org/docs/json/complex-modifications-manipulator-definition/conditions/device/ . Це дає ремапи саме для Flow 2 в USB, 2.4G і BT. [В]
- Обмеження обох способів: вони не змінюють вміст EEPROM клавіатури (шари/макроси в самій прошивці не з'являються), тому працюють лише на цьому Mac. Fn-шар і функції клавіш, які прошивка обробляє сама (BT-перемикання, touch bar), з хоста не перехопити. [С, міркування, а не цитата]

---

## 5. Вердикт і рекомендації

### 5.1 Вердикт

| Задача | Чи можливо | Впевненість | Як |
|---|---|---|---|
| Ремап клавіш, шари | **Так**, без власного софту | [В] | VIA у Chrome по USB (з ручним JSON) або Karabiner/`hidutil` для macOS-ремапу |
| Макроси | **Так**, у прошивці через VIA | [С] | VIA (dynamic macros); власний клієнт ще ніхто не зробив |
| Керування підсвіткою (не RGB) | **Так, по USB** | [С] | `linder3hs/lofree-flow-2` або свій `IOHIDManager`-клієнт |
| RGB | **Ні** (апаратно біла) | [В] | немає |
| Батарея | **Так, по Bluetooth** (системний сервіс) | [С] | `system_profiler` / `IOBluetooth`; по USB/2.4G не встановлено |
| Все по Bluetooth (шари, підсвітка) | **Ні** (за поточними даними) | [С] | ремап на боці macOS |
| Власна QMK-прошивка | **Технічно ні для повної** | [В] | немає коду і розпіновки; є лише неповний WIP |
| Touch bar з хоста | Невідомо | [Н] | потребує реверсу |

### 5.2 Кроки (від найдешевшого)

1. **Спробувати наявне:** зібрати або завантажити `linder3hs/lofree-flow-2` (release `Flow2-0.1.0-macos-arm64.zip`, потрібен правий клік → Open, бо не нотаризовано). Перевірити на своїй моделі командами `flow2ctl probe`, `flow2ctl light on`, `flow2ctl battery`. Це заодно покаже VID/PID і чи відповідає VIA v12 на вашій моделі.
2. **Ремап:** для клавіш, які потрібні лише на Mac, взяти Karabiner-Elements з `device_if` за `vendor_id 0x388D` (PID зняти з власного пристрою) або `hidutil` з LaunchAgent. Для змін у самій клавіатурі: usevia.app у Chrome, JSON з Dropbox-пакета Lofree.
3. **Перед будь-якою зміною EEPROM** зробити бекап keymap (bulk get `0x0C`) і пам'ятати комбінацію заводського скидання `Fn + Left Shift + Backspace` (з `flow.md`, не перевірено мною) [Н].
4. **Якщо потрібен власний macOS-клієнт з ремапом/макросами:** розширити `Flow2Kit` (MIT) командами keymap і макросів. Реверс не потрібен, лише VIA-специфікація https://www.caniusevia.com/docs/specification. Ризик низький: EEPROM скидається.
5. **Не чіпати прошивку**, якщо немає резервної копії офіційного `.hex` і терпимості до цегли (`samkey84`, `dsamjjlenk`, `seobaeksol` показують, що це можливо, але кожен з патчів підходить лише для конкретної моделі/версії v14).
6. **Дізнатися відсутнє:**
   - зміст Dropbox-пакета Lofree (є macOS-збірка конфігуратора чи ні);
   - чи відповідає VIA по 2.4G-донглу;
   - VID/PID для 84-key;
   - чи є Flow 2-специфічні профілі Karabiner (повторити пошук GitHub з авторизацією `gh`).

### 5.3 Що я не зміг перевірити (зведено)

- Вміст Dropbox-папок Lofree (версії, платформи конфігуратора).
- Робота VIA по 2.4G-донглу (лише з чужих слів).
- Причина відсутності VIA по BT (прошивка чи macOS).
- VID/PID і кодова назва моделі 84-key.
- Чи справді `kabolen/via-lofree-flow2` має Flow 2-специфічні зміни (у побачених комітах їх немає).
- Karabiner-профілі й Reddit-треди (пошук через GitHub API впирався в ліміт; Reddit не відкривав).
- Офіційна документація Apple по `IOHIDCheckAccess`/Input Monitoring і по PacketLogger (є лише через результати пошуку/форуми).
- Реальна робота `linder3hs/lofree-flow-2` і будь-якого іншого коду: я читав вихідний код і README, але **нічого не запускав**.
- Огляди Tom's Guide і Notebookcheck (сторінки не відкрилися).
- Існування офіційної macOS-версії конфігуратора для Flow 2 (мітка «For Windows» на сторінках продуктів свідчить проти, але це не доказ).

---

## Додаток: перелік джерел, які реально відкрито

Lofree:
- https://www.lofree.co/pages/download-center
- https://www.lofree.co/products/flow-2-68-low-profile-mechanical-keyboard
- https://www.lofree.co/products/flow-2-84-low-profile-mechanical-keyboard-for-mac
- https://www.lofree.co/products/flow-2-100-low-profile-mechanical-keyboard-for-mac

GitHub (README і/або вихідний код):
- https://github.com/qmk/qmk_firmware/issues/26062
- https://github.com/linder3hs/lofree-flow-2
- https://github.com/andresmarpz/flow
- https://github.com/sameid/samkey84
- https://github.com/dsamjjlenk/lofree-flow2-100-backlight-60s
- https://github.com/seobaeksol/lofree-flow-spacefn-layout
- https://github.com/piperamirez/lofree-flow2-os9-driver
- https://github.com/kabolen/via-lofree-flow2
- https://github.com/aaditagrawal/flow-control
- https://github.com/shikuiow/lofree-key-mapper-mac
- https://github.com/DigitalEdens/lofree-mac-2-4ghz-dongle-battery
- https://github.com/dmtrKovalenko/lofree-hypace-reverse-engineer
- https://github.com/alexeygumirov/lofree-flow-fn-fix
- https://gist.github.com/LC43/ef771d5feb6526ccfd8f223f9661cdf8 (параметри `hid_apple` для Linux, не Flow 2)
- https://github.com/the-via/keyboards (дерево файлів)
- https://github.com/libusb/hidapi (README, issues #344, #266)
- https://github.com/mdn/browser-compat-data (`api/HID.json`)

Документація:
- https://www.caniusevia.com/docs/specification, https://www.caniusevia.com/docs/custom_ui, https://www.caniusevia.com/docs/configuring_qmk
- https://developer.apple.com/library/archive/technotes/tn2450/_index.html
- https://karabiner-elements.pqrs.org/docs/json/complex-modifications-manipulator-definition/conditions/device/
- https://developer.mozilla.org/en-US/docs/Web/API/WebHID_API

Лише результати пошуку (сторінки не відкривав): Apple Developer Forums thread 696673, https://developer.apple.com/bluetooth/, https://wiki.wireshark.org/CaptureSetup/USB, https://ask.wireshark.org/question/30854/how-do-i-capture-usb-traffic-on-macos/, https://www.tomsguide.com/computing/keyboards/lofree-flow-2-review, Notebookcheck (403).
