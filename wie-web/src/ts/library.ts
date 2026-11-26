import { CircleHelp, EllipsisVertical, Globe2, Plus, Settings, Trash2, Upload, X, createIcons } from "lucide";

import type { LibraryApp, Wie } from "./backend";
import { t } from "./i18n";
import type { SettingsController } from "./settings";

const APPS_PER_PAGE = 12;
const icons = {
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
  const menuToggle = document.getElementById("library-menu-toggle") as HTMLButtonElement;
  const menu = document.getElementById("library-menu") as HTMLDivElement;
  const menuAddApp = document.getElementById("menu-add-app") as HTMLButtonElement;
  const appMenu = document.getElementById("app-menu") as HTMLDivElement;
  const menuDeleteApp = document.getElementById("menu-delete-app") as HTMLButtonElement;
  const menuDeleteStorage = document.getElementById("menu-delete-storage") as HTMLButtonElement;
  const menuSettings = document.getElementById("menu-settings") as HTMLButtonElement;
  const menuHelp = document.getElementById("menu-help") as HTMLButtonElement;
  const importDialog = document.getElementById("import-dialog") as HTMLDialogElement;
  const chooseArchive = document.getElementById("choose-archive") as HTMLButtonElement;
  const archiveInput = document.getElementById("archive-input") as HTMLInputElement;
  const importStatus = document.getElementById("import-status") as HTMLParagraphElement;
  const deleteDialog = document.getElementById("delete-dialog") as HTMLDialogElement;
  const deleteHeading = document.getElementById("delete-heading") as HTMLElement;
  const deleteDescription = document.getElementById("delete-description") as HTMLElement;
  const deleteAppTitle = document.getElementById("delete-app-title") as HTMLElement;
  const confirmDelete = document.getElementById("confirm-delete") as HTMLButtonElement;
  const helpDialog = document.getElementById("help-dialog") as HTMLDialogElement;
  const hideHelp = document.getElementById("hide-help") as HTMLInputElement;
  const welcomeDialog = document.getElementById("welcome-dialog") as HTMLDialogElement;
  const dismissWelcome = document.getElementById("dismiss-welcome") as HTMLButtonElement;

  hideHelp.checked = settings.get("helpDismissed");

  let apps = await backend.listApps();
  let activeMenu: { popup: HTMLDivElement; toggle: HTMLButtonElement; app?: LibraryApp } | undefined;
  let appStarting = false;
  let pendingDelete: { app: LibraryApp; storage: boolean };
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
    if (!activeMenu) {
      return;
    }
    activeMenu.popup.classList.remove("visible");
    activeMenu.toggle.setAttribute("aria-expanded", "false");
    activeMenu = undefined;
  };

  const toggleMenu = (popup: HTMLDivElement, toggle: HTMLButtonElement, app?: LibraryApp) => {
    const wasOpen = activeMenu?.toggle === toggle;
    closeMenu();
    if (wasOpen) {
      return;
    }

    activeMenu = { popup, toggle, app };
    popup.classList.add("visible");
    toggle.setAttribute("aria-expanded", "true");
    if (app) {
      const bounds = toggle.getBoundingClientRect();
      const { offsetWidth: width, offsetHeight: height } = popup;
      const left = Math.max(0, Math.min(bounds.right - width, window.innerWidth - width));
      const top = bounds.bottom + height <= window.innerHeight ? bounds.bottom : Math.max(0, bounds.top - height);
      popup.style.left = `${left}px`;
      popup.style.top = `${top}px`;
    }
    popup.querySelector<HTMLElement>('[role="menuitem"]')!.focus({ preventScroll: true });
  };

  const openImportDialog = () => {
    closeMenu();
    setImportStatus(() => "");
    importStatus.classList.remove("error");
    importDialog.showModal();
  };

  const updateLabels = () => {
    importStatus.textContent = importStatusText();
    const pages = libraryPages.querySelectorAll<HTMLElement>(".library-page");
    pages.forEach((page, index) => {
      page.setAttribute("aria-label", t("libraryPage", { page: index + 1, count: pages.length }));
    });
    const addButton = libraryPages.querySelector<HTMLButtonElement>(".add-app")!;
    addButton.title = t("addApp");
    addButton.setAttribute("aria-label", t("addApp"));
    addButton.querySelector(".app-title")!.textContent = t("addApp");
    libraryPages.querySelectorAll<HTMLButtonElement>(".app-menu-toggle").forEach((button, index) => {
      button.title = t("appMenu", { title: apps[index].title });
      button.setAttribute("aria-label", button.title);
    });
    Array.from(pageIndicators.children).forEach((indicator, index) => {
      indicator.setAttribute("aria-label", t("goToPage", { page: index + 1 }));
    });
  };

  const renderLibrary = (targetPageIndex = currentPageIndex) => {
    closeMenu();
    for (const iconUrl of iconUrls) {
      URL.revokeObjectURL(iconUrl);
    }
    iconUrls.clear();
    libraryPages.replaceChildren();
    pageIndicators.replaceChildren();
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

        const appMenuToggle = document.createElement("button");
        appMenuToggle.className = "icon-button app-menu-toggle";
        appMenuToggle.type = "button";
        appMenuToggle.setAttribute("aria-haspopup", "menu");
        appMenuToggle.setAttribute("aria-expanded", "false");
        appMenuToggle.setAttribute("aria-controls", "app-menu");
        const ellipsis = document.createElement("i");
        ellipsis.dataset.lucide = "ellipsis-vertical";
        appMenuToggle.appendChild(ellipsis);
        appMenuToggle.addEventListener("click", () => toggleMenu(appMenu, appMenuToggle, app));

        entry.append(launchButton, appMenuToggle);
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

  document.addEventListener("scroll", closeMenu, { capture: true });
  window.addEventListener("resize", closeMenu);
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

  menuToggle.addEventListener("click", () => toggleMenu(menu, menuToggle));
  document.addEventListener("click", event => {
    const target = event.target as Element;
    if (activeMenu && !activeMenu.toggle.contains(target) &&
      (!activeMenu.popup.contains(target) || target.closest('[role="menuitem"]'))) {
      closeMenu();
    }
  });
  document.addEventListener("keydown", event => {
    if (event.key === "Escape" && activeMenu) {
      activeMenu.toggle.focus({ preventScroll: true });
      closeMenu();
    }
  });

  const openDeleteDialog = (storage: boolean) => {
    if (!activeMenu?.app) {
      return;
    }

    pendingDelete = { app: activeMenu.app, storage };
    activeMenu.toggle.focus({ preventScroll: true });
    closeMenu();
    deleteAppTitle.textContent = pendingDelete.app.title;
    deleteHeading.textContent = t(storage ? "deleteStorage" : "deleteApp");
    deleteDescription.textContent = t(storage ? "deleteStorageConfirmation" : "deleteConfirmation");
    deleteDialog.showModal();
  };

  menuDeleteApp.addEventListener("click", () => openDeleteDialog(false));
  menuDeleteStorage.addEventListener("click", () => openDeleteDialog(true));
  menuAddApp.addEventListener("click", openImportDialog);
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
    const { app, storage } = pendingDelete;
    confirmDelete.disabled = true;
    try {
      if (storage) {
        await backend.deleteAppStorage(app.id);
      } else {
        await backend.deleteApp(app.id);
        apps = await backend.listApps();
        renderLibrary();
      }
      deleteDialog.close();
    } catch (error) {
      window.alert(t(storage ? "deleteStorageFailed" : "deleteFailed", { error: String(error) }));
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
