import { configStore } from "./config_store";

export type LanguagePreference = "system" | "ko" | "en";

const ko = {
  pageTitle: "wie - 피처폰 앱 에뮬레이터 (WIPI/J2ME)",
  pageDescription: "옛 한국 피처폰의 WIPI/J2ME 앱을 웹 브라우저에서 바로 실행하는 에뮬레이터. 앱을 브라우저에 보관하고 모바일과 PC에서 실행할 수 있습니다.",
  pageKeywords: "wie, WIPI, J2ME, 피처폰, 피처폰 앱, 에뮬레이터, 모바일 앱, 추억의 앱, 휴대폰 에뮬레이터",
  socialTitle: "wie - 피처폰 앱 에뮬레이터",
  socialDescription: "옛 한국 피처폰의 WIPI/J2ME 앱을 웹 브라우저에서 바로 실행하는 에뮬레이터.",
  socialLocale: "ko_KR",
  library: "라이브러리",
  menu: "메뉴",
  settings: "설정",
  help: "조작 도움말",
  projectWebsite: "프로젝트 웹사이트",
  libraryLoading: "라이브러리를 불러오는 중입니다.",
  libraryPages: "라이브러리 페이지",
  advertisement: "광고",
  backToLibrary: "라이브러리로 돌아가기",
  preparing: "실행 준비중",
  keypad: "앱 키패드",
  up: "위",
  left: "왼쪽",
  right: "오른쪽",
  down: "아래",
  close: "닫기",
  soundEffects: "효과음",
  language: "언어",
  systemLanguage: "시스템 기본값",
  welcome: "wie에 오신 것을 환영합니다",
  welcomeDescription: "wie는 WIPI와 J2ME 앱을 이 기기에서 보관하고 실행할 수 있는 에뮬레이터입니다.",
  welcomeAdd: "WIPI ZIP 또는 J2ME JAR 파일을 선택합니다.",
  launchApp: "앱 실행",
  welcomeLaunch: "라이브러리에서 추가된 앱 아이콘을 누릅니다.",
  manageApps: "앱 관리",
  welcomeManage: "오른쪽 위 메뉴에서 앱을 삭제하거나 설정을 변경합니다.",
  dismissWelcome: "닫고 더 이상 보지 않기",
  chooseArchiveDescription: "WIPI ZIP 또는 J2ME JAR 파일을 선택하거나 이 창에 끌어다 놓으세요.",
  chooseFile: "파일 선택",
  deleteConfirmation: "이 앱을 라이브러리에서 삭제합니다.",
  cancel: "취소",
  delete: "삭제",
  numbers123: "숫자 1, 2, 3",
  numbers456: "숫자 4, 5, 6",
  numbers789: "숫자 7, 8, 9",
  arrowKeys: "방향키",
  move: "이동",
  hideHelp: "다시 보지 않기",
  restartRequired: "오류로 실행이 중단되었습니다. wie 앱을 완전히 종료한 뒤 다시 실행해 주세요.",
  emulatorStopped: "에뮬레이터가 중단되었습니다. 다시 실행하려면 페이지를 새로고침해 주세요.",
  libraryOpenFailed: "라이브러리를 열 수 없습니다. {error}",
  settingsSaveFailed: "설정을 저장할 수 없습니다. {error}",
  manageDone: "삭제 완료",
  deleteApps: "앱 삭제",
  addApp: "앱 추가",
  libraryPage: "{page} / {count} 페이지",
  deleteApp: "{title} 삭제",
  goToPage: "{page} 페이지로 이동",
  checkingFiles: "{count}개 앱 파일을 확인하는 중입니다.",
  invalidArchive: "ZIP 또는 JAR 파일이 아닙니다.",
  appsAdded: "{count}개 앱을 추가했습니다.",
  importFailures: "추가하지 못한 파일: {files}",
  importFailed: "앱을 추가할 수 없습니다. {error}",
  deleteFailed: "앱을 삭제할 수 없습니다. {error}",
  appAlreadyAdded: "이미 라이브러리에 추가된 앱입니다: {title}",
  appFileMissing: "저장된 앱 파일을 찾을 수 없습니다.",
};

export type MessageKey = keyof typeof ko;

