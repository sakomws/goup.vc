import { showInfoAlert } from "/static/js/common/alerts.js";
import {
  consumePendingDeploymentRefreshAlert,
  DEPLOYMENT_REFRESH_MESSAGE,
} from "/static/js/common/deployment-version.js";
import {
  registerHtmxNoEmptyValuesExtensions,
  registerHtmxResponseHandlers,
} from "/static/js/common/htmx-extensions.js";
import { resetRestoredModalState } from "/static/js/common/modals/modal-lifecycle.js";
import "/static/js/common/profile-completion-alert.js";

// Install request filtering before HTMX builds GET query strings.
registerHtmxNoEmptyValuesExtensions(window.htmx);
// Wire document-level handlers for alerts, 404 swaps, and deployment checks.
registerHtmxResponseHandlers(document);

// Show the one-shot notice queued before a deployment-triggered reload.
if (consumePendingDeploymentRefreshAlert()) {
  showInfoAlert(DEPLOYMENT_REFRESH_MESSAGE);
}

// HTMX can restore cached snapshots without running module scripts again.
document.addEventListener("htmx:historyRestore", () => {
  resetRestoredModalState(document);
});

// Native Back/Forward cache restores need the same stale modal cleanup.
window.addEventListener("pageshow", (event) => {
  if (event.persisted) {
    resetRestoredModalState(document);
  }
});

function websocketUrl(path) {
  const scheme = window.location.protocol === "https:" ? "wss:" : "ws:";
  return `${scheme}//${window.location.host}${path}`;
}

function refreshDiscoveryDashboard(scope) {
  if (scope === "user" && window.location.pathname === "/dashboard/jobs/discovery") {
    window.location.reload();
    return;
  }

  if (
    scope === "group" &&
    document.querySelector('form[hx-put="/dashboard/group/integrations"]')
  ) {
    window.htmx.ajax("GET", "/dashboard/group/integrations", {
      target: "#dashboard-content",
      swap: "innerHTML",
    });
  }
}

function subscribeToDiscoveryUpdates(path, expectedScope) {
  let retried = false;

  function connect() {
    const socket = new WebSocket(websocketUrl(path));
    let opened = false;

    socket.addEventListener("open", () => {
      opened = true;
    });
    socket.addEventListener("message", (event) => {
      try {
        const notification = JSON.parse(event.data);
        if (notification.scope === expectedScope) {
          refreshDiscoveryDashboard(expectedScope);
        }
      } catch {
        // Ignore malformed messages; polling remains available as a fallback.
      }
    });
    socket.addEventListener("close", () => {
      // A failed initial handshake means realtime is not enabled. Do not retry it.
      if (opened && !retried) {
        retried = true;
        window.setTimeout(connect, 3_000);
      }
    });
  }

  connect();
}

if (window.location.pathname === "/dashboard/jobs/discovery") {
  subscribeToDiscoveryUpdates("/ws/discovery/user", "user");
}
if (window.location.pathname.startsWith("/dashboard/group")) {
  subscribeToDiscoveryUpdates("/ws/discovery/group", "group");
}
