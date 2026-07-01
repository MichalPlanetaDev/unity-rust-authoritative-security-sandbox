const state = {
  health: null,
  players: [],
  violations: [],
  selectedPlayerId: 2,
  isLoading: false,
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
  timelineForm: document.querySelector("#timeline-form"),
  refreshButton: document.querySelector("#refresh-button"),
};

elements.refreshButton.addEventListener("click", () => {
  loadDashboard();
});

elements.timelineForm.addEventListener("submit", (event) => {
  event.preventDefault();

  const playerId = Number(elements.playerIdInput.value);

  if (!Number.isInteger(playerId) || playerId <= 0) {
    renderError(elements.timeline, "Enter a valid synthetic player ID.");
    return;
  }

  state.selectedPlayerId = playerId;
  renderPlayers();
  loadTimeline(playerId);
});

loadDashboard();

async function loadDashboard() {
  if (state.isLoading) {
    return;
  }

  state.isLoading = true;
  setStatus("loading");
  setRefreshEnabled(false);

  renderLoading(elements.playersList, "Loading suspicious players.");
  renderLoading(elements.violationBreakdown, "Loading violation breakdown.");
  renderLoading(elements.timeline, "Loading player timeline.");

  try {
    const [health, playersPayload, violationsPayload] = await Promise.all([
      getJson("/health"),
      getJson("/players/suspicious"),
      getJson("/violations/breakdown"),
    ]);

    state.health = health;
    state.players = sortPlayers(playersPayload.players ?? []);
    state.violations = sortViolations(violationsPayload.violations ?? []);

    const firstPlayerId = state.players[0]?.player_id ?? state.selectedPlayerId;
    state.selectedPlayerId = firstPlayerId;
    elements.playerIdInput.value = firstPlayerId;

    renderMetrics();
    renderPlayers();
    renderViolations();

    await loadTimeline(firstPlayerId);

    setStatus("online");
  } catch (error) {
    setStatus("offline");
    renderError(elements.playersList, error.message);
    renderError(elements.violationBreakdown, error.message);
    renderError(elements.timeline, error.message);
  } finally {
    state.isLoading = false;
    setRefreshEnabled(true);
  }
}

async function loadTimeline(playerId) {
  renderLoading(elements.timeline, `Loading timeline for PlayerId(${playerId}).`);

  try {
    const payload = await getJson(`/players/${playerId}/timeline`);
    renderTimeline(payload.events ?? []);
  } catch (error) {
    renderError(elements.timeline, error.message);
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

function setRefreshEnabled(enabled) {
  elements.refreshButton.disabled = !enabled;
  elements.refreshButton.textContent = enabled ? "Refresh evidence" : "Refreshing";
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
    renderEmpty(
      elements.playersList,
      "No suspicious players found.",
      "The current investigation database does not contain validation findings.",
    );
    return;
  }

  elements.playersList.replaceChildren(
    ...state.players.map((player) => {
      const row = document.createElement("article");
      row.className = "player-row";

      if (player.player_id === state.selectedPlayerId) {
        row.classList.add("is-selected");
      }

      row.tabIndex = 0;
      row.setAttribute("role", "button");
      row.setAttribute("aria-label", `Load timeline for PlayerId(${player.player_id})`);

      row.addEventListener("click", () => selectPlayer(player.player_id));
      row.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          selectPlayer(player.player_id);
        }
      });

      const main = document.createElement("div");
      main.className = "player-main";

      const title = document.createElement("strong");
      title.textContent = `PlayerId(${player.player_id})`;

      const meta = document.createElement("div");
      meta.className = "player-meta";
      meta.textContent = [
        pluralize(player.report_count, "finding"),
        `last seen at ${formatMilliseconds(player.last_seen_ms)}`,
      ].join(" · ");

      const score = document.createElement("div");
      score.className = "score";
      score.textContent = player.severity_score;
      score.title = "Triage score derived from validation finding severity.";

      main.append(title, meta);
      row.append(main, score);

      return row;
    }),
  );
}

function selectPlayer(playerId) {
  state.selectedPlayerId = playerId;
  elements.playerIdInput.value = playerId;
  renderPlayers();
  loadTimeline(playerId);
}