const en: Record<MessageKey, string> = {
  pageTitle: "wie - Feature Phone App Emulator (WIPI/J2ME)",
  pageDescription: "Run WIPI/J2ME apps from Korean feature phones directly in your browser. Keep apps in your browser and run them on mobile or desktop.",
  pageKeywords: "wie, WIPI, J2ME, feature phone, feature phone apps, emulator, mobile apps, retro apps, phone emulator",
  socialTitle: "wie - Feature Phone App Emulator",
  socialDescription: "Run WIPI/J2ME apps from Korean feature phones directly in your browser.",
  socialLocale: "en_US",
  library: "Library",
  menu: "Menu",
  settings: "Settings",
  help: "Controls",
  projectWebsite: "Project website",
  libraryLoading: "Loading library.",
  libraryPages: "Library pages",
  advertisement: "Advertisement",
  backToLibrary: "Back to library",
  preparing: "Preparing to start",
  keypad: "App keypad",
  up: "Up",
  left: "Left",
  right: "Right",
  down: "Down",
  close: "Close",
  soundEffects: "Sound effects",
  language: "Language",
  systemLanguage: "System default",
  welcome: "Welcome to wie",
  welcomeDescription: "wie is an emulator for storing and running WIPI and J2ME apps on this device.",
  welcomeAdd: "Choose WIPI ZIP or J2ME JAR files.",
  launchApp: "Run an app",
  welcomeLaunch: "Select an app icon in the library.",
  manageApps: "Manage apps",
  welcomeManage: "Delete apps or change settings from the top-right menu.",
  dismissWelcome: "Close and don't show again",
  chooseArchiveDescription: "Choose WIPI ZIP or J2ME JAR files, or drag and drop them here.",
  chooseFile: "Choose files",
  deleteConfirmation: "This app will be deleted from the library.",
  cancel: "Cancel",
  delete: "Delete",
  numbers123: "Numbers 1, 2, 3",
  numbers456: "Numbers 4, 5, 6",
  numbers789: "Numbers 7, 8, 9",
  arrowKeys: "Arrow keys",
  move: "Move",
  hideHelp: "Don't show again",
  restartRequired: "An error stopped the app. Fully close wie and open it again.",
  emulatorStopped: "The emulator has stopped. Refresh the page to run it again.",
  libraryOpenFailed: "Could not open the library. {error}",
  settingsSaveFailed: "Could not save settings. {error}",
  manageDone: "Done deleting",
  deleteApps: "Delete apps",
  addApp: "Add app",
  libraryPage: "Page {page} of {count}",
  deleteApp: "Delete {title}",
  goToPage: "Go to page {page}",
  checkingFiles: "Checking app files: {count}.",
  invalidArchive: "Not a ZIP or JAR file.",
  appsAdded: "Apps added: {count}.",
  importFailures: "Files not added: {files}",
  importFailed: "Could not add apps. {error}",
  deleteFailed: "Could not delete the app. {error}",
  appAlreadyAdded: "This app is already in the library: {title}",
  appFileMissing: "The saved app file could not be found.",
};

let preference: LanguagePreference = "system";
let locale: "ko" | "en" = "en";

export function t(key: MessageKey, params: Record<string, string | number> = {}): string {
  return (locale === "ko" ? ko : en)[key].replace(/\{(\w+)\}/g, (placeholder, name: string) =>
    Object.hasOwn(params, name) ? String(params[name]) : placeholder,
  );
}

export function getLocale(): "ko" | "en" {
  return locale;
}

export function getLanguagePreference(): LanguagePreference {
  return preference;
}

function updateLocalization(): void {
  const next = preference === "system" ? (/^ko(?:-|$)/i.test(navigator.language) ? "ko" : "en") : preference;
  const changed = locale !== next;
  locale = next;
  document.documentElement.lang = locale;
  for (const attribute of ["text", "title", "aria-label", "content"] as const) {
    for (const element of document.querySelectorAll<HTMLElement>(`[data-i18n-${attribute}]`)) {
      const key = element.getAttribute(`data-i18n-${attribute}`) as MessageKey;
      if (attribute === "text") {
        element.textContent = t(key);
      } else {
        element.setAttribute(attribute, t(key));
      }
    }
  }
  if (changed) document.dispatchEvent(new Event("wie:languagechange"));
}

export function initializeLocalization(): void {
  preference = configStore.get("language");
  updateLocalization();
  window.addEventListener("languagechange", updateLocalization);
}

export function setLanguagePreference(value: LanguagePreference): void {
  configStore.set("language", value);
  preference = value;
  updateLocalization();
}
