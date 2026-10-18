import { Check, CircleHelp, EllipsisVertical, Globe2, Plus, Settings, Trash2, Upload, X, createIcons } from "lucide";

import type { LibraryApp, Wie } from "./backend";
import { t } from "./i18n";
import type { SettingsController } from "./settings";

const APPS_PER_PAGE = 12;
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

export const initializeLibrary = async (backend: Wie, launchApp: (app: LibraryApp) => Promise<void>, settings: SettingsController) => {
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
  const hideHelp = document.getElementById("hide-help") as HTMLInputElement;
  const welcomeDialog = document.getElementById("welcome-dialog") as HTMLDialogElement;
  const dismissWelcome = document.getElementById("dismiss-welcome") as HTMLButtonElement;

  hideHelp.checked = settings.get("helpDismissed");

  let apps = await backend.listApps();
  let manageMode = false;
  let appStarting = false;
  let pendingDelete: LibraryApp;
  const iconUrls = new Set<string>();
  let dragPointer: number | undefined;
  let dragStartX = 0;
  let dragStartScrollLeft = 0;
  let dragging = false;
  let suppressClick = false;
  let currentPageIndex = 0;
  let importStatusText = () => "";

  const setImportStatus = (text: () => string) => {
    importStatusText = text;
    importStatus.textContent = text();
  };

  const closeMenu = () => {
    menu.classList.remove("visible");
    menuToggle.setAttribute("aria-expanded", "false");
  };

  const openImportDialog = () => {
    closeMenu();
    setImportStatus(() => "");
    importStatus.classList.remove("error");
    importDialog.showModal();
  };

  const updateLabels = () => {
    importStatus.textContent = importStatusText();
    manageAppsLabel.textContent = t(manageMode ? "manageDone" : "deleteApps");
    const pages = libraryPages.querySelectorAll<HTMLElement>(".library-page");
    pages.forEach((page, index) => {
      page.setAttribute("aria-label", t("libraryPage", { page: index + 1, count: pages.length }));
    });
    const addButton = libraryPages.querySelector<HTMLButtonElement>(".add-app")!;
    addButton.title = t("addApp");
    addButton.setAttribute("aria-label", t("addApp"));
    addButton.querySelector(".app-title")!.textContent = t("addApp");
    libraryPages.querySelectorAll<HTMLButtonElement>(".delete-app-button").forEach((button, index) => {
      button.title = t("deleteApp", { title: apps[index].title });
      button.setAttribute("aria-label", button.title);
    });
    Array.from(pageIndicators.children).forEach((indicator, index) => {
      indicator.setAttribute("aria-label", t("goToPage", { page: index + 1 }));
    });
  };

  const renderLibrary = (targetPageIndex = currentPageIndex) => {
    for (const iconUrl of iconUrls) {
      URL.revokeObjectURL(iconUrl);
    }
    iconUrls.clear();
    libraryPages.replaceChildren();
    pageIndicators.replaceChildren();
    libraryView.classList.toggle("manage-mode", manageMode);
    menuManageApps.disabled = apps.length === 0;
    menuManageApps.setAttribute("aria-pressed", String(manageMode));

    const oldManageIcon = menuManageApps.querySelector("svg, i");
    const manageIcon = document.createElement("i");
    manageIcon.dataset.lucide = manageMode ? "check" : "trash-2";
    oldManageIcon?.replaceWith(manageIcon);

    const entries: Array<LibraryApp | undefined> = [...apps, undefined];
    const pageCount = Math.ceil(entries.length / APPS_PER_PAGE);
    currentPageIndex = Math.max(0, Math.min(targetPageIndex, pageCount - 1));
    libraryPages.classList.toggle("draggable", pageCount > 1);

    for (let pageIndex = 0; pageIndex < pageCount; pageIndex += 1) {
      const page = document.createElement("section");
      page.className = "library-page";

      for (const app of entries.slice(pageIndex * APPS_PER_PAGE, (pageIndex + 1) * APPS_PER_PAGE)) {
        if (!app) {
          const addButton = document.createElement("button");
          addButton.className = "app-entry add-app";
          addButton.type = "button";

          const icon = document.createElement("span");
          icon.className = "app-icon add-app-icon";
          const plus = document.createElement("i");
          plus.dataset.lucide = "plus";
          icon.appendChild(plus);

          const label = document.createElement("span");
          label.className = "app-title";
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
          image.draggable = false;
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
            if (!hideHelp.checked) {
              helpDialog.showModal();
              await new Promise<void>(resolve => helpDialog.addEventListener("close", () => resolve(), { once: true }));
            }
            await launchApp(app);
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
      indicator.classList.toggle("active", pageIndex === currentPageIndex);
      indicator.addEventListener("click", () => {
        libraryPages.scrollTo({ left: pageIndex * libraryPages.firstElementChild!.clientWidth, behavior: "smooth" });
      });
      pageIndicators.appendChild(indicator);
    }

    updateLabels();
    pageIndicators.hidden = pageCount < 2;
    libraryPages.scrollLeft = currentPageIndex * libraryPages.firstElementChild!.clientWidth;
    createIcons({ icons, root: libraryView });
    createIcons({ icons, root: importDialog });
    createIcons({ icons, root: deleteDialog });
    createIcons({ icons, root: helpDialog });
    createIcons({ icons, root: welcomeDialog });
  };

  libraryPages.addEventListener("scroll", () => {
    currentPageIndex = Math.round(libraryPages.scrollLeft / libraryPages.firstElementChild!.clientWidth);
    for (const [index, indicator] of Array.from(pageIndicators.children).entries()) {
      indicator.classList.toggle("active", index === currentPageIndex);
    }
  });
  libraryPages.addEventListener("pointerdown", event => {
    if (event.pointerType !== "mouse" || event.button !== 0 || libraryPages.scrollWidth <= libraryPages.clientWidth) {
      return;
    }

    dragPointer = event.pointerId;
    dragStartX = event.clientX;
    dragStartScrollLeft = libraryPages.scrollLeft;
    dragging = false;
  });
  window.addEventListener("pointermove", event => {
    if (event.pointerId !== dragPointer) {
      return;
    }

    const distance = event.clientX - dragStartX;
    if (!dragging && Math.abs(distance) < 6) {
      return;
    }

    if (!dragging) {
      dragging = true;
      libraryPages.classList.add("dragging");
      libraryPages.setPointerCapture(event.pointerId);
    }
    event.preventDefault();
    libraryPages.scrollLeft = dragStartScrollLeft - distance;
  });
  const finishMouseDrag = (event: PointerEvent) => {
    if (event.pointerId !== dragPointer) {
      return;
    }

    if (libraryPages.hasPointerCapture(event.pointerId)) {
      libraryPages.releasePointerCapture(event.pointerId);
    }
    libraryPages.classList.remove("dragging");
    dragPointer = undefined;

    if (dragging) {
      dragging = false;
      suppressClick = true;
      const pageWidth = libraryPages.firstElementChild!.clientWidth;
      const pageIndex = Math.round(libraryPages.scrollLeft / pageWidth);
      libraryPages.scrollTo({ left: pageIndex * pageWidth, behavior: "smooth" });
      window.setTimeout(() => {
        suppressClick = false;
      });
    }
  };
  window.addEventListener("pointerup", finishMouseDrag);
  window.addEventListener("pointercancel", finishMouseDrag);
  libraryPages.addEventListener(
    "click",
    event => {
      if (suppressClick) {
        event.preventDefault();
        event.stopPropagation();
        suppressClick = false;
      }
    },
    { capture: true },
  );

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
  helpDialog.addEventListener("close", () => {
    void settings.set("helpDismissed", hideHelp.checked).catch(error => {
      hideHelp.checked = settings.get("helpDismissed");
      window.alert(t("settingsSaveFailed", { error: String(error) }));
    });
  });

  const importFiles = async (files: File[]) => {
    if (files.length === 0 || chooseArchive.disabled) {
      return;
    }

    chooseArchive.disabled = true;
    importStatus.classList.remove("error");
    setImportStatus(() => t("checkingFiles", { count: files.length }));

    try {
      const failures: string[] = [];
      let addedCount = 0;

      for (const file of files) {
        try {
          const filename = file.name.toLowerCase();
          if (!filename.endsWith(".zip") && !filename.endsWith(".jar")) {
            throw new Error(t("invalidArchive"));
          }

          await backend.importApp(file);
          addedCount += 1;
        } catch (error) {
          failures.push(`${file.name}: ${error instanceof Error ? error.message : String(error)}`);
        }
      }

      if (addedCount > 0) {
        apps = await backend.listApps();
        renderLibrary(Math.floor(apps.length / APPS_PER_PAGE));
      }

      if (failures.length === 0) {
        importDialog.close();
      } else {
        importStatus.classList.add("error");
        setImportStatus(() => {
          const result = addedCount > 0 ? `${t("appsAdded", { count: addedCount })} ` : "";
          return `${result}${t("importFailures", { files: failures.join(" / ") })}`;
        });
      }
    } catch (error) {
      importStatus.classList.add("error");
      setImportStatus(() => t("importFailed", { error: String(error) }));
    } finally {
      chooseArchive.disabled = false;
      archiveInput.value = "";
    }
  };

  chooseArchive.addEventListener("click", () => archiveInput.click());
  archiveInput.addEventListener("change", () => {
    void importFiles(Array.from(archiveInput.files ?? []));
  });
  importDialog.addEventListener("dragover", event => {
    if (!event.dataTransfer?.types.includes("Files")) {
      return;
    }

    event.preventDefault();
    event.dataTransfer.dropEffect = chooseArchive.disabled ? "none" : "copy";
    chooseArchive.classList.toggle("drop-active", !chooseArchive.disabled);
  });
  importDialog.addEventListener("dragleave", event => {
    if (event.relatedTarget instanceof Node && importDialog.contains(event.relatedTarget)) {
      return;
    }
    chooseArchive.classList.remove("drop-active");
  });
  importDialog.addEventListener("drop", event => {
    event.preventDefault();
    chooseArchive.classList.remove("drop-active");
    void importFiles(Array.from(event.dataTransfer?.files ?? []));
  });
  importDialog.addEventListener("close", () => {
    chooseArchive.classList.remove("drop-active");
  });

  confirmDelete.addEventListener("click", async () => {
    confirmDelete.disabled = true;
    try {
      await backend.deleteApp(pendingDelete.id);
      apps = await backend.listApps();
      if (apps.length === 0) {
        manageMode = false;
      }
      deleteDialog.close();
      renderLibrary();
    } catch (error) {
      window.alert(t("deleteFailed", { error: String(error) }));
    } finally {
      confirmDelete.disabled = false;
    }
  });
  dismissWelcome.addEventListener("click", async () => {
    try {
      await settings.set("welcomeSeen", true);
      welcomeDialog.close();
    } catch (error) {
      window.alert(t("settingsSaveFailed", { error: String(error) }));
    }
  });

  document.addEventListener("wie:languagechange", updateLabels);
  renderLibrary();
  if (!settings.get("welcomeSeen")) {
    welcomeDialog.showModal();
  }
};
