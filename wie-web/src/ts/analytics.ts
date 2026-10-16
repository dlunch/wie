type PageView = {
  page_location: string;
  page_title: string;
  page_referrer: string;
};

declare global {
  interface Window {
    gtag?: {
      (command: "set", parameters: PageView): void;
      (command: "event", name: "page_view", parameters: PageView): void;
    };
  }
}

const libraryUrl = location.href;
const libraryTitle = document.title;
let currentUrl = libraryUrl;

export const recordPageView = (gameTitle?: string) => {
  try {
    const page_location = gameTitle === undefined ? libraryUrl : new URL(`/games/${encodeURIComponent(gameTitle)}`, libraryUrl).href;
    if (page_location === currentUrl) return;
    const parameters: PageView = {
      page_location,
      page_title: gameTitle ?? libraryTitle,
      page_referrer: currentUrl,
    };
    currentUrl = page_location;
    window.gtag?.("set", parameters);
    window.gtag?.("event", "page_view", parameters);
  } catch {
    // Analytics must not interrupt gameplay or navigation.
  }
};
