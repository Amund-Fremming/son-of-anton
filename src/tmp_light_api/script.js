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

// Set all handlers
function addListeners() {
  addLoginListener();

  addLightListener("off", "AllOff");
  addLightListener("on", "AllOn");
  addLightListener("movie", "MovieMode");
  addLightListener("party", "PartyMode");
  addLightListener("night", "NightMode");
}
