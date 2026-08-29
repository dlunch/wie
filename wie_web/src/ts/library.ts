import { extractAppMetadata } from "@pkg";
import { Check, CircleHelp, EllipsisVertical, Globe2, Plus, Settings, Trash2, Upload, X, createIcons } from "lucide";

import { AppLibraryStore, AppMetadata } from "./app_library_store";
import { SettingsController } from "./settings";

const APPS_PER_PAGE = 12;
const WELCOME_STORAGE_KEY = "wie_welcome_seen";
const icons = {
  Check,
  CircleHelp,
  EllipsisVertical,
  Globe2,
  Plus,
  Settings,
  Trash2,
  Upload,
  X,
};

export const initializeLibrary = async (launchApp: (app: AppMetadata, archive: Uint8Array) => Promise<void>, settings: SettingsController) => {
  const store = await AppLibraryStore.open();
  const libraryView = document.getElementById("library-view") as HTMLDivElement;
  const libraryPages = document.getElementById("library-pages") as HTMLDivElement;
  const pageIndicators = document.getElementById("page-indicators") as HTMLDivElement;
  const menuWrap = document.querySelector(".library-menu-wrap") as HTMLDivElement;
  const menuToggle = document.getElementById("library-menu-toggle") as HTMLButtonElement;
  const menu = document.getElementById("library-menu") as HTMLDivElement;
  const menuAddApp = document.getElementById("menu-add-app") as HTMLButtonElement;
  const menuManageApps = document.getElementById("menu-manage-apps") as HTMLButtonElement;
  const manageAppsLabel = document.getElementById("manage-apps-label") as HTMLSpanElement;
  const menuSettings = document.getElementById("menu-settings") as HTMLButtonElement;
  const menuHelp = document.getElementById("menu-help") as HTMLButtonElement;
  const importDialog = document.getElementById("import-dialog") as HTMLDialogElement;
  const chooseArchive = document.getElementById("choose-archive") as HTMLButtonElement;
  const archiveInput = document.getElementById("archive-input") as HTMLInputElement;
  const importStatus = document.getElementById("import-status") as HTMLParagraphElement;
  const deleteDialog = document.getElementById("delete-dialog") as HTMLDialogElement;
  const deleteAppTitle = document.getElementById("delete-app-title") as HTMLElement;
  const confirmDelete = document.getElementById("confirm-delete") as HTMLButtonElement;
  const helpDialog = document.getElementById("help-dialog") as HTMLDialogElement;
  const welcomeDialog = document.getElementById("welcome-dialog") as HTMLDialogElement;
  const dismissWelcome = document.getElementById("dismiss-welcome") as HTMLButtonElement;

  let apps = await store.list();
  let manageMode = false;
  let appStarting = false;
  let pendingDelete: AppMetadata;
  const iconUrls = new Set<string>();

  const closeMenu = () => {
    menu.classList.remove("visible");
    menuToggle.setAttribute("aria-expanded", "false");
  };

  const openImportDialog = () => {
    closeMenu();
    importStatus.textContent = "";
    importStatus.classList.remove("error");
    importDialog.showModal();
  };

  const renderLibrary = () => {
    for (const iconUrl of iconUrls) {
      URL.revokeObjectURL(iconUrl);
    }
    iconUrls.clear();
    libraryPages.replaceChildren();
    pageIndicators.replaceChildren();
    libraryView.classList.toggle("manage-mode", manageMode);
    menuManageApps.disabled = apps.length === 0;
    menuManageApps.setAttribute("aria-pressed", String(manageMode));
    manageAppsLabel.textContent = manageMode ? "삭제 완료" : "앱 삭제";

    const oldManageIcon = menuManageApps.querySelector("svg, i");
    const manageIcon = document.createElement("i");
    manageIcon.dataset.lucide = manageMode ? "check" : "trash-2";
    oldManageIcon?.replaceWith(manageIcon);

    const entries: Array<AppMetadata | undefined> = [...apps, undefined];
    const pageCount = Math.ceil(entries.length / APPS_PER_PAGE);

    for (let pageIndex = 0; pageIndex < pageCount; pageIndex += 1) {
      const page = document.createElement("section");
      page.className = "library-page";
      page.setAttribute("aria-label", `${pageIndex + 1} / ${pageCount} 페이지`);

      for (const app of entries.slice(pageIndex * APPS_PER_PAGE, (pageIndex + 1) * APPS_PER_PAGE)) {
        if (!app) {
          const addButton = document.createElement("button");
          addButton.className = "app-entry add-app";
          addButton.type = "button";
          addButton.title = "앱 추가";
          addButton.setAttribute("aria-label", "앱 추가");

          const icon = document.createElement("span");
          icon.className = "app-icon add-app-icon";
          const plus = document.createElement("i");
          plus.dataset.lucide = "plus";
          icon.appendChild(plus);

          const label = document.createElement("span");
          label.className = "app-title";
          label.textContent = "앱 추가";
          addButton.append(icon, label);
          addButton.addEventListener("click", openImportDialog);
          page.appendChild(addButton);
          continue;
        }

        const entry = document.createElement("div");
        entry.className = "app-entry";

        const launchButton = document.createElement("button");
        launchButton.className = "app-launch";
        launchButton.type = "button";
        launchButton.title = app.title;
        launchButton.disabled = manageMode;

        const icon = document.createElement("span");
        icon.className = "app-icon";
        const fallbackIcon = Array.from(app.title.trim())[0] ?? "?";
        if (app.icon) {
          const iconUrl = URL.createObjectURL(app.icon);
          const image = document.createElement("img");
          iconUrls.add(iconUrl);
          image.alt = "";
          image.src = iconUrl;
          image.addEventListener(
            "load",
            () => {
              URL.revokeObjectURL(iconUrl);
              iconUrls.delete(iconUrl);
            },
            { once: true },
          );
          image.addEventListener(
            "error",
            () => {
              URL.revokeObjectURL(iconUrl);
              iconUrls.delete(iconUrl);
              icon.textContent = fallbackIcon;
            },
            { once: true },
          );
          icon.appendChild(image);
        } else {
          icon.textContent = fallbackIcon;
        }

        const title = document.createElement("span");
        title.className = "app-title";
        title.textContent = app.title;
        launchButton.append(icon, title);
        launchButton.addEventListener("click", async () => {
          if (appStarting) {
            return;
          }

          appStarting = true;
          launchButton.disabled = true;
          try {
            const archive = await store.getArchive(app.id);
            if (!archive) {
              throw new Error("저장된 앱 파일을 찾을 수 없습니다.");
            }
            await launchApp(app, archive);
          } catch (error) {
            window.alert(String(error));
          } finally {
            appStarting = false;
            launchButton.disabled = false;
          }
        });

        const deleteButton = document.createElement("button");
        deleteButton.className = "delete-app-button";
        deleteButton.type = "button";
        deleteButton.title = `${app.title} 삭제`;
        deleteButton.setAttribute("aria-label", `${app.title} 삭제`);
        const trash = document.createElement("i");
        trash.dataset.lucide = "trash-2";
        deleteButton.appendChild(trash);
        deleteButton.addEventListener("click", () => {
          pendingDelete = app;
          deleteAppTitle.textContent = app.title;
          deleteDialog.showModal();
        });

        entry.append(launchButton, deleteButton);
        page.appendChild(entry);
      }

      libraryPages.appendChild(page);

      const indicator = document.createElement("button");
      indicator.className = "page-indicator";
      indicator.type = "button";
      indicator.setAttribute("aria-label", `${pageIndex + 1} 페이지로 이동`);
      indicator.classList.toggle("active", pageIndex === 0);
      indicator.addEventListener("click", () => {
        libraryPages.scrollTo({ left: pageIndex * libraryPages.clientWidth, behavior: "smooth" });
      });
      pageIndicators.appendChild(indicator);
    }

    pageIndicators.hidden = pageCount < 2;
    libraryPages.scrollLeft = 0;
    createIcons({ icons, root: libraryView });
    createIcons({ icons, root: importDialog });
    createIcons({ icons, root: deleteDialog });
    createIcons({ icons, root: helpDialog });
    createIcons({ icons, root: welcomeDialog });
  };

  libraryPages.addEventListener("scroll", () => {
    const pageIndex = Math.round(libraryPages.scrollLeft / libraryPages.clientWidth);
    for (const [index, indicator] of Array.from(pageIndicators.children).entries()) {
      indicator.classList.toggle("active", index === pageIndex);
    }
  });

  menuToggle.addEventListener("click", () => {
    const visible = menu.classList.toggle("visible");
    menuToggle.setAttribute("aria-expanded", String(visible));
  });
  document.addEventListener("click", (event) => {
    if (!menuWrap.contains(event.target as Node)) {
      closeMenu();
    }
  });

  menuAddApp.addEventListener("click", openImportDialog);
  menuManageApps.addEventListener("click", () => {
    manageMode = !manageMode;
    closeMenu();
    renderLibrary();
  });
  menuSettings.addEventListener("click", () => {
    closeMenu();
    settings.open();
  });
  menuHelp.addEventListener("click", () => {
    closeMenu();
    helpDialog.showModal();
  });

  chooseArchive.addEventListener("click", () => archiveInput.click());
  archiveInput.addEventListener("change", async () => {
    const file = archiveInput.files?.[0];
    if (!file) {
      return;
    }

    chooseArchive.disabled = true;
    importStatus.classList.remove("error");
    importStatus.textContent = "앱 파일을 확인하는 중입니다.";

    try {
      const filename = file.name.toLowerCase();
      if (!filename.endsWith(".zip") && !filename.endsWith(".jar")) {
        throw new Error("ZIP 또는 JAR 파일을 선택해 주세요.");
      }

      const archive = new Uint8Array(await file.arrayBuffer());
      const extracted = extractAppMetadata(file.name, archive);
      try {
        const duplicate = apps.find(app => app.id === extracted.id);
        if (duplicate) {
          throw new Error(`이미 라이브러리에 추가된 앱입니다: ${duplicate.title}`);
        }

        const icon = extracted.icon;
        const metadata: AppMetadata = {
          id: extracted.id,
          title: extracted.title,
          filename: file.name,
          addedAt: Date.now(),
        };
        if (icon.length > 0) {
          metadata.icon = new Blob([new Uint8Array(icon).buffer]);
        }
        await store.add(metadata, archive);
      } finally {
        extracted.free();
      }
      apps = await store.list();
      importDialog.close();
      renderLibrary();
    } catch (error) {
      importStatus.classList.add("error");
      importStatus.textContent = `앱을 추가할 수 없습니다. ${String(error)}`;
    } finally {
      chooseArchive.disabled = false;
      archiveInput.value = "";
    }
  });

  confirmDelete.addEventListener("click", async () => {
    confirmDelete.disabled = true;
    try {
      await store.delete(pendingDelete.id);
      apps = await store.list();
      if (apps.length === 0) {
        manageMode = false;
      }
      deleteDialog.close();
      renderLibrary();
    } catch (error) {
      window.alert(`앱을 삭제할 수 없습니다. ${String(error)}`);
    } finally {
      confirmDelete.disabled = false;
    }
  });
  dismissWelcome.addEventListener("click", () => {
    localStorage.setItem(WELCOME_STORAGE_KEY, "true");
    welcomeDialog.close();
  });

  renderLibrary();
  if (!localStorage.getItem(WELCOME_STORAGE_KEY)) {
    welcomeDialog.showModal();
  }
};
