document.addEventListener("DOMContentLoaded", addListeners);

// Handle login
function addLoginListener() {
  const loginButton = document.getElementById("login");
  if (!loginButton)
    return;

  const loginForm = loginButton.closest("form");
  loginForm.addEventListener("submit", async function (e) {
    e.preventDefault();

    const passwordInput = document.getElementById("password").value;
    localStorage.setItem("password", passwordInput);

    try {
      const response = await fetch(`/login/${passwordInput}`, {
        method: "POST",
      });

      if (response.ok || response.status === 303) {
        window.location.href = `/control/${passwordInput}`;
      } else {
        alert("Invalid password");
      }
    } catch (error) {
      alert("Login failed: " + error.message);
    }
  });
}

function addLightListener(id, action) {
  const button = document.getElementById(id);
  if (!button)
    return;

  button.addEventListener("click", async function (e) {
    e.preventDefault();
    try {
      const password = localStorage.getItem("password");
      console.log(password);
      if (!password) {
        window.location.href = "/";
        return;
      }
      const response = await fetch(`/set-light/${password}/${action}`, {
        method: "POST",
      });

      if (!response.ok) {
        alert("Failed to control light");
      }
    } catch (error) {
      alert("Light control failed: " + error.message);
    }
  })
}

// Handle logs link click
function addLogsLinkListener() {
  const logsLink = document.getElementById("logs-link");
  if (!logsLink)
    return;

  logsLink.addEventListener("click", function (e) {
    e.preventDefault();
    const password = localStorage.getItem("password");
    if (!password) {
      window.location.href = "/";
      return;
    }
    window.location.href = `/logs/${password}`;
  });
}

// Handle back to control button
function addBackToControlListener() {
  const backButton = document.getElementById("back-to-control");
  if (!backButton)
    return;

  backButton.addEventListener("click", function (e) {
    e.preventDefault();
    const password = localStorage.getItem("password");
    if (!password) {
      window.location.href = "/";
      return;
    }
    window.location.href = `/control/${password}`;
  });
}

// Load and display logs
async function loadLogs() {
  const logsList = document.getElementById("logs-list");
  if (!logsList)
    return;

  const password = localStorage.getItem("password");
  if (!password) {
    window.location.href = "/";
    return;
  }

  try {
    const response = await fetch(`/api/logs/${password}`);
    if (!response.ok) {
      logsList.innerHTML = '<div class="empty-logs">Failed to load logs</div>';
      return;
    }

    const logs = await response.json();
    
    if (logs.length === 0) {
      logsList.innerHTML = '<div class="empty-logs">No logs available</div>';
      return;
    }

    // Display logs in reverse order (newest first)
    logsList.innerHTML = logs.reverse().map(([timestamp, message]) => `
      <div class="log-entry">
        <div class="log-timestamp">${timestamp}</div>
        <div class="log-message">${message}</div>
      </div>
    `).join('');
  } catch (error) {
    logsList.innerHTML = '<div class="empty-logs">Error loading logs: ' + error.message + '</div>';
  }
}

// Set all handlers
function addListeners() {
  addLoginListener();

  addLightListener("off", "AllOff");
  addLightListener("on", "AllOn");
  addLightListener("movie", "MovieMode");
  addLightListener("party", "PartyMode");
  addLightListener("night", "NightMode");

  addLogsLinkListener();
  addBackToControlListener();
  loadLogs();
}
