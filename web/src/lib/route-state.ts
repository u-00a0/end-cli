import type { RouteKey } from "./routes";
import { parseHashRoute } from "./routes";

type RouteListener = (route: RouteKey) => void;

const routeListeners = new Set<RouteListener>();
let removeWindowListener: (() => void) | null = null;

function readRouteFromHash(): RouteKey {
  if (typeof window === "undefined") {
    return "home";
  }

  return parseHashRoute(window.location.hash);
}

function startWindowListenerIfNeeded(): void {
  if (removeWindowListener || typeof window === "undefined") {
    return;
  }

  const notify = (): void => {
    const nextRoute = parseHashRoute(window.location.hash);
    for (const listener of routeListeners) {
      listener(nextRoute);
    }
  };

  window.addEventListener("hashchange", notify);
  removeWindowListener = () => {
    window.removeEventListener("hashchange", notify);
    removeWindowListener = null;
  };
}

export function getCurrentHashRoute(): RouteKey {
  return readRouteFromHash();
}

export function observeHashRoute(onRouteChange: (route: RouteKey) => void): () => void {
  if (typeof window === "undefined") {
    onRouteChange("home");
    return () => undefined;
  }

  routeListeners.add(onRouteChange);
  startWindowListenerIfNeeded();
  onRouteChange(readRouteFromHash());

  return () => {
    routeListeners.delete(onRouteChange);
    if (routeListeners.size === 0 && removeWindowListener) {
      removeWindowListener();
    }
  };
}
