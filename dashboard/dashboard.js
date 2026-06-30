const state = {
  health: null,
  players: [],
  violations: [],
  selectedPlayerId: 2,
};

const elements = {
  apiStatus: document.querySelector("#api-status"),
  eventCount: document.querySelector("#event-count"),
  violationCount: document.querySelector("#violation-count"),
  playerCount: document.querySelector("#player-count"),
  topScore: document.querySelector("#top-score"),
  playersList: document.querySelector("#players-list"),
  violationBreakdown: document.querySelector("#violation-breakdown"),
  timeline: document.querySelector("#timeline"),
  playerIdInput: document.querySelector("#player-id-input"),
  loadTimelineButton: document.querySelector("#load-timeline-button"),
  refreshButton: document.querySelector("#refresh-button"),
};

elements.refreshButton.addEventListener("click", () => {
  loadDashboard();
});

elements.loadTimelineButton.addEventListener("click", () => {
  const playerId = Number(elements.playerIdInput.value);

  if (Number.isInteger(playerId) && playerId > 0) {
    state.selectedPlayerId = playerId;
    loadTimeline(playerId);
  }
});

loadDashboard();

async function loadDashboard() {
  try {
    setStatus("loading");

    const [health, players, violations] = await Promise.all([
      getJson("/health"),
      getJson("/players/suspicious"),
      getJson("/violations/breakdown"),
    ]);

    state.health = health;
    state.players = players.players ?? [];
    state.violations = violations.violations ?? [];

    renderMetrics();
    renderPlayers();
    renderViolations();

    const firstPlayer = state.players[0]?.player_id ?? state.selectedPlayerId;
    state.selectedPlayerId = firstPlayer;
    elements.playerIdInput.value = firstPlayer;

    await loadTimeline(firstPlayer);

    setStatus("online");
  } catch (error) {
    setStatus("offline");
    renderError(elements.playersList, error);
    renderError(elements.violationBreakdown, error);
    renderError(elements.timeline, error);
  }
}

async function loadTimeline(playerId) {
  try {
    const payload = await getJson(`/players/${playerId}/timeline`);
    renderTimeline(payload.events ?? []);
  } catch (error) {
    renderError(elements.timeline, error);
  }
}

async function getJson(path) {
  const response = await fetch(path, {
    headers: {
      Accept: "application/json",
    },
  });

  if (!response.ok) {
    throw new Error(`${path} returned HTTP ${response.status}`);
  }

  return response.json();
}

function setStatus(value) {
  elements.apiStatus.textContent = value;
}

function renderMetrics() {
  const topScore = state.players.reduce(
    (max, player) => Math.max(max, player.severity_score),
    0,
  );

  elements.eventCount.textContent = formatNumber(state.health?.event_count ?? 0);
  elements.violationCount.textContent = formatNumber(state.health?.violation_count ?? 0);
  elements.playerCount.textContent = formatNumber(state.players.length);
  elements.topScore.textContent = formatNumber(topScore);
}

function renderPlayers() {
  if (state.players.length === 0) {
    renderEmpty(elements.playersList, "No suspicious players found.");
    return;
  }

  elements.playersList.replaceChildren(
    ...state.players.map((player) => {
      const row = document.createElement("article");
      row.className = "player-row";
      row.addEventListener("click", () => {
        state.selectedPlayerId = player.player_id;
        elements.playerIdInput.value = player.player_id;
        loadTimeline(player.player_id);
      });

      const main = document.createElement("div");
      main.className = "player-main";
      main.innerHTML = `
        <strong>PlayerId(${player.player_id})</strong>
        <div class="player-meta">
          ${player.report_count} reports · last seen ${player.last_seen_ms}ms
        </div>
      `;

      const score = document.createElement("div");
      score.className = "score";
      score.textContent = player.severity_score;

      row.append(main, score);

      return row;
    }),
  );
}

function renderViolations() {
  if (state.violations.length === 0) {
    renderEmpty(elements.violationBreakdown, "No violations found.");
    return;
  }

  elements.violationBreakdown.replaceChildren(
    ...state.violations.map((violation) => {
      const row = document.createElement("article");
      row.className = "breakdown-row";

      const severityClass = `severity-${violation.severity.toLowerCase()}`;

      row.innerHTML = `
        <strong>${violation.violation_code}</strong>
        <div class="breakdown-meta">
          <span class="${severityClass}">${violation.severity}</span>
          · ${violation.count} events
          · ${violation.first_seen_ms}ms → ${violation.last_seen_ms}ms
        </div>
      `;

      return row;
    }),
  );
}

function renderTimeline(events) {
  if (events.length === 0) {
    renderEmpty(elements.timeline, "No timeline rows for selected player.");
    return;
  }

  elements.timeline.replaceChildren(
    ...events.map((event) => {
      const row = document.createElement("article");
      row.className = "timeline-row";

      const connection = event.connection_id === null ? "-" : event.connection_id;
      const sequence = event.sequence === null ? "-" : event.sequence;

      row.innerHTML = `
        <div class="timeline-time">${padTime(event.server_time_ms)}ms</div>
        <div>
          <div class="timeline-type">${event.event_type}</div>
          <div class="timeline-meta">conn=${connection} seq=${sequence}</div>
        </div>
        <div class="timeline-summary">${escapeHtml(event.summary)}</div>
      `;

      return row;
    }),
  );
}

function renderEmpty(target, message) {
  const node = document.createElement("div");
  node.className = "empty";
  node.textContent = message;
  target.replaceChildren(node);
}

function renderError(target, error) {
  const node = document.createElement("div");
  node.className = "error";
  node.textContent = error.message;
  target.replaceChildren(node);
}

function formatNumber(value) {
  return new Intl.NumberFormat("en-US").format(value);
}

function padTime(value) {
  return String(value).padStart(8, "0");
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}