function renderViolations() {
  if (state.violations.length === 0) {
    renderEmpty(
      elements.violationBreakdown,
      "No validation findings found.",
      "Run the synthetic bot scenarios and ingest the investigation database first.",
    );
    return;
  }

  elements.violationBreakdown.replaceChildren(
    ...state.violations.map((violation) => {
      const row = document.createElement("article");
      row.className = "breakdown-row";

      const title = document.createElement("strong");
      title.textContent = violation.violation_code;

      const meta = document.createElement("div");
      meta.className = "breakdown-meta";

      const severity = document.createElement("span");
      severity.className = `severity-${violation.severity.toLowerCase()}`;
      severity.textContent = violation.severity;

      meta.append(
        severity,
        document.createTextNode(
          ` · ${pluralize(violation.count, "finding")} · ${formatMilliseconds(
            violation.first_seen_ms,
          )} → ${formatMilliseconds(violation.last_seen_ms)}`,
        ),
      );

      row.append(title, meta);

      return row;
    }),
  );
}

function renderTimeline(events) {
  if (events.length === 0) {
    renderEmpty(
      elements.timeline,
      `No timeline rows for PlayerId(${state.selectedPlayerId}).`,
      "The player may not exist in the current investigation database.",
    );
    return;
  }

  elements.timeline.replaceChildren(
    ...events.map((event) => {
      const row = document.createElement("article");
      row.className = "timeline-row";

      const time = document.createElement("div");
      time.className = "timeline-time";
      time.textContent = formatMilliseconds(event.server_time_ms);

      const identity = document.createElement("div");

      const type = document.createElement("div");
      type.className = "timeline-type";
      type.textContent = event.event_type;

      const meta = document.createElement("div");
      meta.className = "timeline-meta";
      meta.textContent = `conn=${formatOptional(event.connection_id)} seq=${formatOptional(
        event.sequence,
      )}`;

      const summary = document.createElement("div");
      summary.className = "timeline-summary";
      summary.textContent = event.summary;

      identity.append(type, meta);
      row.append(time, identity, summary);

      return row;
    }),
  );
}

function renderLoading(target, message) {
  const node = document.createElement("div");
  node.className = "empty";

  const title = document.createElement("strong");
  title.textContent = message;

  const description = document.createElement("span");
  description.textContent = "Waiting for the investigation API response.";

  node.append(title, description);
  target.replaceChildren(node);
}

function renderEmpty(target, titleText, descriptionText) {
  const node = document.createElement("div");
  node.className = "empty";

  const title = document.createElement("strong");
  title.textContent = titleText;

  const description = document.createElement("span");
  description.textContent = descriptionText;

  node.append(title, description);
  target.replaceChildren(node);
}

function renderError(target, message) {
  const node = document.createElement("div");
  node.className = "error";

  const title = document.createElement("strong");
  title.textContent = "Investigation data unavailable";

  const description = document.createElement("span");
  description.textContent = message;

  node.append(title, description);
  target.replaceChildren(node);
}

function sortPlayers(players) {
  return [...players].sort((left, right) => {
    if (right.severity_score !== left.severity_score) {
      return right.severity_score - left.severity_score;
    }

    if (right.report_count !== left.report_count) {
      return right.report_count - left.report_count;
    }

    return left.player_id - right.player_id;
  });
}

function sortViolations(violations) {
  return [...violations].sort((left, right) => {
    const severityDelta = severityRank(right.severity) - severityRank(left.severity);

    if (severityDelta !== 0) {
      return severityDelta;
    }

    if (right.count !== left.count) {
      return right.count - left.count;
    }

    return left.violation_code.localeCompare(right.violation_code);
  });
}

function severityRank(severity) {
  switch (severity) {
    case "Critical":
      return 4;
    case "High":
      return 3;
    case "Medium":
      return 2;
    case "Low":
      return 1;
    default:
      return 0;
  }
}

function formatNumber(value) {
  return new Intl.NumberFormat("en-US").format(value);
}

function formatMilliseconds(value) {
  return `${formatNumber(value)}ms`;
}

function formatOptional(value) {
  return value === null || value === undefined ? "—" : value;
}

function pluralize(count, singular) {
  const suffix = count === 1 ? "" : "s";
  return `${formatNumber(count)} ${singular}${suffix}`;
}